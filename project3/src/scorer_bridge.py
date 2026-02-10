"""Bridge between project3 and project1 scoring infrastructure.

Handles sys.path setup, loads LocalRanker and TopicScorer from project1,
and provides adapter functions for converting JournalArticle to the
formats expected by those scorers.

Project3's config module is named 'p3_config' to avoid collision with
project1's 'config' module when both are on sys.path.
"""

from pathlib import Path

from p3_config import Project3Config, setup_project1_imports
from models import JournalArticle


def article_to_paper_record(article: JournalArticle):
    """Convert a JournalArticle to a project1 PaperRecord.

    Journal articles lack arXiv categories, so categories and
    primary_category are left empty. This triggers the weight
    normalization logic in LocalRanker.score_papers().

    Args:
        article: The journal article to convert.

    Returns:
        song_db.models.PaperRecord instance.
    """
    from song_db.models import PaperRecord

    return PaperRecord(
        arxiv_id=article.doi or article.arxiv_id or article.title[:50],
        title=article.title,
        abstract=article.abstract,
        categories=[],
        primary_category="",
        published=article.published,
    )


def load_local_ranker(p3_config: Project3Config):
    """Load the LocalRanker from project1's interest model.

    Args:
        p3_config: Project3 config (for project1 path).

    Returns:
        Tuple of (LocalRanker, Embedder) instances.
    """
    setup_project1_imports(p3_config.project1_config_path)

    from config import load_config as load_p1_config
    from song_db.distill import load_interest_model
    from song_db.embeddings import Embedder
    from song_db.rank import LocalRanker

    p1_config = load_p1_config(p3_config.project1_config_path)
    p1_root = p3_config.project1_config_path.parent

    model_path = p1_root / p1_config.local_filter.interest_model
    if not model_path.exists():
        raise FileNotFoundError(f"Interest model not found: {model_path}")

    interest_model = load_interest_model(model_path)
    interest_model.scoring_weights = p1_config.local_filter.weights

    embedder = Embedder()
    ranker = LocalRanker(interest_model, embedder)

    return ranker, embedder


def load_topic_scorer(p3_config: Project3Config, embedder):
    """Load the TopicScorer from project1, reusing the same Embedder.

    Args:
        p3_config: Project3 config (for project1 path).
        embedder: Pre-loaded Embedder instance (shared with LocalRanker).

    Returns:
        Tuple of (TopicScorer, Config) from project1.
    """
    setup_project1_imports(p3_config.project1_config_path)

    from config import load_config as load_p1_config
    from topic_scorer import TopicScorer

    p1_config = load_p1_config(p3_config.project1_config_path)
    scorer = TopicScorer(p1_config, embedder)

    return scorer, p1_config


def filter_with_local_ranker(
    articles: list[JournalArticle],
    ranker,
    threshold: float,
) -> list[JournalArticle]:
    """Filter articles using the LocalRanker (Stage 1 corpus filter).

    Args:
        articles: All parsed articles.
        ranker: LocalRanker instance.
        threshold: Minimum score to pass (0-1 scale).

    Returns:
        Articles that pass the threshold (field-relevant papers).
    """
    if not articles:
        return []

    records = [article_to_paper_record(a) for a in articles]
    scores = ranker.score_papers(records)

    passed = []
    for article, score in zip(articles, scores):
        if score.score_total >= threshold:
            passed.append(article)

    print(f"  LocalRanker: {len(passed)}/{len(articles)} passed (threshold={threshold})")
    return passed


def score_with_topic_scorer(
    articles: list[JournalArticle],
    scorer,
    p1_config,
):
    """Score articles using the TopicScorer (Stage 2 topic ranking).

    TopicScorer expects objects with .title and .abstract attributes.
    JournalArticle already has these, so it works via duck typing.

    Args:
        articles: Stage 1 survivors (field-relevant papers).
        scorer: TopicScorer instance.
        p1_config: Project1 Config (for tier thresholds and projects).

    Returns:
        List of ScoredPaper from project1's scorer module.
    """
    if not articles:
        return []

    return scorer.score_papers(articles, p1_config)

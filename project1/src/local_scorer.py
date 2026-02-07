"""Integration bridge: connects song_db local ranker to the main pipeline.

Provides lazy loading (model + embedder loaded once, reused) and
conversion between ArxivPaper and PaperRecord types.
"""

import sys
from pathlib import Path
from typing import Optional

# Add song_db parent to path so imports work from src/
_project_root = Path(__file__).parent.parent
if str(_project_root) not in sys.path:
    sys.path.insert(0, str(_project_root))

from song_db.models import LocalScore, PaperRecord

# Singleton state
_ranker_instance = None


def arxiv_paper_to_record(paper) -> PaperRecord:
    """Convert an ArxivPaper to a PaperRecord for local scoring.

    Handles the type mismatch between ArxivPaper (datetime fields)
    and PaperRecord (string fields).
    """
    published = ""
    if paper.published:
        try:
            published = paper.published.isoformat()
        except AttributeError:
            published = str(paper.published)

    updated = ""
    if paper.updated:
        try:
            updated = paper.updated.isoformat()
        except AttributeError:
            updated = str(paper.updated)

    return PaperRecord(
        arxiv_id=paper.arxiv_id,
        title=paper.title,
        abstract=paper.abstract,
        categories=paper.categories,
        primary_category=paper.primary_category,
        published=published,
        updated=updated,
    )


def load_local_ranker(interest_model_path: str | Path):
    """Load the local ranker singleton.

    Lazy-imports sentence_transformers and song_db modules only when called.
    Returns the ranker, or raises if model file not found.
    """
    global _ranker_instance

    if _ranker_instance is not None:
        return _ranker_instance

    from song_db.distill import load_interest_model
    from song_db.embeddings import Embedder
    from song_db.rank import LocalRanker

    model_path = Path(interest_model_path)
    if not model_path.exists():
        raise FileNotFoundError(f"Interest model not found: {model_path}")

    model = load_interest_model(model_path)
    embedder = Embedder(model.model_id)
    _ranker_instance = LocalRanker(model, embedder)

    return _ranker_instance


def local_score_paper(paper, ranker=None, interest_model_path: Optional[str] = None) -> LocalScore:
    """Score a single ArxivPaper using the local ranker."""
    if ranker is None:
        if interest_model_path is None:
            raise ValueError("Either ranker or interest_model_path must be provided")
        ranker = load_local_ranker(interest_model_path)

    record = arxiv_paper_to_record(paper)
    scores = ranker.score_papers([record])
    return scores[0]


def local_score_papers(papers: list, ranker=None, interest_model_path: Optional[str] = None) -> list[LocalScore]:
    """Score a batch of ArxivPaper objects using the local ranker."""
    if ranker is None:
        if interest_model_path is None:
            raise ValueError("Either ranker or interest_model_path must be provided")
        ranker = load_local_ranker(interest_model_path)

    records = [arxiv_paper_to_record(p) for p in papers]
    return ranker.score_papers(records)


def local_score_to_llm_scale(local_score: float) -> float:
    """Map a local score from [0, 1] range to [0, 10] LLM scale.

    The mapping is non-linear to align with existing tier thresholds:
      - local >= 0.7 → ~8-10 (Most Relevant)
      - local 0.5-0.7 → ~5-8 (Somewhat Relevant)
      - local 0.3-0.5 → ~3-5 (Could Be Interesting)
      - local < 0.3 → ~0-3 (Not Relevant)
    """
    clamped = max(0.0, min(1.0, local_score))
    return clamped * 10.0

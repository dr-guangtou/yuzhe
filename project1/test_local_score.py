#!/usr/bin/env python3
"""Test the full local scoring pipeline (Stage 1 + Stage 2) for arXiv papers.

Shows both the corpus filter score (Stage 1) and the topic-embedding score
(Stage 2) side by side, with matched topics, cosine similarities, and tier
assignment. Useful for calibrating breakpoints in config.yaml.

Usage:
    uv run python test_local_score.py 2602.06904,2602.06439
    uv run python test_local_score.py 2602.06904 2602.06439 2602.06119
    uv run python test_local_score.py https://arxiv.org/abs/2602.06904
    uv run python test_local_score.py 2602.06904 --config config.yaml
"""

import argparse
import re
import sys
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent / "src"))


def normalize_arxiv_id(raw: str) -> str:
    """Normalize arxiv_id by stripping prefixes and version suffix."""
    if "/abs/" in raw:
        raw = raw.split("/abs/")[-1]
    if raw.lower().startswith("arxiv:"):
        raw = raw[6:]
    raw = re.sub(r"v\d+$", "", raw)
    return raw.strip()


def parse_ids(args_ids: list[str]) -> list[str]:
    """Parse positional args into a flat list of arXiv IDs.

    Supports both comma-separated ("2602.001,2602.002") and
    space-separated ("2602.001 2602.002") input.
    """
    ids = []
    for arg in args_ids:
        for part in arg.split(","):
            part = part.strip()
            if part:
                ids.append(normalize_arxiv_id(part))
    return ids


def fetch_papers_from_arxiv(arxiv_ids: list[str]) -> list[dict]:
    """Fetch paper metadata from arXiv API for a batch of IDs.

    Returns list of dicts with arxiv_id, title, abstract, categories,
    primary_category. Papers that fail to fetch are silently skipped.
    """
    base_url = "https://export.arxiv.org/api/query"
    params = {"id_list": ",".join(arxiv_ids), "max_results": len(arxiv_ids)}
    url = f"{base_url}?{urllib.parse.urlencode(params)}"

    req = urllib.request.Request(
        url,
        headers={"User-Agent": "local-score-test/1.0 (academic research tool)"},
    )

    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            xml_data = response.read()
    except Exception as e:
        print(f"Error fetching from arXiv: {e}", file=sys.stderr)
        return []

    ns = "http://www.w3.org/2005/Atom"
    arxiv_ns = "http://arxiv.org/schemas/atom"
    root = ET.fromstring(xml_data)

    papers = []
    for entry in root.findall(f"{{{ns}}}entry"):
        id_elem = entry.find(f"{{{ns}}}id")
        title_elem = entry.find(f"{{{ns}}}title")
        summary_elem = entry.find(f"{{{ns}}}summary")

        if id_elem is None or title_elem is None or summary_elem is None:
            continue

        fetched_id = id_elem.text.strip()
        if "/abs/" in fetched_id:
            fetched_id = fetched_id.split("/abs/")[-1]
        fetched_id = re.sub(r"v\d+$", "", fetched_id)

        title = " ".join(title_elem.text.split())
        abstract = " ".join(summary_elem.text.split())

        # Check for "Error" responses (arXiv returns an entry with error text)
        if title.lower().startswith("error"):
            print(f"  Warning: arXiv returned error for {fetched_id}: {title}", file=sys.stderr)
            continue

        categories = []
        for cat_elem in entry.findall(f"{{{ns}}}category"):
            term = cat_elem.get("term")
            if term:
                categories.append(term)

        primary_cat_elem = entry.find(f"{{{arxiv_ns}}}primary_category")
        primary_category = primary_cat_elem.get("term", "") if primary_cat_elem is not None else ""

        papers.append({
            "arxiv_id": fetched_id,
            "title": title,
            "abstract": abstract,
            "categories": categories,
            "primary_category": primary_category,
        })

    return papers


def print_paper_results(paper, corpus_score, topic_result, config, tier):
    """Pretty-print scoring results for a single paper."""
    ts_config = config.topic_scorer
    thresholds = config.llm_scoring.tier_thresholds

    print("\n" + "=" * 80)
    print(f"arXiv: {paper['arxiv_id']}  |  {paper['primary_category']}")
    print(f"Title: {paper['title']}")
    print("=" * 80)

    # Abstract (truncated)
    abstract = paper["abstract"]
    if len(abstract) > 250:
        abstract = abstract[:250] + "..."
    print(f"\n{abstract}\n")

    # Stage 1: Corpus filter
    print("-" * 40 + " STAGE 1: Corpus Filter " + "-" * 16)
    print(f"  Total score:  {corpus_score.score_total:.4f}  (threshold: {config.local_filter.threshold})")
    print(f"    Global sim: {corpus_score.score_global:.4f}  (w={config.local_filter.weights['w_global']:.2f})")
    print(f"    Topic sim:  {corpus_score.score_topic_max:.4f}  (w={config.local_filter.weights['w_topic']:.2f})")
    print(f"    Category:   {corpus_score.score_category:.4f}  (w={config.local_filter.weights['w_category']:.2f})")
    passed = corpus_score.score_total >= config.local_filter.threshold
    print(f"  Result: {'PASS' if passed else 'FILTERED OUT'}")

    # Stage 2: Topic-embedding scorer
    print("-" * 40 + " STAGE 2: Topic Score " + "-" * 18)
    print(f"  Final score:  {topic_result.score:.1f}/10  ->  Tier: {tier.value}")
    print()

    # Primary topic breakdown
    print(f"  Best primary cosine sim:   {topic_result.best_primary_sim:.4f}")
    from topic_scorer import piecewise_map
    primary_mapped = piecewise_map(topic_result.best_primary_sim, ts_config.primary_breakpoints)
    print(f"    -> mapped score: {primary_mapped:.1f}/10")

    # Secondary topic breakdown
    print(f"  Best secondary cosine sim: {topic_result.best_secondary_sim:.4f}")
    secondary_mapped = piecewise_map(topic_result.best_secondary_sim, ts_config.secondary_breakpoints)
    print(f"    -> mapped score: {secondary_mapped:.1f}/10")

    # Negative topic breakdown
    print(f"  Best negative cosine sim:  {topic_result.best_negative_sim:.4f}")
    if topic_result.negative_penalty > 0:
        print(f"  Negative penalty: -{topic_result.negative_penalty:.1f}")

    # Multi-match
    if topic_result.n_primary_matches >= 2:
        bonus = min(1.0, ts_config.multi_match_bonus * (topic_result.n_primary_matches - 1))
        print(f"  Multi-match bonus: +{bonus:.1f} ({topic_result.n_primary_matches} primary topics matched)")

    # Project match
    if topic_result.matched_projects:
        print(f"  Project match: {', '.join(topic_result.matched_projects)}")

    # Matched negative topics
    if topic_result.matched_negative:
        print(f"\n  Matched NEGATIVE topics ({len(topic_result.matched_negative)}):")
        for t in topic_result.matched_negative:
            display = t if len(t) <= 70 else t[:67] + "..."
            print(f"    ! {display}")

    # Matched positive topics
    if topic_result.matched_topics:
        print(f"\n  Matched positive topics ({len(topic_result.matched_topics)}):")
        for t in topic_result.matched_topics[:8]:
            display = t if len(t) <= 70 else t[:67] + "..."
            print(f"    + {display}")
        if len(topic_result.matched_topics) > 8:
            print(f"    ... and {len(topic_result.matched_topics) - 8} more")
    else:
        print(f"\n  No positive topics matched (threshold: {ts_config.match_threshold})")

    # Tier thresholds reference
    print(f"\n  Tier thresholds: most_relevant >= {thresholds.most_relevant}, "
          f"somewhat >= {thresholds.somewhat_relevant}, "
          f"could_be >= {thresholds.could_be_interesting}")


def main():
    parser = argparse.ArgumentParser(
        description="Test local scoring pipeline (Stage 1 + Stage 2) for arXiv papers",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  uv run python test_local_score.py 2602.06904,2602.06439
  uv run python test_local_score.py 2602.06904 2602.06439 2602.06119
  uv run python test_local_score.py https://arxiv.org/abs/2602.06904
  uv run python test_local_score.py 2602.06904 --config config.yaml
        """,
    )
    parser.add_argument(
        "ids",
        nargs="+",
        help="arXiv IDs: comma-separated or space-separated (e.g., 2602.06904,2602.06439)",
    )
    parser.add_argument(
        "--config",
        default="config.yaml",
        help="Path to config.yaml (default: config.yaml)",
    )
    parser.add_argument(
        "--interest",
        default=None,
        help="Path to interest model (default: from config)",
    )

    args = parser.parse_args()

    # Parse IDs
    arxiv_ids = parse_ids(args.ids)
    if not arxiv_ids:
        print("Error: No valid arXiv IDs provided", file=sys.stderr)
        return 1

    print(f"Papers to score: {', '.join(arxiv_ids)}")

    # Load config
    config_path = Path(args.config)
    if not config_path.exists():
        print(f"Error: Config not found: {config_path}", file=sys.stderr)
        return 1

    from config import load_config
    config = load_config(config_path)

    # Check interest model exists
    interest_path = Path(args.interest) if args.interest else Path(config.local_filter.interest_model)
    if not interest_path.exists():
        print(f"Error: Interest model not found at {interest_path}", file=sys.stderr)
        return 1

    # Fetch papers from arXiv
    print(f"Fetching {len(arxiv_ids)} paper(s) from arXiv API...")
    papers = fetch_papers_from_arxiv(arxiv_ids)
    if not papers:
        print("Error: Could not fetch any papers from arXiv", file=sys.stderr)
        return 1

    fetched_ids = {p["arxiv_id"] for p in papers}
    missing = set(arxiv_ids) - fetched_ids
    if missing:
        print(f"Warning: Could not fetch: {', '.join(missing)}", file=sys.stderr)

    # Load models (once)
    print("Loading interest model and sentence-transformer...")
    from song_db.distill import load_interest_model
    from song_db.embeddings import Embedder
    from song_db.models import PaperRecord
    from song_db.rank import LocalRanker
    from topic_scorer import TopicScorer
    from scorer import assign_tier

    model = load_interest_model(interest_path)
    embedder = Embedder(model.model_id)
    ranker = LocalRanker(model, embedder)

    print("Embedding topic descriptions...")
    topic_scorer = TopicScorer(config, embedder)
    print(f"  {len(config.topics.primary)} primary + {len(config.topics.secondary)} secondary topics\n")

    from types import SimpleNamespace

    # Score all papers (batch embed once, cache results)
    results = []  # list of (paper, corpus_score, topic_result, tier)

    for paper in papers:
        # Stage 1: Corpus filter score
        record = PaperRecord(
            arxiv_id=paper["arxiv_id"],
            title=paper["title"],
            abstract=paper["abstract"],
            categories=paper["categories"],
            primary_category=paper["primary_category"],
        )
        corpus_score = ranker.score_papers([record])[0]

        # Stage 2: Topic-embedding score
        text = f"{paper['title']}\n\n{paper['abstract']}"
        paper_embedding = embedder.embed_texts([text])[0]

        fake_paper = SimpleNamespace(
            arxiv_id=paper["arxiv_id"],
            title=paper["title"],
            abstract=paper["abstract"],
            categories=paper["categories"],
            primary_category=paper["primary_category"],
        )

        topic_result = topic_scorer._score_single(paper_embedding, fake_paper)
        project_match = bool(topic_result.matched_projects)
        tier = assign_tier(topic_result.score, project_match, config)

        results.append((paper, corpus_score, topic_result, tier))
        print_paper_results(paper, corpus_score, topic_result, config, tier)

    # Summary table for multiple papers (reuse cached results)
    if len(results) > 1:
        print("\n" + "=" * 80)
        print("SUMMARY TABLE")
        print("=" * 80)
        print(f"{'arXiv ID':<16} {'Corpus':>7} {'Topic':>6} {'Tier':<22} {'#Topics':>7}")
        print("-" * 66)

        for paper, cs, tr, t in results:
            print(f"{paper['arxiv_id']:<16} {cs.score_total:>7.4f} {tr.score:>6.1f} {t.value:<22} {len(tr.matched_topics):>7}")

    print()
    return 0


if __name__ == "__main__":
    sys.exit(main())

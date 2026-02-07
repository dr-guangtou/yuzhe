#!/usr/bin/env python3
"""
Test the local interest filter with a single arXiv paper.

Usage:
    uv run python test_local_filter.py --id 2602.00001
    uv run python test_local_filter.py --id 2301.12345 --interest song_db/artifacts/interest_model.json
"""

import argparse
import json
import re
import sys
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent / "src"))


def normalize_arxiv_id(arxiv_id: str) -> str:
    """Normalize arxiv_id by stripping prefixes and version suffix."""
    # Strip URL prefix
    if "/abs/" in arxiv_id:
        arxiv_id = arxiv_id.split("/abs/")[-1]
    # Strip arXiv: prefix
    if arxiv_id.lower().startswith("arxiv:"):
        arxiv_id = arxiv_id[6:]
    # Strip version suffix vN
    arxiv_id = re.sub(r"v\d+$", "", arxiv_id)
    return arxiv_id.strip()


def fetch_paper_from_arxiv(arxiv_id: str) -> dict | None:
    """Fetch paper metadata from arXiv API.

    Respects arXiv rate limits: single request, no burst.
    Returns dict with arxiv_id, title, abstract, categories, primary_category.
    """
    base_url = "https://export.arxiv.org/api/query"
    params = {"id_list": arxiv_id, "max_results": 1}
    url = f"{base_url}?{urllib.parse.urlencode(params)}"

    req = urllib.request.Request(url, headers={"User-Agent": "local-filter-test/1.0 (academic research tool)"})

    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            xml_data = response.read()
    except Exception as e:
        print(f"Error fetching from arXiv: {e}", file=sys.stderr)
        return None

    # Parse Atom feed
    root = ET.fromstring(xml_data)
    ns = "http://www.w3.org/2005/Atom"
    arxiv_ns = "http://arxiv.org/schemas/atom"

    entries = root.findall(f"{{{ns}}}entry")
    if not entries:
        return None

    entry = entries[0]

    # Extract fields
    id_elem = entry.find(f"{{{ns}}}id")
    title_elem = entry.find(f"{{{ns}}}title")
    summary_elem = entry.find(f"{{{ns}}}summary")

    if id_elem is None or title_elem is None or summary_elem is None:
        return None

    fetched_id = id_elem.text.strip()
    if "/abs/" in fetched_id:
        fetched_id = fetched_id.split("/abs/")[-1]
    fetched_id = re.sub(r"v\d+$", "", fetched_id)

    title = " ".join(title_elem.text.split())
    abstract = " ".join(summary_elem.text.split())

    # Categories
    categories = []
    for cat_elem in entry.findall(f"{{{ns}}}category"):
        term = cat_elem.get("term")
        if term:
            categories.append(term)

    # Primary category
    primary_cat_elem = entry.find(f"{{{arxiv_ns}}}primary_category")
    primary_category = primary_cat_elem.get("term", "") if primary_cat_elem is not None else ""

    return {
        "arxiv_id": fetched_id,
        "title": title,
        "abstract": abstract,
        "categories": categories,
        "primary_category": primary_category,
    }


def load_and_score(paper: dict, interest_model_path: Path) -> tuple:
    """Load interest model and score the paper.

    Returns: (local_score, local_score_obj, ranker)
    """
    from song_db.distill import load_interest_model
    from song_db.embeddings import Embedder
    from song_db.models import PaperRecord
    from song_db.rank import LocalRanker

    # Load model
    model = load_interest_model(interest_model_path)
    embedder = Embedder(model.model_id)
    ranker = LocalRanker(model, embedder)

    # Convert to PaperRecord
    record = PaperRecord(
        arxiv_id=paper["arxiv_id"],
        title=paper["title"],
        abstract=paper["abstract"],
        categories=paper["categories"],
        primary_category=paper["primary_category"],
        published="",
        updated="",
    )

    # Score (score_papers expects a list, returns a list)
    scores = ranker.score_papers([record])
    local_score_obj = scores[0]

    return local_score_obj.score_total, local_score_obj, ranker


def print_results(paper: dict, score: float, score_obj, ranker):
    """Pretty-print scoring results."""
    print("\n" + "=" * 80)
    print(f"arXiv ID: {paper['arxiv_id']}")
    print("=" * 80)
    print(f"\nTitle: {paper['title']}")
    print(f"\nPrimary Category: {paper['primary_category']}")
    print(f"All Categories: {', '.join(paper['categories'])}")
    print(f"\nAbstract:\n{paper['abstract'][:300]}{'...' if len(paper['abstract']) > 300 else ''}")

    print("\n" + "-" * 80)
    print("LOCAL FILTER SCORE")
    print("-" * 80)
    print(f"TOTAL SCORE: {score:.4f}  [0.0 = not relevant, 1.0 = highly relevant]")
    print()
    print(f"  Global similarity:  {score_obj.score_global:.4f}  (weight: {ranker.model.scoring_weights['w_global']:.2f})")
    print(f"  Topic similarity:   {score_obj.score_topic_max:.4f}  (weight: {ranker.model.scoring_weights['w_topic']:.2f})")
    print(f"  Category prior:     {score_obj.score_category:.4f}  (weight: {ranker.model.scoring_weights['w_category']:.2f})")

    # Best topic
    print(f"\nBest matching topic: #{score_obj.best_topic_id}")
    topic = ranker.model.topic_clusters[score_obj.best_topic_id]
    keywords = ", ".join(topic.top_keywords[:10])
    print(f"  Keywords: {keywords}")
    print(f"  Cluster size: {topic.size} papers")

    # Top 3 topics
    if score_obj.topic_scores_top3:
        print(f"\nTop 3 topic matches:")
        for topic_id, topic_score in score_obj.topic_scores_top3:
            topic = ranker.model.topic_clusters[topic_id]
            kw_short = ", ".join(topic.top_keywords[:5])
            print(f"  #{topic_id}: {topic_score:.4f} - {kw_short}")

    # Interpretation
    print("\n" + "-" * 80)
    print("INTERPRETATION")
    print("-" * 80)
    if score >= 0.7:
        print("✓ Highly relevant - likely matches your research interests")
    elif score >= 0.5:
        print("~ Moderately relevant - worth reviewing")
    elif score >= 0.3:
        print("? Possibly relevant - marginal interest")
    else:
        print("✗ Low relevance - likely outside your research area")

    # Integration mode recommendations
    print("\n" + "-" * 80)
    print("INTEGRATION MODE BEHAVIOR")
    print("-" * 80)

    threshold = 0.3  # default from config
    llm_scale = score * 10  # [0,1] -> [0,10]

    print(f"Skip-LLM + Local Filter mode:")
    print(f"  → Would assign score: {llm_scale:.1f}/10")

    if score >= threshold:
        print(f"\nHybrid mode (threshold={threshold}):")
        print(f"  → Would send to LLM for full scoring")
    else:
        print(f"\nHybrid mode (threshold={threshold}):")
        print(f"  → Would skip LLM (below threshold), use embedding score only")

    print("\n" + "=" * 80 + "\n")


def main():
    parser = argparse.ArgumentParser(
        description="Test local interest filter with a single arXiv paper",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  uv run python test_local_filter.py --id 2602.00001
  uv run python test_local_filter.py --id arxiv:2301.12345
  uv run python test_local_filter.py --id https://arxiv.org/abs/2401.00123
        """,
    )
    parser.add_argument(
        "--id",
        required=True,
        help="arXiv identifier (e.g., 2602.00001, arxiv:2301.12345, or full URL)",
    )
    parser.add_argument(
        "--interest",
        default="song_db/artifacts/interest_model.json",
        help="Path to interest model (default: song_db/artifacts/interest_model.json)",
    )

    args = parser.parse_args()

    # Normalize arxiv_id
    arxiv_id = normalize_arxiv_id(args.id)
    print(f"Normalized arXiv ID: {arxiv_id}")

    # Check interest model exists
    interest_path = Path(args.interest)
    if not interest_path.exists():
        print(f"Error: Interest model not found at {interest_path}", file=sys.stderr)
        print(f"\nRun the following to create it:", file=sys.stderr)
        print(f"  uv run python -m song_db distill --corpus song_db/artifacts/corpus_clean.jsonl --embeddings song_db/artifacts/embeddings.npy --ids song_db/artifacts/ids.json --output {interest_path}", file=sys.stderr)
        return 1

    # Fetch paper
    print(f"Fetching paper from arXiv API...")
    paper = fetch_paper_from_arxiv(arxiv_id)
    if paper is None:
        print(f"Error: Could not fetch paper {arxiv_id} from arXiv", file=sys.stderr)
        print(f"Check that the arXiv ID is valid and the paper exists.", file=sys.stderr)
        return 1

    # Score paper
    print(f"Loading interest model and computing score...")
    try:
        score, score_obj, ranker = load_and_score(paper, interest_path)
    except Exception as e:
        print(f"Error scoring paper: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 1

    # Display results
    print_results(paper, score, score_obj, ranker)

    return 0


if __name__ == "__main__":
    sys.exit(main())

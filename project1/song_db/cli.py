"""CLI entry points for the song_db pipeline.

Usage:
    uv run python -m song_db <command> [options]
"""

import argparse
import json
from pathlib import Path


def cmd_ingest(args: argparse.Namespace) -> None:
    """Ingest and clean the corpus JSONL."""
    from .corpus_ingest import ingest_corpus

    input_path = Path(args.input)
    output_path = Path(args.out)
    ingest_corpus(input_path, output_path)


def cmd_embed(args: argparse.Namespace) -> None:
    """Compute embeddings for the cleaned corpus."""
    from .corpus_ingest import load_clean_corpus
    from .embeddings import compute_and_save_embeddings

    corpus_path = Path(args.corpus)
    output_dir = Path(args.out)
    model_name = args.model

    records = load_clean_corpus(corpus_path)
    print(f"Loaded {len(records)} records from {corpus_path}")

    compute_and_save_embeddings(
        records, output_dir,
        model_name=model_name,
        batch_size=args.batch_size,
    )


def cmd_distill(args: argparse.Namespace) -> None:
    """Distill the interest model from corpus + embeddings."""
    from .corpus_ingest import load_clean_corpus
    from .distill import distill_interest_model
    from .embeddings import load_embeddings

    corpus_path = Path(args.corpus)
    embeddings_dir = Path(args.embeddings)
    output_path = Path(args.out)
    k_topics = args.k_topics

    records = load_clean_corpus(corpus_path)
    embeddings, ids = load_embeddings(embeddings_dir)

    print(f"Corpus: {len(records)} records, Embeddings: {embeddings.shape}")

    distill_interest_model(
        records, embeddings, ids,
        k_topics=k_topics,
        output_path=output_path,
    )

    # Print topic summaries for interpretability
    from .distill import load_interest_model
    model = load_interest_model(output_path)
    print(f"\nTopic cluster summary ({len(model.topic_clusters)} clusters):")
    for tc in model.topic_clusters:
        keywords = ", ".join(tc.top_keywords[:8])
        print(f"  Topic {tc.topic_id} (n={tc.size}): {keywords}")


def cmd_rank(args: argparse.Namespace) -> None:
    """Rank candidate papers using the interest model."""
    from .corpus_ingest import load_clean_corpus
    from .distill import load_interest_model
    from .embeddings import Embedder
    from .rank import LocalRanker

    model = load_interest_model(Path(args.interest))
    embedder = Embedder(model.model_id)
    ranker = LocalRanker(model, embedder)

    # Load candidates
    candidates_path = Path(args.candidates)
    candidates = load_clean_corpus(candidates_path)
    print(f"Loaded {len(candidates)} candidate papers")

    # Score
    scores = ranker.score_papers(candidates)

    # Pair and sort
    paired = sorted(zip(candidates, scores), key=lambda x: -x[1].score_total)

    # Apply topk
    if args.topk and args.topk < len(paired):
        paired = paired[:args.topk]

    # Output
    output_path = Path(args.out) if args.out else None
    results = []
    for paper, score in paired:
        result = {
            "arxiv_id": paper.arxiv_id,
            "title": paper.title,
            "score_total": round(score.score_total, 4),
            "score_global": round(score.score_global, 4),
            "score_topic_max": round(score.score_topic_max, 4),
            "best_topic_id": score.best_topic_id,
            "score_category": round(score.score_category, 4),
            "categories": paper.categories,
            "primary_category": paper.primary_category,
        }
        results.append(result)

    if output_path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:
            for result in results:
                f.write(json.dumps(result, ensure_ascii=False) + "\n")
        print(f"Ranked {len(results)} papers saved to {output_path}")
    else:
        for i, result in enumerate(results[:20], 1):
            print(f"  {i:3d}. [{result['score_total']:.3f}] {result['arxiv_id']}: {result['title'][:70]}")
        if len(results) > 20:
            print(f"  ... and {len(results) - 20} more")


def cmd_eval(args: argparse.Namespace) -> None:
    """Run evaluation harness."""
    from .corpus_ingest import load_clean_corpus
    from .distill import load_interest_model
    from .embeddings import Embedder
    from .eval import run_evaluation

    model = load_interest_model(Path(args.interest))
    embedder = Embedder(model.model_id)

    corpus_path = Path(args.corpus)
    records = load_clean_corpus(corpus_path)
    print(f"Loaded {len(records)} corpus records")

    output_path = Path(args.out) if args.out else Path("song_db/artifacts/eval_report.json")
    negatives_cache = Path(args.negatives_cache) if args.negatives_cache else Path("song_db/artifacts/negatives.jsonl")

    run_evaluation(
        model=model,
        embedder=embedder,
        records=records,
        output_path=output_path,
        n_positives=args.n_positives,
        seed=args.seed,
        negatives_cache=negatives_cache,
    )


def build_parser() -> argparse.ArgumentParser:
    """Build the CLI argument parser."""
    parser = argparse.ArgumentParser(
        prog="song_db",
        description="Local interest model pipeline for arXiv paper filtering",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # ingest
    p_ingest = subparsers.add_parser("ingest", help="Ingest and clean corpus JSONL")
    p_ingest.add_argument("--input", required=True, help="Path to raw corpus JSONL")
    p_ingest.add_argument("--out", required=True, help="Path for cleaned output JSONL")

    # embed
    p_embed = subparsers.add_parser("embed", help="Compute corpus embeddings")
    p_embed.add_argument("--corpus", required=True, help="Path to cleaned corpus JSONL")
    p_embed.add_argument("--out", required=True, help="Output directory for embeddings")
    p_embed.add_argument("--model", default="sentence-transformers/all-MiniLM-L6-v2",
                         help="Sentence-transformer model name")
    p_embed.add_argument("--batch-size", type=int, default=128, help="Embedding batch size")

    # distill
    p_distill = subparsers.add_parser("distill", help="Distill interest model")
    p_distill.add_argument("--corpus", required=True, help="Path to cleaned corpus JSONL")
    p_distill.add_argument("--embeddings", required=True, help="Directory with embeddings.npy and ids.json")
    p_distill.add_argument("--k-topics", type=int, default=12, help="Number of topic clusters")
    p_distill.add_argument("--out", required=True, help="Output path for interest_model.json")

    # rank
    p_rank = subparsers.add_parser("rank", help="Rank candidate papers")
    p_rank.add_argument("--interest", required=True, help="Path to interest_model.json")
    p_rank.add_argument("--candidates", required=True, help="Path to candidates JSONL")
    p_rank.add_argument("--topk", type=int, default=None, help="Output only top-K papers")
    p_rank.add_argument("--out", default=None, help="Output path for ranked JSONL")

    # eval
    p_eval = subparsers.add_parser("eval", help="Run evaluation harness")
    p_eval.add_argument("--interest", required=True, help="Path to interest_model.json")
    p_eval.add_argument("--corpus", required=True, help="Path to cleaned corpus JSONL")
    p_eval.add_argument("--out", default=None, help="Output path for eval_report.json")
    p_eval.add_argument("--n-positives", type=int, default=200, help="Number of positive samples")
    p_eval.add_argument("--seed", type=int, default=42, help="Random seed")
    p_eval.add_argument("--negatives-cache", default=None, help="Path to cached negatives JSONL")

    return parser


def main() -> None:
    """Main CLI entry point."""
    parser = build_parser()
    args = parser.parse_args()

    commands = {
        "ingest": cmd_ingest,
        "embed": cmd_embed,
        "distill": cmd_distill,
        "rank": cmd_rank,
        "eval": cmd_eval,
    }

    cmd_func = commands[args.command]
    cmd_func(args)

#!/usr/bin/env python3
"""Daily arXiv Summary - CLI Entry Point.

Pipeline stages:
1. Local Filter (MANDATORY) - embedding-based filtering
2. LLM Scoring (OPTIONAL) - re-score with LLM API
3. Summary Generation (OPTIONAL) - generate summaries or use abstracts
"""

import argparse
import re
import sys
from datetime import datetime, timedelta
from pathlib import Path

from config import load_config, get_default_config_path
from arxiv_fetcher import fetch_papers, fetch_papers_from_rss
from llm_client import create_fallback_client, MockLLMClient
from scorer import score_papers, group_by_tier, Tier
from summarizer import generate_summaries
from formatter import format_digest, save_digest, copy_to_obsidian
from state import StateManager, get_default_state_path
from logger import setup_logging


def parse_digest_date_from_path(path: Path) -> datetime | None:
    """Parse digest date from filename stem.

    Supports both legacy and new naming:
    - YYYY-MM-DD.md
    - arxiv-YYYY-MM-DD.md
    """
    stem = path.stem
    for fmt in ("%Y-%m-%d", "arxiv-%Y-%m-%d"):
        try:
            return datetime.strptime(stem, fmt)
        except ValueError:
            continue
    return None


def get_default_digest_history_root(config) -> Path:
    """Return the default digest history root from config."""
    if config.config_path:
        base_dir = config.config_path.parent
    else:
        base_dir = Path(".")

    return base_dir / config.output.digest_dir / config.output.archive_subdir


def resolve_digest_history_root(
    config,
    output_path: Path | None = None,
    output_dir: Path | None = None,
) -> Path:
    """Resolve which directory should be scanned for previous digest files."""
    if output_dir is not None:
        return output_dir

    if output_path is not None:
        return output_path.parent

    return get_default_digest_history_root(config)


def get_dated_digest_files(
    config,
    history_root: Path | None = None,
) -> list[tuple[datetime, float, Path]]:
    """Collect digest files with parsed dates from the selected history root."""
    scan_root = history_root if history_root is not None else get_default_digest_history_root(config)
    if not scan_root.exists():
        return []

    dated_files: list[tuple[datetime, float, Path]] = []
    for digest_file in scan_root.rglob("*.md"):
        parsed_date = parse_digest_date_from_path(digest_file)
        if parsed_date is None:
            continue
        dated_files.append((parsed_date, digest_file.stat().st_mtime, digest_file))

    return dated_files


def get_latest_digest_date(
    config,
    history_root: Path | None = None,
) -> datetime | None:
    """Get the date of the most recent digest file.

    Returns:
        datetime of latest digest, or None if no digests exist
    """
    dated_files = get_dated_digest_files(config, history_root=history_root)
    if not dated_files:
        return None

    latest_date, _, _ = max(dated_files, key=lambda item: (item[0], item[1]))
    return latest_date


def get_previous_digest_ids(
    config,
    since_date: datetime | None = None,
    history_root: Path | None = None,
) -> tuple[set[str], int]:
    """Extract arXiv IDs from existing digest files.

    Reads digest Markdown files and extracts arXiv IDs from links.
    Supports deduping against a date window by using `since_date`.

    Returns:
        Tuple of (set_of_ids, number_of_files_scanned).
    """
    dated_files = get_dated_digest_files(config, history_root=history_root)
    if since_date is not None:
        dated_files = [item for item in dated_files if item[0] >= since_date]

    ids: set[str] = set()
    for _, _, digest_file in dated_files:
        content = digest_file.read_text(encoding="utf-8")
        ids.update(re.findall(r"arxiv\.org/abs/(\d{4}\.\d{4,5})", content))

    return ids, len(dated_files)


def get_dedup_since_date(dedup_days: int, now: datetime | None = None) -> datetime:
    """Build an inclusive dedup cutoff at midnight local time."""
    if dedup_days < 0:
        raise ValueError("--dedup-days must be >= 0")

    reference = now if now is not None else datetime.now()
    midnight_today = datetime(reference.year, reference.month, reference.day)
    return midnight_today - timedelta(days=dedup_days)


def count_papers_by_primary_category(
    papers: list,
    categories: list[str],
) -> dict[str, int]:
    """Count papers by primary category, preserving configured category order."""
    counts = {category: 0 for category in categories}
    for paper in papers:
        primary_category = paper.primary_category
        if primary_category in counts:
            counts[primary_category] += 1
        else:
            counts[primary_category] = counts.get(primary_category, 0) + 1
    return counts


def split_duplicate_papers(
    papers: list,
    previous_ids: set[str],
) -> tuple[list, list]:
    """Split fetched papers into fresh and duplicate partitions."""
    fresh_papers = []
    duplicate_papers = []
    for paper in papers:
        if paper.arxiv_id in previous_ids:
            duplicate_papers.append(paper)
        else:
            fresh_papers.append(paper)
    return fresh_papers, duplicate_papers


def build_dated_output_path(output_dir: Path, date: datetime) -> Path:
    """Build default digest filename in a user-specified directory."""
    return output_dir / f"arxiv-{date.strftime('%Y-%m-%d')}.md"


def main():
    parser = argparse.ArgumentParser(
        description="Daily arXiv Summary - Monitor arXiv for relevant papers",
        epilog="""
Pipeline stages:
  1. Local Filter (MANDATORY)  - embedding-based filtering
  2. LLM Scoring (OPTIONAL)    - re-score with LLM API
  3. Summary Generation (OPT.) - generate summaries or use abstracts

Modes:
  Default: Stage 1 + Stage 3 (local filter + summaries), update mode
  --use-llm-scoring: Add Stage 2 (LLM scoring)
  --debug: Force run without update check
  --no-summary: Skip Stage 3, output abstracts only
        """
    )

    # Mode flags
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Force run without update check (disable update mode)"
    )

    # Stage 2: LLM Scoring (optional)
    parser.add_argument(
        "--use-llm-scoring",
        action="store_true",
        help="Enable Stage 2: LLM-based scoring (default: OFF to save tokens)"
    )

    # Stage 3: Summary Generation (optional)
    parser.add_argument(
        "--no-summary",
        action="store_true",
        help="Disable Stage 3: skip summary generation, output abstracts only"
    )

    # Configuration
    parser.add_argument(
        "--config",
        type=Path,
        default=None,
        help="Path to config.yaml (default: project1/config.yaml)"
    )

    # Fetching options
    parser.add_argument(
        "--category",
        type=str,
        default=None,
        help="Override: fetch only from this category"
    )
    parser.add_argument(
        "--days",
        type=int,
        default=None,
        help="Number of days to look back (default: 1, overrides digest-based cutoff)"
    )
    parser.add_argument(
        "--dedup-days",
        type=int,
        default=2,
        help="Scan only the last N days of digest files for dedup (default: 2)"
    )
    parser.add_argument(
        "--no-dedup",
        action="store_true",
        help="Disable digest-history deduplication"
    )
    parser.add_argument(
        "--max-papers",
        type=int,
        default=None,
        help="Override: max papers per category"
    )
    parser.add_argument(
        "--source",
        choices=["rss", "api"],
        default="rss",
        help="Paper source: rss (announcement-date, default) or api (submission-date)"
    )

    # Local filter (Stage 1) options
    parser.add_argument(
        "--local-filter-threshold",
        type=float,
        default=None,
        help="Override local filter threshold (0-1 scale, default from config: 0.5)"
    )
    parser.add_argument(
        "--interest-model",
        type=Path,
        default=None,
        help="Path to interest_model.json (default: song_db/artifacts/interest_model.json)"
    )

    # Testing/debugging options
    parser.add_argument(
        "--skip-scoring",
        action="store_true",
        help="Skip all stages after fetching (fetch only)"
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=5,
        help="Number of papers per LLM scoring call (default: 5, only with --use-llm-scoring)"
    )
    parser.add_argument(
        "--llm-call-gap-seconds",
        type=float,
        default=5.0,
        help="Delay between consecutive LLM API calls in scoring/summary stages (default: 5.0)"
    )
    parser.add_argument(
        "--mock-llm",
        action="store_true",
        help="Use mock LLM client for testing"
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Limit number of papers to process (for testing)"
    )
    output_group = parser.add_mutually_exclusive_group()
    output_group.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Write digest to an explicit file path"
    )
    output_group.add_argument(
        "--output-dir",
        "--dir",
        dest="output_dir",
        type=Path,
        default=None,
        help="Write digest to this directory with default filename (arxiv-YYYY-MM-DD.md)"
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Enable verbose output"
    )
    parser.add_argument(
        "--no-log-file",
        action="store_true",
        help="Disable file logging"
    )

    # Deprecated flags (backward compatibility)
    parser.add_argument(
        "--skip-llm",
        action="store_true",
        help="(DEPRECATED) Use default mode instead (local filter only)"
    )
    parser.add_argument(
        "--use-local-filter",
        action="store_true",
        help="(DEPRECATED) Local filter is now always enabled"
    )
    parser.add_argument(
        "--skip-summary",
        action="store_true",
        help="(DEPRECATED) Use --no-summary instead"
    )
    parser.add_argument(
        "--mode",
        choices=["update", "debug"],
        default=None,
        help="(DEPRECATED) Use --debug flag instead"
    )

    args = parser.parse_args()

    # Setup logging
    import logging
    log_level = logging.DEBUG if args.verbose else logging.INFO
    logger = setup_logging(
        console_level=log_level,
        log_to_file=not args.no_log_file
    )

    # Handle deprecated flags
    if args.skip_llm:
        logger.warning("--skip-llm is deprecated. Default mode now uses local filter without LLM scoring.")
        logger.warning("To enable LLM scoring, use --use-llm-scoring instead.")

    if args.use_local_filter:
        logger.warning("--use-local-filter is deprecated. Local filter is now always enabled.")

    if args.skip_summary:
        logger.warning("--skip-summary is deprecated. Use --no-summary instead.")
        args.no_summary = True

    if args.mode:
        logger.warning("--mode is deprecated. Use --debug flag to force run without update check.")
        if args.mode == "debug":
            args.debug = True

    # Determine update mode
    update_mode = not args.debug
    if update_mode:
        logger.info("Update mode: will skip if no new papers")
    else:
        logger.info("Debug mode: force run without update check")

    # Load configuration
    config_path = args.config if args.config else get_default_config_path()
    logger.info(f"Loading config from: {config_path}")

    try:
        config = load_config(config_path)
    except FileNotFoundError:
        logger.error(f"Config file not found: {config_path}")
        sys.exit(1)
    except ValueError as e:
        logger.error(f"Invalid config: {e}")
        sys.exit(1)

    # Apply CLI overrides to config
    if args.use_llm_scoring:
        config.llm_scoring.enabled = True
        logger.info("Stage 2 (LLM Scoring): ENABLED via CLI flag")
    else:
        logger.info(f"Stage 2 (LLM Scoring): {'ENABLED' if config.llm_scoring.enabled else 'DISABLED'}")

    if args.no_summary:
        config.summary.enabled = False
        logger.info("Stage 3 (Summary Generation): DISABLED via CLI flag")
    else:
        logger.info(f"Stage 3 (Summary Generation): {'ENABLED' if config.summary.enabled else 'DISABLED'}")

    if args.local_filter_threshold is not None:
        config.local_filter.threshold = args.local_filter_threshold
        logger.info(f"Local filter threshold overridden: {args.local_filter_threshold}")

    logger.debug(f"Primary categories: {config.category.primary}")
    logger.debug(f"Secondary categories: {config.category.secondary}")

    digest_history_root = resolve_digest_history_root(
        config,
        output_path=args.output,
        output_dir=args.output_dir,
    )
    logger.info(f"Digest history root: {digest_history_root}")

    # Determine cutoff date from latest digest (all modes)
    since_date = None
    latest_digest_date = get_latest_digest_date(config, history_root=digest_history_root)

    if latest_digest_date:
        days_since_latest = (datetime.now() - latest_digest_date).days
        logger.info(f"Latest digest: {latest_digest_date.strftime('%Y-%m-%d')} ({days_since_latest} days ago)")

        if update_mode and days_since_latest < 1:
            logger.info(f"Skipping: latest digest is from today ({latest_digest_date.strftime('%Y-%m-%d')})")
            logger.info("Use --debug to force run")
            return

        since_date = latest_digest_date - timedelta(days=1)
        logger.info(f"Using cutoff date: {since_date.strftime('%Y-%m-%d')} (1-day overlap from digest {latest_digest_date.strftime('%Y-%m-%d')})")
    else:
        logger.info("No previous digest found - using --days lookback")

    # Explicit --days overrides digest-based cutoff
    if args.days is not None:
        since_date = None
        logger.info(f"--days {args.days} specified, overriding digest-based cutoff")

    # Default to 1 day when neither digest nor --days is available
    days = args.days if args.days is not None else 1
    if args.dedup_days < 0:
        logger.error("--dedup-days must be >= 0")
        sys.exit(1)
    if args.llm_call_gap_seconds < 0:
        logger.error("--llm-call-gap-seconds must be >= 0")
        sys.exit(1)

    # Determine categories to fetch
    if args.category:
        categories = [args.category]
    else:
        categories = config.category.all_categories()

    # Determine max papers
    max_papers = args.max_papers if args.max_papers else config.api.arxiv_max_results_per_category

    # Auto-switch to API when --days is specified (RSS only has today)
    source = args.source
    if args.days is not None and source == "rss":
        source = "api"
        logger.info("--days specified, auto-switching source from rss to api")

    logger.info(f"Fetching papers from {len(categories)} categories (source: {source})...")
    if source == "api":
        logger.info(f"Max papers per category: {max_papers}")
    if since_date:
        logger.info(f"Cutoff date: {since_date.strftime('%Y-%m-%d')}")
    else:
        logger.info(f"Days to look back: {days}")

    # Fetch papers
    if source == "rss":
        papers = fetch_papers_from_rss(categories=categories)
    else:
        papers = fetch_papers(
            categories=categories,
            max_results_per_category=max_papers,
            delay_seconds=config.api.arxiv_delay_seconds,
            days=days,
            since_date=since_date
        )

    logger.info(f"Total papers fetched: {len(papers)}")
    if args.debug:
        counts_before_dedup = count_papers_by_primary_category(papers, categories)
        print("\nDebug: fetched papers by primary category (before digest-history dedup)")
        for category in counts_before_dedup:
            print(f"  {category}: {counts_before_dedup[category]}")
        print(f"  TOTAL: {len(papers)}")

    # Deduplicate against recent digest files
    if args.no_dedup:
        logger.info("Digest-history dedup disabled (--no-dedup)")
        if args.debug:
            print("\nDebug: digest-history dedup skipped (--no-dedup)")
    else:
        dedup_since_date = get_dedup_since_date(args.dedup_days)
        previous_ids, digest_file_count = get_previous_digest_ids(
            config,
            since_date=dedup_since_date,
            history_root=digest_history_root,
        )
        papers, duplicate_papers = split_duplicate_papers(papers, previous_ids)
        if args.debug:
            print(
                "\nDebug: duplicates matched in digest-history dedup "
                f"(since {dedup_since_date.strftime('%Y-%m-%d')}, files={digest_file_count})"
            )
            if duplicate_papers:
                for paper in sorted(duplicate_papers, key=lambda p: p.arxiv_id):
                    print(f"  {paper.arxiv_id} [{paper.primary_category}] {paper.title}")
            else:
                print("  none")
        if previous_ids:
            logger.info(
                f"Dedup against {digest_file_count} digest file(s) since "
                f"{dedup_since_date.strftime('%Y-%m-%d')}: "
                f"{len(papers) + len(duplicate_papers)} -> {len(papers)} "
                f"({len(duplicate_papers)} already processed)"
            )
        else:
            logger.info(
                f"No prior IDs found in dedup window (since {dedup_since_date.strftime('%Y-%m-%d')}, "
                f"{digest_file_count} digest file(s) scanned)"
            )

    if not papers:
        logger.warning("No papers found.")
        return

    # Skip all processing if requested
    if args.skip_scoring:
        logger.info("All stages skipped (--skip-scoring)")
        for i, paper in enumerate(papers[:20], 1):
            print(f"{i}. [{paper.primary_category}] {paper.arxiv_id}: {paper.title[:60]}...")
        if len(papers) > 20:
            print(f"... and {len(papers) - 20} more papers")
        return

    # Limit papers for testing
    if args.limit:
        papers = papers[:args.limit]
        logger.info(f"Limited to {len(papers)} papers for processing")

    # ========================================================================
    # STAGE 1: Local Filter (MANDATORY)
    # ========================================================================
    logger.info("=" * 60)
    logger.info("STAGE 1: Local Filter (embedding-based)")
    logger.info("=" * 60)

    try:
        from local_scorer import load_local_ranker

        model_path = args.interest_model
        if model_path is None:
            model_path = Path(__file__).parent.parent / config.local_filter.interest_model

        logger.info(f"Loading interest model: {model_path}")
        local_ranker = load_local_ranker(model_path)

        threshold = config.local_filter.threshold
        logger.info(f"Local filter threshold: {threshold} (papers scoring < {threshold} filtered out)")

        # Score all papers with local filter
        from local_scorer import arxiv_paper_to_record
        records = [arxiv_paper_to_record(p) for p in papers]
        local_scores = local_ranker.score_papers(records)

        # Filter papers by local threshold
        papers_stage1 = []
        papers_stage1_rejected = []
        for paper, local_score in zip(papers, local_scores):
            if local_score.score_total >= threshold:
                papers_stage1.append(paper)
            else:
                papers_stage1_rejected.append(paper)

        logger.info("Local filter results:")
        logger.info(f"  Input: {len(papers)} papers")
        logger.info(f"  Passed threshold: {len(papers_stage1)} papers")
        logger.info(f"  Filtered out: {len(papers) - len(papers_stage1)} papers")
        if args.debug:
            print("\nDebug: rejected after local filter")
            if papers_stage1_rejected:
                for paper in papers_stage1_rejected:
                    print(f"  {paper.arxiv_id} [{paper.primary_category}] {paper.title}")
            else:
                print("  none")

        if not papers_stage1:
            logger.warning("No papers passed local filter threshold")
            return

        papers = papers_stage1  # Continue with filtered papers

    except Exception as e:
        logger.error(f"FATAL: Local filter failed: {e}")
        logger.error("Local filter is mandatory. Ensure song_db pipeline has been run.")
        logger.error("Run: uv run python -m song_db distill --help")
        sys.exit(1)

    # ========================================================================
    # STAGE 2: LLM Scoring (OPTIONAL)
    # ========================================================================
    scored_papers = None

    if config.llm_scoring.enabled:
        logger.info("=" * 60)
        logger.info("STAGE 2: LLM Scoring")
        logger.info("=" * 60)

        # Create LLM client
        if args.mock_llm:
            logger.info("Using mock LLM client")
            llm_client = MockLLMClient(config.llm)
        else:
            primary = config.llm.provider
            fallback = config.llm_fallback if config.llm_fallback else []
            chain = [primary] + [n for n in fallback if n != primary]
            logger.info(f"Initializing LLM clients: {chain}")

            try:
                llm_client = create_fallback_client(config)
                logger.info("LLM client ready")
            except Exception as e:
                logger.warning(f"Could not create any LLM client: {e}")
                logger.warning("Falling back to mock LLM client")
                llm_client = MockLLMClient(config.llm)

        # Score papers with LLM
        batch_sz = args.batch_size
        logger.info(f"Scoring {len(papers)} papers with LLM (batch_size={batch_sz})...")
        scored_papers = score_papers(
            papers=papers,
            config=config,
            llm_client=llm_client,
            skip_llm=False,
            batch_size=batch_sz,
            llm_call_gap_seconds=args.llm_call_gap_seconds,
        )

        # Keep summary tiers plus "Could Be Interesting" in the final digest.
        summary_tiers_str = config.llm_scoring.summary_tiers
        summary_tiers = [Tier(t) for t in summary_tiers_str]
        digest_keep_tiers = set(summary_tiers)
        digest_keep_tiers.add(Tier.COULD_BE_INTERESTING)

        papers_stage2 = []
        papers_stage2_rejected = []
        for sp in scored_papers:
            if sp.tier in digest_keep_tiers:
                papers_stage2.append(sp)
            else:
                papers_stage2_rejected.append(sp)

        logger.info("LLM scoring results:")
        logger.info(f"  Input: {len(scored_papers)} papers")
        logger.info(
            "  Digest tiers kept: "
            + ", ".join(t.value for t in sorted(digest_keep_tiers, key=lambda tier: tier.value))
        )
        logger.info(f"  Summary tiers: {', '.join(summary_tiers_str)}")
        logger.info(f"  Kept: {len(papers_stage2)} papers")
        logger.info(f"  Filtered out: {len(scored_papers) - len(papers_stage2)} papers")
        if args.debug:
            print("\nDebug: rejected after LLM digest filter")
            if papers_stage2_rejected:
                for sp in papers_stage2_rejected:
                    paper = sp.paper
                    print(f"  {paper.arxiv_id} [{paper.primary_category}] {paper.title}")
            else:
                print("  none")

        # Detailed tier breakdown
        groups = group_by_tier(scored_papers)
        logger.info("  Tier breakdown:")
        logger.info(f"    Most Relevant: {len(groups[Tier.MOST_RELEVANT])}")
        logger.info(f"    Somewhat Relevant: {len(groups[Tier.SOMEWHAT_RELEVANT])}")
        logger.info(f"    Could Be Interesting: {len(groups[Tier.COULD_BE_INTERESTING])}")
        logger.info(f"    Not Relevant: {len(groups[Tier.NOT_RELEVANT])}")

        if not papers_stage2:
            logger.warning("No papers passed LLM tier filter")
            return

        scored_papers = papers_stage2  # Continue with filtered papers
    else:
        # Stage 2: Topic-Embedding Scoring (replaces linear mapping)
        logger.info("=" * 60)
        logger.info("STAGE 2: Topic-Embedding Scoring (no-LLM path)")
        logger.info("=" * 60)

        from topic_scorer import TopicScorer

        topic_scorer = TopicScorer(config, local_ranker.embedder)
        logger.info(
            f"Topic scorer: {len(config.topics.primary)} primary + "
            f"{len(config.topics.secondary)} secondary topics embedded"
        )

        scored_papers = topic_scorer.score_papers(papers, config)
        logger.info(f"Topic scoring complete: {len(scored_papers)} papers scored")

        # Log tier distribution
        groups = group_by_tier(scored_papers)
        logger.info(f"  Most Relevant: {len(groups[Tier.MOST_RELEVANT])}")
        logger.info(f"  Somewhat Relevant: {len(groups[Tier.SOMEWHAT_RELEVANT])}")
        logger.info(f"  Could Be Interesting: {len(groups[Tier.COULD_BE_INTERESTING])}")
        logger.info(f"  Not Relevant: {len(groups[Tier.NOT_RELEVANT])}")

    # ========================================================================
    # STAGE 3: Summary Generation (OPTIONAL)
    # ========================================================================
    summaries = {}

    if config.summary.enabled:
        logger.info("=" * 60)
        logger.info("STAGE 3: Summary Generation")
        logger.info("=" * 60)
        summary_skip_llm = False

        # Create LLM client if not already created
        if not config.llm_scoring.enabled:
            if args.mock_llm:
                logger.info("Using mock LLM client")
                llm_client = MockLLMClient(config.llm)
            else:
                primary = config.llm.provider
                fallback = config.llm_fallback if config.llm_fallback else []
                chain = [primary] + [n for n in fallback if n != primary]
                logger.info(f"Initializing LLM clients: {chain}")

                try:
                    llm_client = create_fallback_client(config)
                    logger.info("LLM client ready")
                except Exception as e:
                    logger.warning(f"Could not create any LLM client: {e}")
                    if config.summary.fallback_to_abstract:
                        logger.warning("Will use abstracts as fallback")
                        llm_client = None
                        summary_skip_llm = True
                    else:
                        logger.error("Summary generation requires LLM client")
                        sys.exit(1)

        if llm_client is None:
            summary_skip_llm = True

        if summary_skip_llm:
            logger.info("Summary LLM unavailable, using direct fallback summaries (no retries)")

        logger.info(f"Generating summaries for {len(scored_papers)} papers...")

        try:
            summaries = generate_summaries(
                scored_papers=scored_papers,
                llm_client=llm_client,
                config=config,
                skip_llm=summary_skip_llm,
                llm_call_gap_seconds=args.llm_call_gap_seconds,
            )
            logger.info(f"Generated {len(summaries)} summaries")
        except Exception as e:
            logger.error(f"Summary generation failed: {e}")
            if config.summary.fallback_to_abstract:
                logger.warning("Falling back to abstracts")
                summaries = {}  # Empty dict = use abstracts in formatter
            else:
                logger.error("Fallback disabled, aborting")
                sys.exit(1)
    else:
        logger.info("STAGE 3: SKIPPED (summary generation disabled)")
        logger.info("Will output abstracts only")

    # ========================================================================
    # Format and Save Digest
    # ========================================================================
    logger.info("=" * 60)
    logger.info("Formatting and saving digest...")

    today = datetime.now()
    digest_content = format_digest(
        scored_papers=scored_papers,
        summaries=summaries,
        config=config,
        date=today
    )

    # Save digest
    if args.output:
        output_path = args.output
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(digest_content, encoding="utf-8")
    elif args.output_dir:
        output_path = build_dated_output_path(args.output_dir, today)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(digest_content, encoding="utf-8")
    else:
        output_path = save_digest(digest_content, config, today)

    logger.info(f"Digest saved to: {output_path}")

    # Copy to Obsidian if configured
    obsidian_path = copy_to_obsidian(digest_content, config, today)
    if obsidian_path:
        logger.info(f"Copied to Obsidian: {obsidian_path}")

    # Save state
    groups = group_by_tier(scored_papers)
    state_path = get_default_state_path(config_path)
    state_manager = StateManager(state_path)
    state = state_manager.create_state(
        papers_processed=len(scored_papers),
        most_relevant=len(groups[Tier.MOST_RELEVANT]),
        somewhat_relevant=len(groups[Tier.SOMEWHAT_RELEVANT]),
        could_be_interesting=len(groups[Tier.COULD_BE_INTERESTING])
    )
    state_manager.save_state(state)
    logger.debug(f"State saved to: {state_path}")

    # Print summary to console
    print("\n" + "=" * 60)
    print(f"Daily arXiv Summary - {today.strftime('%Y-%m-%d')}")
    print("=" * 60)
    print(f"Papers fetched: {len(papers)} (after Stage 1 filter)")
    if config.llm_scoring.enabled:
        print(f"Papers after LLM scoring: {len(scored_papers)}")
    print("\nTier Distribution:")
    print(f"  Most Relevant: {len(groups[Tier.MOST_RELEVANT])}")
    print(f"  Somewhat Relevant: {len(groups[Tier.SOMEWHAT_RELEVANT])}")
    print(f"  Could Be Interesting: {len(groups[Tier.COULD_BE_INTERESTING])}")
    if config.summary.enabled:
        print(f"\nSummaries generated: {len(summaries)}")
    else:
        print("\nSummaries: disabled (abstracts only)")
    print(f"\nDigest saved to: {output_path}")


if __name__ == "__main__":
    main()

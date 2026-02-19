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
from scorer import score_papers, group_by_tier, filter_by_tier, Tier
from summarizer import generate_summaries
from formatter import format_digest, save_digest, copy_to_obsidian
from state import StateManager, get_default_state_path
from logger import setup_logging, get_logger


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


def get_latest_digest_date(config) -> datetime | None:
    """Get the date of the most recent digest file.

    Returns:
        datetime of latest digest, or None if no digests exist
    """
    current_year = datetime.now().year
    digest_path = config.get_digest_path(current_year)

    if not digest_path.exists():
        return None

    digest_files = list(digest_path.glob("*.md"))
    if not digest_files:
        return None

    dated_files: list[tuple[datetime, float, Path]] = []
    for digest_file in digest_files:
        parsed_date = parse_digest_date_from_path(digest_file)
        if parsed_date is None:
            continue
        dated_files.append((parsed_date, digest_file.stat().st_mtime, digest_file))

    if not dated_files:
        return None

    latest_date, _, _ = max(dated_files, key=lambda item: (item[0], item[1]))
    return latest_date


def get_previous_digest_ids(config) -> set[str]:
    """Extract arxiv IDs from the latest digest file.

    Reads the most recent digest Markdown and extracts all arxiv IDs
    from both detailed sections (arXiv: [ID](url)) and title-only
    sections ([title](arxiv.org/abs/ID)).

    Returns:
        Set of arxiv ID strings, or empty set if no digest exists
    """
    current_year = datetime.now().year
    digest_path = config.get_digest_path(current_year)

    if not digest_path.exists():
        return set()

    digest_files = list(digest_path.glob("*.md"))
    if not digest_files:
        return set()

    dated_files: list[tuple[datetime, float, Path]] = []
    for digest_file in digest_files:
        parsed_date = parse_digest_date_from_path(digest_file)
        if parsed_date is None:
            continue
        dated_files.append((parsed_date, digest_file.stat().st_mtime, digest_file))

    if not dated_files:
        return set()

    _, _, latest_file = max(dated_files, key=lambda item: (item[0], item[1]))
    content = latest_file.read_text(encoding="utf-8")

    ids = set(re.findall(r"arxiv\.org/abs/(\d{4}\.\d{4,5})", content))
    return ids


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
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Override output path for digest"
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

    # Determine cutoff date from latest digest (all modes)
    since_date = None
    latest_digest_date = get_latest_digest_date(config)

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
        logger.info(f"--days specified, auto-switching source from rss to api")

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

    # Deduplicate against previous digest
    previous_ids = get_previous_digest_ids(config)
    if previous_ids:
        before = len(papers)
        papers = [p for p in papers if p.arxiv_id not in previous_ids]
        logger.info(f"Dedup against previous digest: {before} -> {len(papers)} ({before - len(papers)} already in previous digest)")

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
        for paper, local_score in zip(papers, local_scores):
            if local_score.score_total >= threshold:
                papers_stage1.append(paper)

        logger.info(f"Local filter results:")
        logger.info(f"  Input: {len(papers)} papers")
        logger.info(f"  Passed threshold: {len(papers_stage1)} papers")
        logger.info(f"  Filtered out: {len(papers) - len(papers_stage1)} papers")

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
        )

        # Filter by tier (keep only configured tiers)
        keep_tiers_str = config.llm_scoring.keep_tiers
        keep_tiers = [Tier(t) for t in keep_tiers_str]

        papers_stage2 = []
        for sp in scored_papers:
            if sp.tier in keep_tiers:
                papers_stage2.append(sp)

        logger.info(f"LLM scoring results:")
        logger.info(f"  Input: {len(scored_papers)} papers")
        logger.info(f"  Keep tiers: {', '.join(keep_tiers_str)}")
        logger.info(f"  Kept: {len(papers_stage2)} papers")
        logger.info(f"  Filtered out: {len(scored_papers) - len(papers_stage2)} papers")

        # Detailed tier breakdown
        groups = group_by_tier(scored_papers)
        logger.info(f"  Tier breakdown:")
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
                    else:
                        logger.error("Summary generation requires LLM client")
                        sys.exit(1)

        logger.info(f"Generating summaries for {len(scored_papers)} papers...")

        try:
            summaries = generate_summaries(
                scored_papers=scored_papers,
                llm_client=llm_client,
                config=config,
                skip_llm=False
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
    print(f"\nTier Distribution:")
    print(f"  Most Relevant: {len(groups[Tier.MOST_RELEVANT])}")
    print(f"  Somewhat Relevant: {len(groups[Tier.SOMEWHAT_RELEVANT])}")
    print(f"  Could Be Interesting: {len(groups[Tier.COULD_BE_INTERESTING])}")
    if config.summary.enabled:
        print(f"\nSummaries generated: {len(summaries)}")
    else:
        print(f"\nSummaries: disabled (abstracts only)")
    print(f"\nDigest saved to: {output_path}")


if __name__ == "__main__":
    main()

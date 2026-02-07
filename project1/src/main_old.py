#!/usr/bin/env python3
"""Daily arXiv Summary - CLI Entry Point."""

import argparse
import sys
from datetime import datetime
from pathlib import Path

from config import load_config, get_default_config_path
from arxiv_fetcher import fetch_papers
from llm_client import create_fallback_client, MockLLMClient
from scorer import score_papers, group_by_tier, filter_by_tier, Tier
from summarizer import generate_summaries
from formatter import format_digest, save_digest, copy_to_obsidian
from state import StateManager, get_default_state_path
from logger import setup_logging, get_logger


def main():
    parser = argparse.ArgumentParser(
        description="Daily arXiv Summary - Monitor arXiv for relevant papers"
    )
    parser.add_argument(
        "--mode",
        choices=["update", "debug"],
        default="debug",
        help="Run mode: 'update' checks for new papers, 'debug' always runs"
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=None,
        help="Path to config.yaml (default: project1/config.yaml)"
    )
    parser.add_argument(
        "--category",
        type=str,
        default=None,
        help="Override: fetch only from this category"
    )
    parser.add_argument(
        "--days",
        type=int,
        default=1,
        help="Number of days to look back (default: 1)"
    )
    parser.add_argument(
        "--max-papers",
        type=int,
        default=None,
        help="Override: max papers per category"
    )
    parser.add_argument(
        "--skip-scoring",
        action="store_true",
        help="Skip scoring and summary (fetch only)"
    )
    parser.add_argument(
        "--skip-summary",
        action="store_true",
        help="Skip summary generation (scoring only)"
    )
    parser.add_argument(
        "--skip-llm",
        action="store_true",
        help="Skip LLM calls, use prefilter scoring and fallback summaries"
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
        "--use-local-filter",
        action="store_true",
        help="Use local embedding-based filter (requires interest model)"
    )
    parser.add_argument(
        "--local-filter-threshold",
        type=float,
        default=0.3,
        help="Local score threshold for LLM pre-filter (default: 0.3)"
    )
    parser.add_argument(
        "--interest-model",
        type=Path,
        default=None,
        help="Path to interest_model.json (default: song_db/artifacts/interest_model.json)"
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

    args = parser.parse_args()

    # Setup logging
    import logging
    log_level = logging.DEBUG if args.verbose else logging.INFO
    logger = setup_logging(
        console_level=log_level,
        log_to_file=not args.no_log_file
    )

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

    logger.info(f"Mode: {args.mode}")
    logger.debug(f"Primary categories: {config.category.primary}")
    logger.debug(f"Secondary categories: {config.category.secondary}")

    # Check state in update mode
    state_path = get_default_state_path(config_path)
    state_manager = StateManager(state_path)

    if args.mode == "update":
        should_run, reason = state_manager.should_run()
        if not should_run:
            logger.info(f"Skipping: {reason}")
            return
        logger.info(f"Running: {reason}")
    else:
        logger.info("Debug mode: ignoring state")

    # Determine categories to fetch
    if args.category:
        categories = [args.category]
    else:
        categories = config.category.all_categories()

    # Determine max papers
    max_papers = args.max_papers if args.max_papers else config.api.arxiv_max_results_per_category

    logger.info(f"Fetching papers from {len(categories)} categories...")
    logger.info(f"Max papers per category: {max_papers}")
    logger.info(f"Days to look back: {args.days}")

    # Fetch papers
    papers = fetch_papers(
        categories=categories,
        max_results_per_category=max_papers,
        delay_seconds=config.api.arxiv_delay_seconds,
        days=args.days
    )

    logger.info(f"Total papers fetched: {len(papers)}")

    if not papers:
        logger.warning("No papers found.")
        return

    # Skip scoring if requested
    if args.skip_scoring:
        logger.info("Scoring skipped (--skip-scoring)")
        for i, paper in enumerate(papers[:20], 1):
            print(f"{i}. [{paper.primary_category}] {paper.arxiv_id}: {paper.title[:60]}...")
        if len(papers) > 20:
            print(f"... and {len(papers) - 20} more papers")
        return

    # Limit papers for testing
    if args.limit:
        papers = papers[:args.limit]
        logger.info(f"Limited to {len(papers)} papers for processing")

    # Create LLM client
    if args.mock_llm:
        logger.info("Using mock LLM client")
        llm_client = MockLLMClient(config.llm)
    else:
        fallback_names = config.llm_fallback if config.llm_fallback else [config.llm.provider]
        logger.info(f"Initializing LLM clients: {fallback_names}")

        try:
            llm_client = create_fallback_client(config)
            logger.info("LLM client ready")
        except Exception as e:
            logger.warning(f"Could not create any LLM client: {e}")
            logger.warning("Falling back to mock LLM client")
            llm_client = MockLLMClient(config.llm)

    # Load local ranker if requested
    local_ranker = None
    local_threshold = 0.0
    if args.use_local_filter:
        try:
            from local_scorer import load_local_ranker
            model_path = args.interest_model
            if model_path is None:
                model_path = Path(__file__).parent.parent / "song_db" / "artifacts" / "interest_model.json"
            logger.info(f"Loading local ranker from: {model_path}")
            local_ranker = load_local_ranker(model_path)
            local_threshold = args.local_filter_threshold if not args.skip_llm else 0.0
            logger.info(f"Local ranker loaded (threshold: {local_threshold})")
        except Exception as e:
            logger.warning(f"Could not load local ranker: {e}")
            logger.warning("Continuing without local filter")

    # Score papers
    logger.info("Scoring papers...")

    scored_papers = score_papers(
        papers=papers,
        config=config,
        llm_client=llm_client,
        skip_llm=args.skip_llm,
        local_ranker=local_ranker,
        local_threshold=local_threshold,
    )

    # Filter out not relevant papers
    relevant_papers = filter_by_tier(scored_papers, Tier.COULD_BE_INTERESTING)

    # Group by tier
    groups = group_by_tier(relevant_papers)

    # Log scoring results
    logger.info(f"Scoring complete:")
    logger.info(f"  Most Relevant: {len(groups[Tier.MOST_RELEVANT])}")
    logger.info(f"  Somewhat Relevant: {len(groups[Tier.SOMEWHAT_RELEVANT])}")
    logger.info(f"  Could Be Interesting: {len(groups[Tier.COULD_BE_INTERESTING])}")
    logger.info(f"  Not Relevant: {len(groups[Tier.NOT_RELEVANT])}")

    if not relevant_papers:
        logger.warning("No relevant papers found.")
        return

    # Generate summaries
    summaries = {}
    if not args.skip_summary:
        logger.info("Generating summaries...")

        summaries = generate_summaries(
            scored_papers=relevant_papers,
            llm_client=llm_client,
            config=config,
            skip_llm=args.skip_llm
        )

        logger.info(f"Generated {len(summaries)} summaries")

    # Format digest
    logger.info("Formatting digest...")

    today = datetime.now()
    digest_content = format_digest(
        scored_papers=relevant_papers,
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
    print(f"Papers processed: {len(scored_papers)}")
    print(f"Most Relevant: {len(groups[Tier.MOST_RELEVANT])}")
    print(f"Somewhat Relevant: {len(groups[Tier.SOMEWHAT_RELEVANT])}")
    print(f"Could Be Interesting: {len(groups[Tier.COULD_BE_INTERESTING])}")
    print(f"\nDigest saved to: {output_path}")


if __name__ == "__main__":
    main()

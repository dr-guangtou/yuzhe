"""CLI entry point for the Personal Publication Record pipeline."""

import argparse
import logging
import os
import sys
from pathlib import Path

# Allow running from project root: python project2/src/main.py
sys.path.insert(0, str(Path(__file__).parent))

from config import load_config, get_default_config_path
from orcid_fetcher import ORCIDFetcher
from ads_fetcher import ADSFetcher
from database import PublicationDatabase
from summarizer import Summarizer
from portfolio_builder import PortfolioBuilder
from state import StateManager
from llm_client import create_fallback_client

logger = logging.getLogger(__name__)


def setup_logging(debug: bool = False):
    level = logging.DEBUG if debug else logging.INFO
    logging.basicConfig(
        level=level,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
        datefmt="%H:%M:%S",
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Personal Publication Record: fetch, summarize, and build portfolio"
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=None,
        help="Path to config.yaml (default: project2/config.yaml)",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Force re-run all publications regardless of state",
    )
    parser.add_argument(
        "--skip-summaries",
        action="store_true",
        help="Only fetch/store publications, skip LLM summary generation",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    setup_logging(debug=args.debug)

    # Resolve config path
    config_path = args.config or get_default_config_path()
    config_path = config_path.resolve()
    logger.info("Loading config from %s", config_path)
    config = load_config(config_path)

    # Setup paths
    project_dir = config_path.parent
    record_dir = config.get_record_path()
    record_dir.mkdir(parents=True, exist_ok=True)

    # State management
    state_mgr = StateManager(project_dir / ".state.json")

    # Step 1: Fetch publications from ORCID
    logger.info("Step 1: Fetching publications from ORCID...")
    fetcher = ORCIDFetcher(
        orcid_id=config.user.orcid,
        year_cutoff=config.user.year_cutoff,
        timeout=config.api.orcid_timeout,
    )
    xml_data = fetcher.fetch_works()
    publications = fetcher.parse_publications(xml_data)
    logger.info("Fetched %d publications from ORCID", len(publications))

    # Step 2: Enrich via ADS
    ads_token = os.environ.get(config.api.ads_api_key_env)
    if ads_token:
        logger.info("Step 2: Enriching via ADS API...")
        ads = ADSFetcher(
            api_token=ads_token,
            timeout=config.api.ads_timeout,
            rate_limit=config.api.ads_rate_limit_delay,
        )
        publications = ads.enrich_all(publications)
    else:
        logger.warning(
            "ADS API token not found (%s). Skipping enrichment.",
            config.api.ads_api_key_env,
        )

    # Step 3: Merge into database
    logger.info("Step 3: Merging into publication database...")
    db_path = record_dir / "publication_list.yaml"
    db = PublicationDatabase(db_path)
    all_pubs, new_pubs = db.merge(publications)
    db.save(all_pubs)
    db.generate_markdown_list(all_pubs)
    logger.info("Database: %d total, %d new", len(all_pubs), len(new_pubs))

    # Step 4: Check if we need to generate summaries
    if not new_pubs and not args.debug:
        logger.info("No new publications found. Use --debug to force re-run.")
        state = state_mgr.create_state(
            publications_count=len(all_pubs), summaries_generated=0
        )
        state_mgr.save_state(state)
        return

    # Determine which publications to summarize
    pubs_to_summarize = all_pubs if args.debug else new_pubs

    if args.skip_summaries:
        logger.info("Skipping summaries (--skip-summaries). Building portfolio with available data.")
        all_summaries = {}
    else:
        # Step 5: Generate summaries
        logger.info(
            "Step 4: Generating summaries for %d publications...",
            len(pubs_to_summarize),
        )

        llm_client = create_fallback_client(config)

        # Determine model name for output directory
        provider_name = config.llm.provider
        provider_cfg = config.get_provider(provider_name)
        model_name = config.llm.model or provider_cfg.default_model
        # Sanitize model name for directory
        model_name = model_name.replace("/", "_").replace(":", "_")

        summarizer = Summarizer(
            llm_client=llm_client,
            prompts_dir=project_dir / "prompts",
            output_dir=record_dir,
            model_name=model_name,
            retry_attempts=config.api.llm_retry_attempts,
            retry_delay=config.api.llm_retry_delay_seconds,
        )

        all_summaries = {}
        for pub in pubs_to_summarize:
            pub_key = pub.doi or pub.title
            try:
                summaries = summarizer.process_publication(pub)
                if summaries:
                    all_summaries[pub_key] = summaries
            except Exception as e:
                logger.error(
                    "Failed to summarize '%s': %s", pub.title[:50], e
                )

        if all_summaries:
            summarizer.save_summaries(all_summaries)
        logger.info("Generated summaries for %d publications", len(all_summaries))

    # Step 6: Build portfolio
    logger.info("Step 5: Building portfolio documents...")
    portfolio_dir = config.get_portfolio_path()
    builder = PortfolioBuilder(
        portfolio_dir=portfolio_dir,
        user_info={
            "first_name": config.user.first_name,
            "last_name": config.user.last_name,
            "chinese_name": config.user.chinese_name,
        },
    )
    builder.build_all(all_pubs, all_summaries)

    # Save state
    state = state_mgr.create_state(
        publications_count=len(all_pubs),
        summaries_generated=len(all_summaries),
    )
    state_mgr.save_state(state)

    logger.info("Pipeline complete. %d publications, %d summaries.",
                len(all_pubs), len(all_summaries))


if __name__ == "__main__":
    main()

"""CLI entry point for Periodic Journal Summary.

Orchestrates the full pipeline:
fetch -> parse -> dedup -> enrich -> local filter -> topic score -> archive -> remind
"""

import argparse
import sys
from datetime import datetime
from pathlib import Path


def main():
    args = parse_args()

    # Add project3/src to path for local imports
    src_dir = Path(__file__).resolve().parent
    if str(src_dir) not in sys.path:
        sys.path.insert(0, str(src_dir))

    from p3_config import load_config, setup_project1_imports
    from fetcher import fetch_url
    from parser import parse_feed
    from enricher import enrich_articles
    from scorer_bridge import (
        load_local_ranker,
        load_topic_scorer,
        filter_with_local_ranker,
        score_with_topic_scorer,
    )
    from archiver import archive_articles
    from reminder import generate_reminder
    from state import State

    # Step 1: Load configs
    config_path = Path(args.config).resolve()
    p3_config = load_config(config_path)
    project_root = config_path.parent

    print(f"Config loaded: {len(p3_config.journals)} journals configured")

    # Step 2: Load state
    state_path = project_root / ".state.json"
    state = State(state_path)
    print(f"State loaded: {len(state.seen_dois)} previously seen DOIs")

    # Step 3: Load scorers
    if not args.dry_run:
        print("Loading scoring models...")
        setup_project1_imports(p3_config.project1_config_path)
        ranker, embedder = load_local_ranker(p3_config)
        topic_scorer, p1_config = load_topic_scorer(p3_config, embedder)
        print("Scoring models loaded")

    # Step 4: Fetch and parse feeds
    all_articles = []
    enabled_journals = [j for j in p3_config.journals if j.enabled]

    if args.journal:
        enabled_journals = [j for j in enabled_journals if j.acronym == args.journal]
        if not enabled_journals:
            print(f"Error: journal '{args.journal}' not found or not enabled")
            sys.exit(1)

    print(f"\nFetching {len(enabled_journals)} journal feeds...")
    for journal in enabled_journals:
        print(f"  Fetching {journal.name} ({journal.acronym})...")
        try:
            content = fetch_url(
                journal.feed_url,
                timeout=p3_config.fetch.timeout_seconds,
                user_agent=p3_config.fetch.user_agent,
                delay=p3_config.fetch.delay_seconds,
            )
            articles = parse_feed(content, journal)
            print(f"    Parsed {len(articles)} articles")
            all_articles.extend(articles)
            state.mark_fetched(journal.acronym)
        except Exception as e:
            print(f"    Error: {e}")
            continue

    if not all_articles:
        print("\nNo articles fetched. Exiting.")
        return

    # Step 5: Dedup by DOI + filter already-seen
    before_dedup = len(all_articles)
    all_articles = _dedup_by_doi(all_articles)
    all_articles = state.filter_unseen(all_articles)
    print(f"\nDedup: {before_dedup} -> {len(all_articles)} new articles")

    if not all_articles:
        print("No new articles after dedup. Exiting.")
        state.save()
        return

    # Step 6: Enrich abstracts (optional)
    if not args.no_enrichment:
        print("\nEnriching abstracts...")
        enrich_articles(all_articles, p3_config.enrichment, p3_config.fetch.user_agent)

    if args.dry_run:
        print(f"\n[Dry run] Would process {len(all_articles)} articles")
        for a in all_articles[:5]:
            print(f"  - [{a.journal}] {a.title[:80]}")
            if a.doi:
                print(f"    DOI: {a.doi}")
        if len(all_articles) > 5:
            print(f"  ... and {len(all_articles) - 5} more")
        return

    # Step 7: Stage 1 - LocalRanker corpus filter
    print("\nStage 1: LocalRanker corpus filter...")
    stage1_survivors = filter_with_local_ranker(
        all_articles, ranker, p3_config.local_filter.threshold
    )

    # Step 8: Archive ALL Stage 1 survivors (permanent database)
    print("\nArchiving Stage 1 survivors...")
    archive_dir = project_root / p3_config.output.archive_dir
    date_str = datetime.now().strftime("%Y-%m-%d")
    counts = archive_articles(stage1_survivors, archive_dir, date_str)
    for journal, count in sorted(counts.items()):
        print(f"  {journal}: {count} articles archived")

    # Step 9: Stage 2 - TopicScorer ranking
    print("\nStage 2: TopicScorer ranking...")
    scored_papers = score_with_topic_scorer(stage1_survivors, topic_scorer, p1_config)

    # Import tier info for summary
    from scorer import Tier, filter_by_tier, group_by_tier
    min_tier_map = {
        "most_relevant": Tier.MOST_RELEVANT,
        "somewhat_relevant": Tier.SOMEWHAT_RELEVANT,
        "could_be_interesting": Tier.COULD_BE_INTERESTING,
    }
    min_tier = min_tier_map.get(p3_config.scoring.min_tier, Tier.COULD_BE_INTERESTING)
    filtered = filter_by_tier(scored_papers, min_tier)

    # Print tier summary
    groups = group_by_tier(scored_papers)
    for tier in [Tier.MOST_RELEVANT, Tier.SOMEWHAT_RELEVANT, Tier.COULD_BE_INTERESTING, Tier.NOT_RELEVANT]:
        papers = groups.get(tier, [])
        if papers:
            print(f"  {tier.value}: {len(papers)} papers")

    # Step 10: Generate monthly reminder
    print("\nGenerating monthly reminder...")
    reminder_dir = project_root / p3_config.output.reminder_dir
    generate_reminder(filtered, reminder_dir)

    # Step 11: Update state
    new_dois = [a.doi for a in all_articles if a.doi]
    state.add_seen_dois(new_dois)
    state.save()
    print(f"\nState saved: {len(state.seen_dois)} total DOIs tracked")

    # Final summary
    print(f"\nDone! Processed {len(all_articles)} new articles:")
    print(f"  Stage 1 (field-relevant): {len(stage1_survivors)}")
    print(f"  Stage 2 (topic-relevant): {len(filtered)}")

    if args.debug:
        print("\n[Debug] Top 10 scored papers:")
        for sp in scored_papers[:10]:
            print(f"  [{sp.tier.value}] {sp.score:.1f} - {sp.paper.title[:70]}")
            if sp.matched_topics:
                print(f"    Topics: {', '.join(sp.matched_topics[:3])}")


def _dedup_by_doi(articles: list) -> list:
    """Remove duplicate articles by DOI (keep first occurrence)."""
    seen = set()
    result = []
    for article in articles:
        if article.doi:
            if article.doi in seen:
                continue
            seen.add(article.doi)
        result.append(article)
    return result


def parse_args():
    parser = argparse.ArgumentParser(
        description="Periodic Journal Summary - fetch, score, and archive journal articles"
    )
    parser.add_argument(
        "--config",
        default=str(Path(__file__).resolve().parent.parent / "config.yaml"),
        help="Path to config.yaml (default: project3/config.yaml)",
    )
    parser.add_argument(
        "--journal",
        help="Process only this journal (by acronym, e.g. 'apj')",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Fetch and parse only, no scoring or archiving",
    )
    parser.add_argument(
        "--no-enrichment",
        action="store_true",
        help="Skip abstract enrichment step",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Print additional debug information",
    )
    return parser.parse_args()


if __name__ == "__main__":
    main()

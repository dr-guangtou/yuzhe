#!/usr/bin/env python3
"""Score a single arXiv paper against configured research interests.

Usage:
    uv run python src/get_llm_score.py 2602.04962
    uv run python src/get_llm_score.py 2602.04962 2602.04974 2602.05396
    uv run python src/get_llm_score.py 2602.04962 --provider moonshot
    uv run python src/get_llm_score.py 2602.04962 --prompt prompts/match_preprint.md
"""

import argparse
import json
import re
import sys
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

# Add src/ to path so we can import siblings
sys.path.insert(0, str(Path(__file__).parent))

from config import load_config, get_default_config_path
from llm_client import create_single_client


# arXiv namespaces (duplicated to keep this script self-contained for fetching)
ATOM_NS = "http://www.w3.org/2005/Atom"
ARXIV_NS = "http://arxiv.org/schemas/atom"


def fetch_paper_by_id(arxiv_id: str) -> dict:
    """Fetch a single paper's metadata from the arXiv API.

    Returns a dict with title, abstract, authors, categories,
    primary_category, and arxiv_id.
    """
    clean_id = re.sub(r'v\d+$', '', arxiv_id)
    match = re.search(r'(\d{4}\.\d{4,5})', clean_id)
    if match:
        clean_id = match.group(1)

    url = f"https://export.arxiv.org/api/query?id_list={clean_id}&max_results=1"
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "daily-arxiv-summary/1.0"},
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        xml_data = resp.read()

    root = ET.fromstring(xml_data)
    entry = root.find(f"{{{ATOM_NS}}}entry")
    if entry is None:
        raise ValueError(f"Paper not found: {arxiv_id}")

    # Check for error (arXiv returns an entry with id containing "Error")
    id_elem = entry.find(f"{{{ATOM_NS}}}id")
    if id_elem is not None and "Error" in (id_elem.text or ""):
        raise ValueError(f"Paper not found: {arxiv_id}")

    title_elem = entry.find(f"{{{ATOM_NS}}}title")
    title = " ".join((title_elem.text or "").split())

    summary_elem = entry.find(f"{{{ATOM_NS}}}summary")
    abstract = " ".join((summary_elem.text or "").split())

    authors = []
    for author in entry.findall(f"{{{ATOM_NS}}}author"):
        name_elem = author.find(f"{{{ATOM_NS}}}name")
        if name_elem is not None and name_elem.text:
            authors.append(name_elem.text)

    categories = []
    for cat in entry.findall(f"{{{ATOM_NS}}}category"):
        term = cat.get("term")
        if term:
            categories.append(term)

    primary_cat_elem = entry.find(f"{{{ARXIV_NS}}}primary_category")
    primary_category = primary_cat_elem.get("term", "") if primary_cat_elem is not None else ""

    return {
        "arxiv_id": clean_id,
        "title": title,
        "abstract": abstract,
        "authors": authors,
        "categories": categories,
        "primary_category": primary_category,
    }


def build_prompt(template: str, paper: dict, config) -> str:
    """Fill the prompt template with paper and config data."""
    return template.format(
        primary_topics="\n".join(f"- {t}" for t in config.topics.primary),
        secondary_topics="\n".join(f"- {t}" for t in config.topics.secondary),
        projects="\n".join(f"- {p.name} ({p.acronym})" for p in config.projects),
        title=paper["title"],
        abstract=paper["abstract"],
        categories=", ".join(paper["categories"]),
    )


def get_provider_candidates(config, explicit_provider: str | None) -> list[str]:
    """Build provider resolution order for get_llm_score.

    - If provider is explicitly passed, only that provider is used.
    - Otherwise, try primary first, then fallbacks.
    """
    if explicit_provider:
        return [explicit_provider]
    primary = config.llm.provider
    fallback = config.llm_fallback if config.llm_fallback else []
    return [primary] + [name for name in fallback if name != primary]


def parse_llm_response(content: str) -> dict:
    """Extract score, matched_topics, and reasoning from the LLM response."""
    # Try JSON extraction
    json_match = re.search(r'\{[^{}]*\}', content, re.DOTALL)
    if json_match:
        try:
            result = json.loads(json_match.group(0))
            return {
                "score": float(result.get("score", -1)),
                "matched_topics": result.get("matched_topics", []),
                "reasoning": result.get("reasoning", ""),
            }
        except (json.JSONDecodeError, ValueError):
            pass

    # Regex fallback for score
    score_match = re.search(r'"?score"?\s*[:=]\s*(\d+(?:\.\d+)?)', content)
    if score_match:
        return {
            "score": float(score_match.group(1)),
            "matched_topics": [],
            "reasoning": "Parsed with regex fallback",
        }

    return {"score": -1, "matched_topics": [], "reasoning": f"Parse failed. Raw: {content[:200]}"}


def score_one_paper(paper: dict, config, llm_client, prompt_template: str) -> dict:
    """Score a single paper and return structured result."""
    prompt = build_prompt(prompt_template, paper, config)
    response = llm_client.generate(prompt)
    result = parse_llm_response(response.content)
    result["provider"] = response.provider
    result["model"] = response.model

    # Check project match
    title_lower = paper["title"].lower()
    matched_projects = []
    for project in config.projects:
        if project.acronym:
            pattern = r'\b' + re.escape(project.acronym.lower()) + r'\b'
            if re.search(pattern, title_lower):
                matched_projects.append(project.acronym)
                continue
        if project.name:
            pattern = r'\b' + re.escape(project.name.lower()) + r'\b'
            if re.search(pattern, title_lower):
                matched_projects.append(project.name)
    result["matched_projects"] = matched_projects

    # Determine tier
    score = result["score"]
    thresholds = config.llm_scoring.tier_thresholds
    if score >= thresholds.most_relevant:
        result["tier"] = "Most Relevant"
    elif score >= thresholds.somewhat_relevant:
        result["tier"] = "Somewhat Relevant"
    elif score >= thresholds.could_be_interesting:
        result["tier"] = "Could Be Interesting"
    else:
        result["tier"] = "Not Relevant"

    return result


def print_result(paper: dict, result: dict):
    """Pretty-print the scoring result for one paper."""
    print(f"\n{'=' * 70}")
    print(f"  arXiv: {paper['arxiv_id']}  |  Score: {result['score']}/10  |  Tier: {result['tier']}")
    print(f"  Model: {result['provider']}/{result['model']}")
    print(f"{'=' * 70}")
    print(f"  Title:    {paper['title'][:90]}")
    if len(paper["title"]) > 90:
        print(f"            {paper['title'][90:]}")
    print(f"  Category: {paper['primary_category']}  ({', '.join(paper['categories'])})")
    first_authors = paper["authors"][:3]
    author_str = ", ".join(first_authors)
    if len(paper["authors"]) > 3:
        author_str += f", et al. ({len(paper['authors'])} total)"
    print(f"  Authors:  {author_str}")
    print()

    if result["matched_topics"]:
        print("  Matched Topics:")
        for topic in result["matched_topics"]:
            print(f"    - {topic}")
        print()

    if result["matched_projects"]:
        print(f"  Matched Projects: {', '.join(result['matched_projects'])}")
        print()

    if result["reasoning"]:
        print(f"  Reasoning: {result['reasoning']}")

    print()


def main():
    parser = argparse.ArgumentParser(
        description="Score arXiv papers against your configured research interests.",
        epilog="Examples:\n"
               "  uv run python src/get_llm_score.py 2602.04962\n"
               "  uv run python src/get_llm_score.py 2602.04962 2602.04974 --provider nvidia\n"
               "  uv run python src/get_llm_score.py 2602.04962 --prompt prompts/my_prompt.md\n",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "arxiv_ids",
        nargs="+",
        help="One or more arXiv IDs (e.g., 2602.04962) or URLs",
    )
    parser.add_argument(
        "--provider",
        type=str,
        default=None,
        help="LLM provider name from config.yaml (default: primary provider, then fallback chain)",
    )
    parser.add_argument(
        "--prompt",
        type=Path,
        default=None,
        help="Path to scoring prompt template (default: prompts/match_preprint.md)",
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=None,
        help="Path to config.yaml",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        dest="output_json",
        help="Output results as JSON",
    )

    args = parser.parse_args()

    # Load config
    config_path = args.config if args.config else get_default_config_path()
    config = load_config(config_path)

    # Load prompt template
    if args.prompt:
        prompt_path = args.prompt if args.prompt.is_absolute() else config_path.parent / args.prompt
    else:
        prompt_path = config_path.parent / "prompts" / "match_preprint.md"

    if not prompt_path.exists():
        print(f"Error: Prompt file not found: {prompt_path}", file=sys.stderr)
        sys.exit(1)
    prompt_template = prompt_path.read_text(encoding="utf-8")

    # Create LLM client
    provider_candidates = get_provider_candidates(config, args.provider)
    llm_client = None
    provider_name = ""
    provider_errors = []

    for candidate in provider_candidates:
        prov_config = config.get_provider(candidate)
        model = config.llm.model if candidate == config.llm.provider else ""
        try:
            llm_client = create_single_client(
                provider_name=candidate,
                provider_config=prov_config,
                model=model,
                temperature=config.llm.temperature,
                max_tokens=config.llm.max_tokens,
            )
            provider_name = candidate
            break
        except ValueError as e:
            provider_errors.append(f"{candidate}: {e}")

    if llm_client is None:
        print("Error: could not initialize any provider", file=sys.stderr)
        for error in provider_errors:
            print(f"  - {error}", file=sys.stderr)
        sys.exit(1)

    print(f"Provider: {provider_name}  |  Prompt: {prompt_path.name}")
    thresholds = config.llm_scoring.tier_thresholds
    print(f"Thresholds: most_relevant >= {thresholds.most_relevant}, "
          f"somewhat >= {thresholds.somewhat_relevant}, "
          f"interesting >= {thresholds.could_be_interesting}")

    # Process each paper
    scored = []  # list of (paper, result) tuples
    for i, raw_id in enumerate(args.arxiv_ids):
        # Rate limit: arXiv API requires minimum 3 seconds between requests
        # See: ~/.claude/skills/arxiv-public-api/references/terms_of_use.md
        if i > 0:
            time.sleep(3)

        # Fetch paper
        try:
            paper = fetch_paper_by_id(raw_id)
        except Exception as e:
            print(f"\nError fetching {raw_id}: {e}", file=sys.stderr)
            continue

        # Score
        try:
            result = score_one_paper(paper, config, llm_client, prompt_template)
        except Exception as e:
            print(f"\nError scoring {raw_id}: {e}", file=sys.stderr)
            continue

        scored.append((paper, result))
        if not args.output_json:
            print_result(paper, result)

    if args.output_json:
        json_out = []
        for paper, result in scored:
            json_out.append({
                "arxiv_id": paper["arxiv_id"],
                "title": paper["title"],
                "primary_category": paper["primary_category"],
                "score": result["score"],
                "tier": result["tier"],
                "matched_topics": result["matched_topics"],
                "matched_projects": result["matched_projects"],
                "reasoning": result["reasoning"],
            })
        print(json.dumps(json_out, indent=2))

    # Print summary table if multiple papers
    if len(scored) > 1 and not args.output_json:
        print("\n" + "=" * 70)
        print("  SUMMARY")
        print("=" * 70)
        print(f"  {'arXiv ID':<14} {'Score':>5}  {'Tier':<22} Title (truncated)")
        print(f"  {'-' * 12:<14} {'-----':>5}  {'-' * 20:<22} {'-' * 30}")
        for paper, result in scored:
            title_short = paper["title"][:40]
            if len(paper["title"]) > 40:
                title_short += "..."
            print(f"  {paper['arxiv_id']:<14} {result['score']:>5.1f}  {result['tier']:<22} {title_short}")


if __name__ == "__main__":
    main()

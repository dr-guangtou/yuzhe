"""Score papers for relevance against configured topics.

Scoring Strategy:
1. Category: Determines which papers to consider (fetch filter)
2. Topic: CORE of scoring - LLM evaluates paper relevance to topic descriptions
3. Project: BOOSTER only - ensures minimum tier floor, does not drive the score

Flow: Fetch by category → LLM scores against TOPICS → Project boosts floor → Tier
"""

import json
import re
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Optional

from arxiv_fetcher import ArxivPaper
from config import Config
from llm_client import LLMClient, retry_with_backoff


class Tier(Enum):
    """Relevance tiers for papers."""
    MOST_RELEVANT = "most_relevant"
    SOMEWHAT_RELEVANT = "somewhat_relevant"
    COULD_BE_INTERESTING = "could_be_interesting"
    NOT_RELEVANT = "not_relevant"


@dataclass
class ScoredPaper:
    """A paper with relevance scoring."""
    paper: ArxivPaper
    score: float
    tier: Tier
    matched_topics: list[str] = field(default_factory=list)
    matched_projects: list[str] = field(default_factory=list)
    reasoning: str = ""
    prefilter_passed: bool = True
    project_match: bool = False
    primary_category_match: bool = False


def load_scoring_prompt() -> str:
    """Load the scoring prompt template."""
    prompt_path = Path(__file__).parent.parent / "prompts" / "match_preprint.md"
    if not prompt_path.exists():
        raise FileNotFoundError(f"Scoring prompt not found: {prompt_path}")
    return prompt_path.read_text(encoding="utf-8")


def check_project_match(paper: ArxivPaper, config: Config) -> list[str]:
    """Check if paper title mentions any projects of interest.

    Uses word boundary matching to avoid false positives (e.g., WST in JWST).
    This is used as a BOOSTER, not for primary scoring.

    Args:
        paper: The paper to check
        config: Configuration with projects list

    Returns:
        List of matched project names/acronyms
    """
    title_lower = paper.title.lower()
    matched = []

    for project in config.projects:
        # Check acronym with word boundary
        if project.acronym:
            pattern = r'\b' + re.escape(project.acronym.lower()) + r'\b'
            if re.search(pattern, title_lower):
                matched.append(project.acronym)
                continue

        # Check full name with word boundary
        if project.name:
            pattern = r'\b' + re.escape(project.name.lower()) + r'\b'
            if re.search(pattern, title_lower):
                matched.append(project.name)

    return matched


def check_category_match(paper: ArxivPaper, config: Config) -> tuple[bool, bool]:
    """Check if paper's PRIMARY category matches configured categories.

    Only checks the paper's primary/main subject, NOT cross-listings.
    This ensures we don't include papers that are primarily about
    unrelated fields (e.g., hep-ph) even if cross-listed in astro-ph.

    Args:
        paper: The paper to check
        config: Configuration with category lists

    Returns:
        Tuple of (primary_match, secondary_match)
        - primary_match: True if paper's PRIMARY category is in config's primary list
        - secondary_match: True if paper's PRIMARY category is in config's secondary list
    """
    # Only check the paper's PRIMARY category, not all cross-listings
    primary_cat = paper.primary_category

    primary_match = primary_cat in config.category.primary
    secondary_match = primary_cat in config.category.secondary

    return primary_match, secondary_match


def prefilter_paper(paper: ArxivPaper, config: Config) -> tuple[bool, list[str], bool, bool]:
    """Simple prefilter - check if paper is in any configured category.

    This is intentionally permissive. The LLM scoring against topics is
    what determines relevance, not the prefilter.

    Args:
        paper: The paper to check
        config: Configuration

    Returns:
        Tuple of (passed, matched_projects, primary_cat_match, secondary_cat_match)
    """
    matched_projects = check_project_match(paper, config)
    primary_match, secondary_match = check_category_match(paper, config)

    # Pass if in ANY configured category (don't filter aggressively)
    passed = primary_match or secondary_match

    return passed, matched_projects, primary_match, secondary_match


def score_paper_with_llm(
    paper: ArxivPaper,
    config: Config,
    llm_client: LLMClient,
    prompt_template: str
) -> tuple[float, list[str], str]:
    """Score a paper using LLM against configured TOPICS.

    This is the CORE scoring mechanism. The LLM evaluates how relevant
    the paper's title and abstract are to the researcher's topic descriptions.

    Args:
        paper: The paper to score
        config: Configuration with topics
        llm_client: LLM client instance
        prompt_template: The scoring prompt template

    Returns:
        Tuple of (score, matched_topics, reasoning)
    """
    prompt = prompt_template.format(
        primary_topics="\n".join(f"- {t}" for t in config.topics.primary),
        secondary_topics="\n".join(f"- {t}" for t in config.topics.secondary),
        projects="\n".join(f"- {p.name} ({p.acronym})" for p in config.projects),
        title=paper.title,
        abstract=paper.abstract,
        categories=", ".join(paper.categories)
    )

    def call_llm():
        return llm_client.generate(prompt)

    try:
        response = retry_with_backoff(
            call_llm,
            max_attempts=config.api.llm_retry_attempts,
            initial_delay=config.api.llm_retry_delay_seconds
        )
    except Exception as e:
        print(f"Warning: LLM scoring failed for {paper.arxiv_id}: {e}")
        return 5.0, [], f"LLM scoring failed: {e}"

    # Parse JSON response
    try:
        content = response.content
        # Extract JSON from markdown code blocks if present
        json_match = re.search(r'\{[^{}]*\}', content, re.DOTALL)
        if json_match:
            content = json_match.group(0)

        result = json.loads(content)
        score = float(result.get("score", 5.0))
        matched_topics = result.get("matched_topics", [])
        reasoning = result.get("reasoning", "")

        score = max(0.0, min(10.0, score))
        return score, matched_topics, reasoning

    except (json.JSONDecodeError, ValueError) as e:
        print(f"Warning: Failed to parse LLM response for {paper.arxiv_id}: {e}")
        score_match = re.search(r'"?score"?\s*[:=]\s*(\d+(?:\.\d+)?)', response.content)
        if score_match:
            return float(score_match.group(1)), [], "Parsed with regex fallback"
        return 5.0, [], f"Parse failed: {e}"


def assign_tier(
    score: float,
    project_match: bool,
    config: Config
) -> Tier:
    """Assign tier based on topic relevance score.

    The score comes from LLM evaluation against TOPICS.
    Project match only provides a floor (ensures at least "Could Be Interesting").

    Args:
        score: Topic relevance score (0-10)
        project_match: Whether paper mentions a tracked project (booster)
        config: Configuration with thresholds

    Returns:
        Assigned Tier
    """
    thresholds = config.llm_scoring.tier_thresholds

    # Most Relevant: high topic relevance
    if score >= thresholds.most_relevant:
        return Tier.MOST_RELEVANT

    # Somewhat Relevant: moderate topic relevance
    if score >= thresholds.somewhat_relevant:
        return Tier.SOMEWHAT_RELEVANT

    # Could Be Interesting: low topic relevance OR project boost
    if score >= thresholds.could_be_interesting:
        return Tier.COULD_BE_INTERESTING

    # Project match ensures minimum floor of "Could Be Interesting"
    if project_match:
        return Tier.COULD_BE_INTERESTING

    return Tier.NOT_RELEVANT


def score_papers(
    papers: list[ArxivPaper],
    config: Config,
    llm_client: LLMClient,
    skip_llm: bool = False,
    local_ranker=None,
    local_threshold: float = 0.0,
) -> list[ScoredPaper]:
    """Score papers against configured topics.

    Scoring Strategy:
    1. LLM evaluates each paper against TOPIC descriptions (primary mechanism)
    2. Project matches provide a floor boost (secondary mechanism)
    3. Tier is assigned based on topic score, with project floor

    Local filter modes (additive, existing path stays intact):
    - skip_llm + local_ranker: Use embedding scores instead of crude category heuristic
    - local_ranker + local_threshold > 0: Pre-filter, skip LLM for papers below threshold

    Args:
        papers: Papers to score
        config: Configuration
        llm_client: LLM client for topic scoring
        skip_llm: If True, use category-based proxy scores (degraded mode)
        local_ranker: Optional local ranker for embedding-based scoring
        local_threshold: If > 0, skip LLM for papers below this local score

    Returns:
        List of ScoredPaper objects, sorted by tier then score
    """
    prompt_template = load_scoring_prompt()
    scored_papers: list[ScoredPaper] = []

    # Batch-compute local scores if ranker is available
    local_scores_map: dict[str, float] = {}
    if local_ranker is not None:
        try:
            from local_scorer import arxiv_paper_to_record, local_score_to_llm_scale
            records = [arxiv_paper_to_record(p) for p in papers]
            local_results = local_ranker.score_papers(records)
            for paper, ls in zip(papers, local_results):
                local_scores_map[paper.arxiv_id] = ls.score_total
            print(f"Local scoring complete: {len(local_scores_map)} papers scored")
        except Exception as e:
            print(f"Warning: Local scoring failed, falling back: {e}")

    for i, paper in enumerate(papers):
        print(f"Scoring paper {i+1}/{len(papers)}: {paper.arxiv_id}")

        # Prefilter (simple category check)
        passed, matched_projects, primary_match, secondary_match = prefilter_paper(paper, config)
        project_match = bool(matched_projects)

        if not passed:
            scored_papers.append(ScoredPaper(
                paper=paper,
                score=0.0,
                tier=Tier.NOT_RELEVANT,
                prefilter_passed=False
            ))
            continue

        local_score_val = local_scores_map.get(paper.arxiv_id)

        # Score against TOPICS
        if skip_llm:
            if local_score_val is not None:
                # Local ranker available: use embedding-based score (replaces
                # the crude category heuristic)
                from local_scorer import local_score_to_llm_scale
                score = local_score_to_llm_scale(local_score_val)
                matched_topics = []
                reasoning = f"Local embedding score: {local_score_val:.4f} -> {score:.1f}/10"
            else:
                # Fallback: original category-based heuristic
                if primary_match:
                    score = 5.0
                else:
                    score = 3.5

                if project_match:
                    score += 1.5

                matched_topics = []
                reasoning = "LLM unavailable - category-based proxy score"
        elif local_threshold > 0 and local_score_val is not None and local_score_val < local_threshold:
            # Pre-filter mode: paper below local threshold, skip LLM call
            from local_scorer import local_score_to_llm_scale
            score = local_score_to_llm_scale(local_score_val)
            matched_topics = []
            reasoning = f"Below local threshold ({local_score_val:.4f} < {local_threshold}), LLM skipped"
        else:
            # Full mode: LLM evaluates topic relevance
            score, matched_topics, reasoning = score_paper_with_llm(
                paper, config, llm_client, prompt_template
            )

            # Project boost: small increase (+0.5) for tracked projects
            # This is a BOOSTER, not the primary score driver
            if project_match and score < 10.0:
                score = min(10.0, score + 0.5)
                reasoning += f" [Project boost: {', '.join(matched_projects)}]"

        # Project boost for skip_llm mode (applies to both local and category heuristic)
        if skip_llm and project_match and local_score_val is not None:
            if score < 10.0:
                score = min(10.0, score + 0.5)
                reasoning += f" [Project boost: {', '.join(matched_projects)}]"

        # Assign tier based on topic score (with project floor)
        tier = assign_tier(score, project_match, config)

        scored_papers.append(ScoredPaper(
            paper=paper,
            score=score,
            tier=tier,
            matched_topics=matched_topics,
            matched_projects=matched_projects,
            reasoning=reasoning,
            prefilter_passed=True,
            project_match=project_match,
            primary_category_match=primary_match
        ))

    # Sort by tier (most relevant first) then by score (highest first)
    tier_order = {
        Tier.MOST_RELEVANT: 0,
        Tier.SOMEWHAT_RELEVANT: 1,
        Tier.COULD_BE_INTERESTING: 2,
        Tier.NOT_RELEVANT: 3
    }
    scored_papers.sort(key=lambda p: (tier_order[p.tier], -p.score))

    return scored_papers


def filter_by_tier(
    papers: list[ScoredPaper],
    min_tier: Tier = Tier.COULD_BE_INTERESTING
) -> list[ScoredPaper]:
    """Filter papers to include only those at or above a minimum tier."""
    tier_order = {
        Tier.MOST_RELEVANT: 0,
        Tier.SOMEWHAT_RELEVANT: 1,
        Tier.COULD_BE_INTERESTING: 2,
        Tier.NOT_RELEVANT: 3
    }
    min_order = tier_order[min_tier]
    return [p for p in papers if tier_order[p.tier] <= min_order]


def group_by_tier(papers: list[ScoredPaper]) -> dict[Tier, list[ScoredPaper]]:
    """Group papers by tier."""
    groups: dict[Tier, list[ScoredPaper]] = {
        Tier.MOST_RELEVANT: [],
        Tier.SOMEWHAT_RELEVANT: [],
        Tier.COULD_BE_INTERESTING: [],
        Tier.NOT_RELEVANT: []
    }
    for paper in papers:
        groups[paper.tier].append(paper)
    return groups

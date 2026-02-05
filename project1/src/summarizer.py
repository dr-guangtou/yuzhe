"""Generate summaries for papers at different detail levels."""

from pathlib import Path

from arxiv_fetcher import ArxivPaper
from config import Config
from llm_client import LLMClient, retry_with_backoff
from scorer import ScoredPaper, Tier


def load_prompt_template(name: str) -> str:
    """Load a prompt template from the prompts directory.

    Args:
        name: Template name (without .md extension)

    Returns:
        Template content as string
    """
    prompt_path = Path(__file__).parent.parent / "prompts" / f"{name}.md"
    if not prompt_path.exists():
        raise FileNotFoundError(f"Prompt template not found: {prompt_path}")
    return prompt_path.read_text(encoding="utf-8")


def format_authors(authors: list[str], max_authors: int = 5) -> str:
    """Format author list for display.

    Args:
        authors: List of author names
        max_authors: Maximum number of authors to show

    Returns:
        Formatted author string
    """
    if not authors:
        return "Unknown authors"

    if len(authors) <= max_authors:
        return ", ".join(authors)

    return ", ".join(authors[:max_authors]) + f", et al. ({len(authors)} total)"


def generate_detailed_summary(
    paper: ArxivPaper,
    llm_client: LLMClient,
    config: Config
) -> str:
    """Generate a detailed summary for a highly relevant paper.

    Args:
        paper: The paper to summarize
        llm_client: LLM client instance
        config: Configuration

    Returns:
        Detailed summary as Markdown string
    """
    template = load_prompt_template("summary_detailed")

    prompt = template.format(
        title=paper.title,
        authors=format_authors(paper.authors),
        arxiv_id=paper.arxiv_id,
        categories=", ".join(paper.categories),
        abstract=paper.abstract
    )

    def call_llm():
        return llm_client.generate(prompt)

    try:
        response = retry_with_backoff(
            call_llm,
            max_attempts=config.api.llm_retry_attempts,
            initial_delay=config.api.llm_retry_delay_seconds
        )
        return response.content
    except Exception as e:
        print(f"Warning: Detailed summary failed for {paper.arxiv_id}: {e}")
        # Fallback to truncated abstract
        return fallback_summary(paper, detailed=True)


def generate_brief_summary(
    paper: ArxivPaper,
    llm_client: LLMClient,
    config: Config
) -> str:
    """Generate a brief 3-5 sentence summary.

    Args:
        paper: The paper to summarize
        llm_client: LLM client instance
        config: Configuration

    Returns:
        Brief summary as string
    """
    template = load_prompt_template("summary_brief")

    prompt = template.format(
        title=paper.title,
        authors=format_authors(paper.authors),
        arxiv_id=paper.arxiv_id,
        categories=", ".join(paper.categories),
        abstract=paper.abstract
    )

    def call_llm():
        return llm_client.generate(prompt)

    try:
        response = retry_with_backoff(
            call_llm,
            max_attempts=config.api.llm_retry_attempts,
            initial_delay=config.api.llm_retry_delay_seconds
        )
        return response.content
    except Exception as e:
        print(f"Warning: Brief summary failed for {paper.arxiv_id}: {e}")
        # Fallback to truncated abstract
        return fallback_summary(paper, detailed=False)


def fallback_summary(paper: ArxivPaper, detailed: bool = False) -> str:
    """Generate a fallback summary when LLM fails.

    Args:
        paper: The paper
        detailed: If True, include more abstract content

    Returns:
        Fallback summary string
    """
    if detailed:
        # For detailed summaries, provide structured fallback
        abstract_truncated = paper.abstract[:1500] + "..." if len(paper.abstract) > 1500 else paper.abstract
        return f"""### 1. ONE-SENTENCE SUMMARY
*Summary generation failed. See abstract below.*

### 2. KEY FINDINGS
*Unable to extract key findings automatically.*

### 3. KEY TECHNICAL DETAILS
*Please refer to the full paper for technical details.*

### 4. DATASETS AND TOOLS
*Not available.*

### 5. ABSTRACT (FALLBACK)
{abstract_truncated}
"""
    else:
        # For brief summaries, just use truncated abstract
        max_length = 500
        if len(paper.abstract) <= max_length:
            return paper.abstract
        return paper.abstract[:max_length].rsplit(' ', 1)[0] + "..."


def generate_summaries(
    scored_papers: list[ScoredPaper],
    llm_client: LLMClient,
    config: Config,
    skip_llm: bool = False
) -> dict[str, str]:
    """Generate summaries for all papers based on their tier.

    Args:
        scored_papers: Papers with scores and tiers
        llm_client: LLM client instance
        config: Configuration
        skip_llm: If True, use fallback summaries only

    Returns:
        Dictionary mapping arxiv_id to summary
    """
    summaries: dict[str, str] = {}

    for sp in scored_papers:
        paper = sp.paper
        print(f"Generating summary for {paper.arxiv_id} ({sp.tier.value})")

        if sp.tier == Tier.MOST_RELEVANT:
            if skip_llm:
                summaries[paper.arxiv_id] = fallback_summary(paper, detailed=True)
            else:
                summaries[paper.arxiv_id] = generate_detailed_summary(
                    paper, llm_client, config
                )

        elif sp.tier == Tier.SOMEWHAT_RELEVANT:
            if skip_llm:
                summaries[paper.arxiv_id] = fallback_summary(paper, detailed=False)
            else:
                summaries[paper.arxiv_id] = generate_brief_summary(
                    paper, llm_client, config
                )

        # COULD_BE_INTERESTING tier doesn't need a summary (title only)

    return summaries

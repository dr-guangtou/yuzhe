"""Format scored papers into Markdown digest."""

from datetime import datetime
from pathlib import Path

from arxiv_fetcher import ArxivPaper
from config import Config
from scorer import ScoredPaper, Tier, group_by_tier
from summarizer import format_authors


def format_paper_header(paper: ArxivPaper, score: float = None) -> str:
    """Format paper metadata as Markdown header.

    Args:
        paper: The paper
        score: Optional relevance score

    Returns:
        Formatted Markdown string
    """
    lines = []

    # Title with abs link + HTML link
    html_render_url = f"https://arxiv.org/html/{paper.arxiv_id}"
    lines.append(f"### [{paper.title}](https://arxiv.org/abs/{paper.arxiv_id}) [HTML]({html_render_url})")
    lines.append("")

    # Metadata (compact)
    lines.append(f"**Authors:** {format_authors(paper.authors, max_authors=8)}")

    if score is not None:
        lines.append(f"**Relevance Score:** {score:.1f}/10")

    lines.append("")

    return "\n".join(lines)


def format_paper_title_only(paper: ArxivPaper, index: int) -> str:
    """Format paper as title-only entry for "Could Be Interesting" tier.

    Args:
        paper: The paper
        index: Paper index in the list

    Returns:
        Formatted Markdown string
    """
    return f"{index}. [{paper.title}](https://arxiv.org/abs/{paper.arxiv_id}) — *{paper.primary_category}*"


def format_digest(
    scored_papers: list[ScoredPaper],
    summaries: dict[str, str],
    config: Config,
    date: datetime = None
) -> str:
    """Format the complete daily digest.

    Args:
        scored_papers: Papers with scores and tiers
        summaries: Dictionary mapping arxiv_id to summary text
        config: Configuration
        date: Date for the digest (default: today)

    Returns:
        Complete Markdown digest
    """
    if date is None:
        date = datetime.now()

    date_str = date.strftime("%Y-%m-%d")
    date_long = date.strftime("%A, %B %d, %Y")

    groups = group_by_tier(scored_papers)

    for tier in groups:
        groups[tier].sort(key=lambda sp: -sp.score)

    lines = []

    # Header
    lines.append(f"# arXiv Daily Digest: {date_str}")
    lines.append("")
    lines.append(f"*Generated on {date_long}*")
    lines.append("")

    # Summary statistics
    lines.append("## Summary")
    lines.append("")
    lines.append(f"- **Most Relevant:** {len(groups[Tier.MOST_RELEVANT])} papers")
    lines.append(f"- **Somewhat Relevant:** {len(groups[Tier.SOMEWHAT_RELEVANT])} papers")
    lines.append(f"- **Could Be Interesting:** {len(groups[Tier.COULD_BE_INTERESTING])} papers")
    lines.append(f"- **Total Processed:** {len(scored_papers)} papers")
    lines.append("")

    # Categories monitored
    lines.append("### Categories Monitored")
    lines.append(f"- **Primary:** {', '.join(config.category.primary)}")
    lines.append(f"- **Secondary:** {', '.join(config.category.secondary)}")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Most Relevant
    most_relevant = groups[Tier.MOST_RELEVANT]
    if most_relevant:
        lines.append("## Most Relevant Papers")
        lines.append("")
        lines.append("*Detailed summaries of papers most aligned with your research interests.*")
        lines.append("")

        for sp in most_relevant:
            paper = sp.paper
            lines.append(format_paper_header(paper, sp.score))

            # Add summary
            summary = summaries.get(paper.arxiv_id, "")
            if summary:
                lines.append(summary)
            else:
                lines.append(paper.abstract)

            lines.append("")
            lines.append("---")
            lines.append("")

    # Somewhat Relevant
    somewhat_relevant = groups[Tier.SOMEWHAT_RELEVANT]
    if somewhat_relevant:
        lines.append("## Somewhat Relevant Papers")
        lines.append("")
        lines.append("*Brief summaries of papers related to your research interests.*")
        lines.append("")

        for sp in somewhat_relevant:
            paper = sp.paper
            lines.append(format_paper_header(paper, sp.score))

            # Add summary
            summary = summaries.get(paper.arxiv_id, "")
            if summary:
                lines.append(summary)
            else:
                lines.append(paper.abstract)

            lines.append("")
            lines.append("---")
            lines.append("")

    # Could Be Interesting
    could_be_interesting = groups[Tier.COULD_BE_INTERESTING]
    if could_be_interesting:
        lines.append("## Could Be Interesting")
        lines.append("")
        lines.append("*Papers that may be of peripheral interest.*")
        lines.append("")

        for i, sp in enumerate(could_be_interesting, 1):
            lines.append(format_paper_title_only(sp.paper, i))

        lines.append("")
        lines.append("---")
        lines.append("")

    # Footer
    lines.append("## About This Digest")
    lines.append("")
    lines.append("This digest was automatically generated by the Daily arXiv Summary tool.")
    lines.append(f"Configuration includes {len(config.topics.primary)} primary topics and {len(config.projects)} tracked projects.")
    lines.append("")

    return "\n".join(lines)


def save_digest(
    content: str,
    config: Config,
    date: datetime = None
) -> Path:
    """Save the digest to the archive directory.

    Args:
        content: Markdown content
        config: Configuration
        date: Date for the filename (default: today)

    Returns:
        Path to the saved file
    """
    if date is None:
        date = datetime.now()

    # Determine output path
    year = date.year
    date_str = date.strftime("%Y-%m-%d")

    if config.config_path:
        base_dir = config.config_path.parent
    else:
        base_dir = Path(".")

    output_dir = base_dir / config.output.digest_dir / config.output.archive_subdir / str(year)
    output_dir.mkdir(parents=True, exist_ok=True)

    output_path = output_dir / f"{date_str}.md"

    # Write atomically
    temp_path = output_path.with_suffix(".tmp")
    try:
        temp_path.write_text(content, encoding="utf-8")
        temp_path.rename(output_path)
    except Exception:
        if temp_path.exists():
            temp_path.unlink()
        raise

    return output_path


def copy_to_obsidian(
    content: str,
    config: Config,
    date: datetime = None
) -> Path | None:
    """Copy the digest to Obsidian vault if configured.

    Args:
        content: Markdown content
        config: Configuration
        date: Date for the filename (default: today)

    Returns:
        Path to the copied file, or None if not configured
    """
    if not config.output.obsidian_vault:
        return None

    if date is None:
        date = datetime.now()

    vault_path = Path(config.output.obsidian_vault).expanduser()
    if not vault_path.exists():
        print(f"Warning: Obsidian vault not found: {vault_path}")
        return None

    date_str = date.strftime("%Y-%m-%d")
    output_path = vault_path / f"arXiv-{date_str}.md"

    output_path.write_text(content, encoding="utf-8")
    return output_path

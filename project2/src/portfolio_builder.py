"""Generates per-paper Markdown portfolio documents."""

import logging
import re
from pathlib import Path

from orcid_fetcher import Publication

logger = logging.getLogger(__name__)


class PortfolioBuilder:
    """Builds per-paper Markdown documents with summaries."""

    def __init__(self, portfolio_dir: Path, user_info: dict):
        self.portfolio_dir = portfolio_dir
        self.user_info = user_info

    def generate_slug(self, pub: Publication) -> str:
        """Generate a filename slug from publication metadata.

        Format: {year}_{last_name}_{short_title}
        """
        # Extract first author last name
        if pub.authors:
            first_author = pub.authors[0]
            # Handle "Last, First" format
            last_name = first_author.split(",")[0].strip()
        else:
            last_name = "unknown"

        # Shorten title: take first 5 meaningful words
        stop_words = {"the", "a", "an", "of", "in", "on", "for", "and", "with", "to", "from"}
        words = re.sub(r"[^\w\s]", "", pub.title.lower()).split()
        meaningful = [w for w in words if w not in stop_words][:5]
        short_title = "_".join(meaningful)

        # Clean for filesystem
        last_name = re.sub(r"[^\w]", "", last_name.lower())
        short_title = re.sub(r"[^\w]", "_", short_title)

        return f"{pub.year}_{last_name}_{short_title}"

    def build_document(
        self, pub: Publication, summaries: dict[str, str]
    ) -> str:
        """Build a Markdown document for a single paper."""
        first_name = self.user_info.get("first_name", "")
        last_name = self.user_info.get("last_name", "")
        chinese_name = self.user_info.get("chinese_name", "")
        slug = self.generate_slug(pub)

        lines = [
            f"# {pub.title}",
            "",
        ]

        # Author line
        if pub.authors:
            author_str = ", ".join(pub.authors[:10])
            if len(pub.authors) > 10:
                author_str += f" (+{len(pub.authors) - 10} more)"
            lines.append(f"**{first_name} {last_name} ({chinese_name})** and collaborators")
            lines.append(f"")
            lines.append(f"*Full author list:* {author_str}")
        else:
            lines.append(f"**{first_name} {last_name} ({chinese_name})**")

        lines.append("")

        # Citation info
        if pub.journal:
            lines.append(f"*{pub.journal}* ({pub.year})")
        else:
            lines.append(f"Preprint ({pub.year})")
        lines.append("")

        # Links
        link_parts = []
        if pub.doi:
            link_parts.append(f"[DOI](https://doi.org/{pub.doi})")
        if pub.arxiv_id:
            link_parts.append(f"[arXiv](https://arxiv.org/abs/{pub.arxiv_id})")
        if pub.ads_url:
            link_parts.append(f"[ADS]({pub.ads_url})")
        if pub.pdf_url:
            link_parts.append(f"[PDF]({pub.pdf_url})")
        if link_parts:
            lines.append(" | ".join(link_parts))
            lines.append("")

        if pub.citation_count > 0:
            lines.append(f"**Citations:** {pub.citation_count}")
            lines.append("")

        # Short summary
        lines.append("---")
        lines.append("")
        lines.append("## Short Summary")
        lines.append("")
        if summaries.get("short_eng"):
            lines.append(summaries["short_eng"])
            lines.append("")
            if summaries.get("short_chn"):
                lines.append(f"**中文：** {summaries['short_chn']}")
                lines.append("")
        else:
            lines.append("*Summary pending.*")
            lines.append("")

        # Detailed summary
        lines.append("## Detailed Summary")
        lines.append("")
        if summaries.get("long_eng"):
            lines.append(summaries["long_eng"])
            lines.append("")
            if summaries.get("long_chn"):
                lines.append("### 中文版")
                lines.append("")
                lines.append(summaries["long_chn"])
                lines.append("")
        else:
            lines.append("*Summary pending.*")
            lines.append("")

        # Figure placeholder
        lines.append("## Key Figure")
        lines.append("")
        lines.append(
            f"*To be added in future version.* See `{slug}_figures/` directory."
        )
        lines.append("")

        return "\n".join(lines)

    def build_all(
        self,
        publications: list[Publication],
        all_summaries: dict[str, dict[str, str]],
    ) -> None:
        """Build portfolio documents for all publications."""
        self.portfolio_dir.mkdir(parents=True, exist_ok=True)

        for pub in publications:
            slug = self.generate_slug(pub)
            pub_key = pub.doi or pub.title
            summaries = all_summaries.get(pub_key, {})

            # Write Markdown document
            doc = self.build_document(pub, summaries)
            md_path = self.portfolio_dir / f"{slug}.md"
            md_path.write_text(doc, encoding="utf-8")

            # Create figure placeholder directory
            fig_dir = self.portfolio_dir / f"{slug}_figures"
            fig_dir.mkdir(exist_ok=True)
            gitkeep = fig_dir / ".gitkeep"
            if not gitkeep.exists():
                gitkeep.touch()

            logger.info("Built portfolio: %s", slug)

        logger.info("Built %d portfolio documents", len(publications))

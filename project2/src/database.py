"""Publication YAML store with merge and update detection."""

import logging
from dataclasses import asdict
from pathlib import Path

import yaml

from orcid_fetcher import Publication

logger = logging.getLogger(__name__)


class PublicationDatabase:
    """YAML-based publication store."""

    def __init__(self, yaml_path: Path):
        self.yaml_path = yaml_path

    def load(self) -> list[Publication]:
        """Load publications from YAML file."""
        if not self.yaml_path.exists():
            return []

        with open(self.yaml_path, "r", encoding="utf-8") as f:
            data = yaml.safe_load(f)

        if not data or not isinstance(data, list):
            return []

        publications = []
        for entry in data:
            try:
                publications.append(Publication(**entry))
            except (TypeError, KeyError) as e:
                logger.warning("Skipping malformed entry: %s", e)
        return publications

    def save(self, publications: list[Publication]):
        """Save publications to YAML file (atomic write)."""
        self.yaml_path.parent.mkdir(parents=True, exist_ok=True)
        temp_path = self.yaml_path.with_suffix(".tmp")

        data = [asdict(pub) for pub in publications]
        try:
            with open(temp_path, "w", encoding="utf-8") as f:
                yaml.dump(
                    data, f,
                    default_flow_style=False,
                    allow_unicode=True,
                    sort_keys=False,
                )
            temp_path.rename(self.yaml_path)
        except Exception:
            if temp_path.exists():
                temp_path.unlink()
            raise

    def merge(
        self, new_pubs: list[Publication]
    ) -> tuple[list[Publication], list[Publication]]:
        """Merge new publications into the database.

        Matches by DOI (primary) or normalized title (fallback).
        Updates metadata for existing entries, adds new ones.

        Returns:
            (all_publications, newly_added_publications)
        """
        existing = self.load()

        # Build lookup indices
        doi_index: dict[str, int] = {}
        title_index: dict[str, int] = {}
        for i, pub in enumerate(existing):
            if pub.doi:
                doi_index[pub.doi.lower()] = i
            title_index[pub.title.lower().strip()] = i

        newly_added = []

        for new_pub in new_pubs:
            match_idx = None

            # Try DOI match first
            if new_pub.doi:
                match_idx = doi_index.get(new_pub.doi.lower())

            # Fallback to title match
            if match_idx is None:
                match_idx = title_index.get(new_pub.title.lower().strip())

            if match_idx is not None:
                # Update existing entry with richer metadata
                old = existing[match_idx]
                old.abstract = new_pub.abstract or old.abstract
                old.authors = new_pub.authors or old.authors
                old.bibcode = new_pub.bibcode or old.bibcode
                old.citation_count = max(
                    new_pub.citation_count, old.citation_count
                )
                old.ads_url = new_pub.ads_url or old.ads_url
                old.pdf_url = new_pub.pdf_url or old.pdf_url
                old.doi = new_pub.doi or old.doi
                old.arxiv_id = new_pub.arxiv_id or old.arxiv_id
                old.journal = new_pub.journal or old.journal
            else:
                existing.append(new_pub)
                newly_added.append(new_pub)
                # Update indices
                idx = len(existing) - 1
                if new_pub.doi:
                    doi_index[new_pub.doi.lower()] = idx
                title_index[new_pub.title.lower().strip()] = idx

        # Sort by year descending
        existing.sort(key=lambda p: p.year, reverse=True)

        logger.info(
            "Merge result: %d total, %d newly added",
            len(existing),
            len(newly_added),
        )
        return existing, newly_added

    def generate_markdown_list(self, publications: list[Publication]) -> None:
        """Write publication_list.md grouped by year."""
        md_path = self.yaml_path.parent / "publication_list.md"

        lines = [
            "# Publication List",
            "",
            f"Total: {len(publications)} publications",
            "",
        ]

        current_year = None
        for pub in publications:
            if pub.year != current_year:
                current_year = pub.year
                lines.extend(["---", "", f"## {current_year}", ""])

            lines.append(f"### {pub.title}")
            lines.append("")

            if pub.authors:
                author_str = ", ".join(pub.authors[:5])
                if len(pub.authors) > 5:
                    author_str += f" (+{len(pub.authors) - 5} more)"
                lines.append(f"**Authors:** {author_str}")

            if pub.journal:
                lines.append(f"*{pub.journal}* ({pub.year})")

            links = []
            if pub.doi:
                links.append(
                    f"[DOI](https://doi.org/{pub.doi})"
                )
            if pub.arxiv_id:
                links.append(
                    f"[arXiv](https://arxiv.org/abs/{pub.arxiv_id})"
                )
            if pub.ads_url:
                links.append(f"[ADS]({pub.ads_url})")
            if links:
                lines.append(" | ".join(links))

            if pub.citation_count > 0:
                lines.append(f"Citations: {pub.citation_count}")

            lines.append("")

        md_path.write_text("\n".join(lines), encoding="utf-8")
        logger.info("Wrote publication list to %s", md_path)

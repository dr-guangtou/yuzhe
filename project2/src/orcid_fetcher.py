"""ORCID API client for fetching publication lists."""

import logging
import xml.etree.ElementTree as ET
import urllib.request
from dataclasses import dataclass, field

logger = logging.getLogger(__name__)

NAMESPACES = {
    "work": "http://www.orcid.org/ns/work",
    "common": "http://www.orcid.org/ns/common",
    "activities": "http://www.orcid.org/ns/activities",
}


@dataclass
class Publication:
    """A single publication record."""
    title: str
    year: int
    journal: str
    doi: str
    arxiv_id: str
    abstract: str = ""
    authors: list[str] = field(default_factory=list)
    bibcode: str = ""
    citation_count: int = 0
    ads_url: str = ""
    pdf_url: str = ""


class ORCIDFetcher:
    """Fetches publication list from ORCID public API."""

    def __init__(self, orcid_id: str, year_cutoff: int, timeout: int = 30):
        self.orcid_id = orcid_id
        self.year_cutoff = year_cutoff
        self.timeout = timeout

    def fetch_works(self) -> str:
        """Fetch raw XML of all works from ORCID."""
        url = f"https://pub.orcid.org/v3.0/{self.orcid_id}/works"
        req = urllib.request.Request(url, headers={"Accept": "application/xml"})
        logger.info("Fetching ORCID works for %s", self.orcid_id)

        with urllib.request.urlopen(req, timeout=self.timeout) as response:
            return response.read().decode("utf-8")

    def parse_publications(self, xml_data: str) -> list[Publication]:
        """Parse publications from ORCID XML, filtered by year_cutoff."""
        root = ET.fromstring(xml_data)
        publications = []
        seen_titles = set()

        for group in root.findall(".//activities:group", NAMESPACES):
            # Each group may have multiple work-summary elements (duplicates from
            # different sources). Take the first one with the most metadata.
            best_pub = None

            for work_summary in group.findall(
                ".//work:work-summary", NAMESPACES
            ):
                title_elem = work_summary.find(
                    ".//work:title/common:title", NAMESPACES
                )
                title = title_elem.text.strip() if title_elem is not None else ""
                if not title:
                    continue

                year_elem = work_summary.find(
                    ".//common:publication-date/common:year", NAMESPACES
                )
                year = int(year_elem.text) if year_elem is not None else 0

                if year < self.year_cutoff:
                    continue

                journal_elem = work_summary.find(
                    ".//work:journal-title", NAMESPACES
                )
                journal = (
                    journal_elem.text.strip()
                    if journal_elem is not None
                    else ""
                )

                doi = ""
                arxiv_id = ""
                for ext_id in work_summary.findall(
                    ".//common:external-id", NAMESPACES
                ):
                    type_elem = ext_id.find(
                        "common:external-id-type", NAMESPACES
                    )
                    value_elem = ext_id.find(
                        "common:external-id-value", NAMESPACES
                    )
                    if type_elem is None or value_elem is None:
                        continue
                    id_type = type_elem.text.lower()
                    if id_type == "doi":
                        doi = value_elem.text.strip()
                    elif id_type == "arxiv":
                        arxiv_id = value_elem.text.strip()

                pub = Publication(
                    title=title,
                    year=year,
                    journal=journal,
                    doi=doi,
                    arxiv_id=arxiv_id,
                )

                # Prefer the entry with more identifiers
                if best_pub is None:
                    best_pub = pub
                elif (bool(pub.doi) + bool(pub.arxiv_id)) > (
                    bool(best_pub.doi) + bool(best_pub.arxiv_id)
                ):
                    best_pub = pub

            if best_pub is not None:
                # Deduplicate by normalized title
                norm_title = best_pub.title.lower().strip()
                if norm_title not in seen_titles:
                    seen_titles.add(norm_title)
                    publications.append(best_pub)

        publications.sort(key=lambda p: p.year, reverse=True)
        logger.info(
            "Parsed %d publications (year >= %d)",
            len(publications),
            self.year_cutoff,
        )
        return publications

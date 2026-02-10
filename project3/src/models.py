"""Data models for Periodic Journal Summary."""

from dataclasses import dataclass, field


@dataclass
class JournalArticle:
    """A published article parsed from a journal RSS feed."""

    # Identity
    title: str
    doi: str
    journal: str  # acronym: "apj", "mnras", etc.

    # Content
    abstract: str

    # Metadata (may be incomplete depending on publisher)
    authors: list[str] = field(default_factory=list)
    url: str = ""
    published: str = ""  # ISO 8601
    volume: str = ""
    issue: str = ""

    # Provenance
    feed_url: str = ""
    fetched_at: str = ""  # ISO 8601
    abstract_enriched: bool = False

    # Cross-reference
    arxiv_id: str = ""


@dataclass
class JournalFeedConfig:
    """Configuration for a single journal RSS feed."""

    name: str       # "The Astrophysical Journal"
    acronym: str    # "apj"
    publisher: str  # "iop" | "oup" | "edp" | "nature" | "oja"
    feed_url: str
    feed_url_alt: str = ""
    enabled: bool = True

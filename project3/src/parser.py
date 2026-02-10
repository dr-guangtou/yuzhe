"""RSS feed parser with per-publisher normalizers.

Uses feedparser to handle RSS/RDF/Atom formats uniformly.
Per-publisher normalizer functions handle DOI extraction, author
format, and abstract location differences.
"""

import html
import re
from datetime import datetime

import feedparser

from models import JournalArticle, JournalFeedConfig


def parse_feed(content: bytes, journal: JournalFeedConfig) -> list[JournalArticle]:
    """Parse an RSS feed into JournalArticle objects.

    Dispatches to per-publisher normalizers for field extraction.

    Args:
        content: Raw bytes of the RSS feed.
        journal: Configuration for this journal.

    Returns:
        List of JournalArticle objects.
    """
    feed = feedparser.parse(content)
    normalizer = _NORMALIZERS.get(journal.publisher, _normalize_generic)

    articles = []
    now = datetime.now().isoformat()

    for entry in feed.entries:
        article = normalizer(entry, journal)
        if article is None:
            continue
        article.feed_url = journal.feed_url
        article.fetched_at = now
        articles.append(article)

    return articles


def _clean_html(text: str) -> str:
    """Strip HTML tags and decode entities."""
    text = re.sub(r"<[^>]+>", "", text)
    text = html.unescape(text)
    return text.strip()


def _extract_doi_from_link(link: str) -> str:
    """Extract DOI from a URL like https://doi.org/10.xxx/yyy."""
    match = re.search(r"(10\.\d{4,}/[^\s]+)", link)
    return match.group(1).rstrip("/") if match else ""


def _normalize_iop(entry: feedparser.FeedParserDict, journal: JournalFeedConfig) -> JournalArticle | None:
    """Normalize IOP Science entries (ApJ, AJ, ApJL, ApJS, PASP, JCAP).

    IOP feeds provide: title, authors, full abstract in description,
    DOI link, publication date.
    """
    title = _clean_html(entry.get("title", ""))
    if not title:
        return None

    link = entry.get("link", "")
    doi = _extract_doi_from_link(link)

    # IOP uses prism:doi or dc:identifier for DOI
    if not doi:
        doi = entry.get("prism_doi", "")
    if not doi:
        dc_id = entry.get("dc_identifier", "")
        if dc_id.startswith("doi:"):
            doi = dc_id[4:]

    abstract = _clean_html(entry.get("description", ""))

    # Authors: IOP uses dc:creator or author_detail
    authors = []
    if "authors" in entry:
        authors = [a.get("name", "") for a in entry["authors"] if a.get("name")]
    elif "author" in entry:
        authors = [entry["author"]]

    published = ""
    if entry.get("published_parsed"):
        try:
            published = datetime(*entry.published_parsed[:6]).isoformat()
        except (TypeError, ValueError):
            pass

    volume = entry.get("prism_volume", "")
    issue = entry.get("prism_number", "")

    return JournalArticle(
        title=title,
        doi=doi,
        journal=journal.acronym,
        abstract=abstract,
        authors=authors,
        url=link,
        published=published,
        volume=str(volume),
        issue=str(issue),
    )


def _normalize_oup(entry: feedparser.FeedParserDict, journal: JournalFeedConfig) -> JournalArticle | None:
    """Normalize OUP entries (MNRAS, MNRASL, PASJ).

    OUP feeds provide: title, abstract in description, DOI link.
    Authors are typically MISSING from the RSS feed.
    """
    title = _clean_html(entry.get("title", ""))
    if not title:
        return None

    link = entry.get("link", "")
    doi = _extract_doi_from_link(link)

    abstract = _clean_html(entry.get("description", ""))

    published = ""
    if entry.get("published_parsed"):
        try:
            published = datetime(*entry.published_parsed[:6]).isoformat()
        except (TypeError, ValueError):
            pass

    return JournalArticle(
        title=title,
        doi=doi,
        journal=journal.acronym,
        abstract=abstract,
        authors=[],  # OUP RSS does not include authors
        url=link,
        published=published,
    )


def _normalize_edp(entry: feedparser.FeedParserDict, journal: JournalFeedConfig) -> JournalArticle | None:
    """Normalize EDP Sciences entries (A&A).

    A&A feeds provide: title, authors, brief abstract in description.
    Full structured abstract needs enrichment from HTML page.
    """
    title = _clean_html(entry.get("title", ""))
    if not title:
        return None

    link = entry.get("link", "")
    doi = _extract_doi_from_link(link)

    abstract = _clean_html(entry.get("description", ""))

    authors = []
    if "authors" in entry:
        authors = [a.get("name", "") for a in entry["authors"] if a.get("name")]
    elif "author" in entry:
        authors = [entry["author"]]

    published = ""
    if entry.get("published_parsed"):
        try:
            published = datetime(*entry.published_parsed[:6]).isoformat()
        except (TypeError, ValueError):
            pass

    return JournalArticle(
        title=title,
        doi=doi,
        journal=journal.acronym,
        abstract=abstract,
        authors=authors,
        url=link,
        published=published,
    )


def _normalize_nature(entry: feedparser.FeedParserDict, journal: JournalFeedConfig) -> JournalArticle | None:
    """Normalize Nature entries (Nature Astronomy).

    Nature feeds provide: title, authors, full abstract in
    content:encoded or description.
    """
    title = _clean_html(entry.get("title", ""))
    if not title:
        return None

    link = entry.get("link", "")
    doi = _extract_doi_from_link(link)

    # Nature sometimes puts the full abstract in content[0]
    abstract = ""
    if entry.get("content"):
        abstract = _clean_html(entry["content"][0].get("value", ""))
    if not abstract:
        abstract = _clean_html(entry.get("description", ""))

    authors = []
    if "authors" in entry:
        authors = [a.get("name", "") for a in entry["authors"] if a.get("name")]
    elif "author" in entry:
        authors = [entry["author"]]

    published = ""
    if entry.get("published_parsed"):
        try:
            published = datetime(*entry.published_parsed[:6]).isoformat()
        except (TypeError, ValueError):
            pass

    return JournalArticle(
        title=title,
        doi=doi,
        journal=journal.acronym,
        abstract=abstract,
        authors=authors,
        url=link,
        published=published,
    )


def _normalize_oja(entry: feedparser.FeedParserDict, journal: JournalFeedConfig) -> JournalArticle | None:
    """Normalize Open Journal of Astrophysics entries.

    OJA feeds provide: title, brief abstract, DOI link.
    Authors may be missing. Full abstract needs enrichment via arXiv.
    """
    title = _clean_html(entry.get("title", ""))
    if not title:
        return None

    link = entry.get("link", "")
    doi = _extract_doi_from_link(link)

    abstract = _clean_html(entry.get("description", ""))

    authors = []
    if "authors" in entry:
        authors = [a.get("name", "") for a in entry["authors"] if a.get("name")]
    elif "author" in entry:
        authors = [entry["author"]]

    published = ""
    if entry.get("published_parsed"):
        try:
            published = datetime(*entry.published_parsed[:6]).isoformat()
        except (TypeError, ValueError):
            pass

    return JournalArticle(
        title=title,
        doi=doi,
        journal=journal.acronym,
        abstract=abstract,
        authors=authors,
        url=link,
        published=published,
    )


def _normalize_generic(entry: feedparser.FeedParserDict, journal: JournalFeedConfig) -> JournalArticle | None:
    """Fallback normalizer for unknown publishers."""
    title = _clean_html(entry.get("title", ""))
    if not title:
        return None

    link = entry.get("link", "")
    doi = _extract_doi_from_link(link)
    abstract = _clean_html(entry.get("description", ""))

    return JournalArticle(
        title=title,
        doi=doi,
        journal=journal.acronym,
        abstract=abstract,
        url=link,
    )


# Publisher -> normalizer dispatch table
_NORMALIZERS = {
    "iop": _normalize_iop,
    "oup": _normalize_oup,
    "edp": _normalize_edp,
    "nature": _normalize_nature,
    "oja": _normalize_oja,
}

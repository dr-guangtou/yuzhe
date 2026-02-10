"""Fetch papers from arXiv API and RSS feeds."""

import re
import time
import urllib.request
import urllib.parse
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import Optional

import feedparser


# arXiv Atom XML namespaces
ATOM_NS = "http://www.w3.org/2005/Atom"
ARXIV_NS = "http://arxiv.org/schemas/atom"

ARXIV_API_BASE = "https://export.arxiv.org/api/query"
USER_AGENT = "daily-arxiv-summary/1.0 (https://github.com/dr-guangtou/yuzhe)"


@dataclass
class ArxivPaper:
    """Represents an arXiv paper."""
    arxiv_id: str
    title: str
    authors: list[str]
    abstract: str
    categories: list[str]
    primary_category: str
    pdf_url: str
    html_url: str
    published: datetime
    updated: datetime

    def __hash__(self):
        return hash(self.arxiv_id)

    def __eq__(self, other):
        if isinstance(other, ArxivPaper):
            return self.arxiv_id == other.arxiv_id
        return False


def extract_arxiv_id(url_or_id: str) -> str:
    """Extract arXiv ID from URL or return ID if already clean.

    Examples:
        http://arxiv.org/abs/2602.12345v1 -> 2602.12345
        2602.12345v2 -> 2602.12345
    """
    # Remove version suffix
    clean = re.sub(r'v\d+$', '', url_or_id)
    # Extract ID from URL
    match = re.search(r'(\d{4}\.\d{4,5})', clean)
    if match:
        return match.group(1)
    # Try old format: cond-mat/0001234
    match = re.search(r'([a-z-]+/\d{7})', clean)
    if match:
        return match.group(1)
    return clean


def clean_text(text: str) -> str:
    """Clean text by normalizing whitespace."""
    return " ".join(text.split())


def parse_datetime(date_str: str) -> datetime:
    """Parse arXiv datetime string."""
    # Format: 2026-02-05T00:00:00Z
    return datetime.fromisoformat(date_str.replace("Z", "+00:00"))


def parse_entry(entry: ET.Element) -> Optional[ArxivPaper]:
    """Parse a single Atom entry into ArxivPaper."""
    try:
        # Get ID
        id_elem = entry.find(f"{{{ATOM_NS}}}id")
        if id_elem is None or id_elem.text is None:
            return None
        arxiv_id = extract_arxiv_id(id_elem.text)

        # Get title
        title_elem = entry.find(f"{{{ATOM_NS}}}title")
        title = clean_text(title_elem.text) if title_elem is not None and title_elem.text else ""

        # Get abstract
        summary_elem = entry.find(f"{{{ATOM_NS}}}summary")
        abstract = clean_text(summary_elem.text) if summary_elem is not None and summary_elem.text else ""

        # Get authors
        authors = []
        for author in entry.findall(f"{{{ATOM_NS}}}author"):
            name_elem = author.find(f"{{{ATOM_NS}}}name")
            if name_elem is not None and name_elem.text:
                authors.append(name_elem.text)

        # Get categories
        categories = []
        for cat in entry.findall(f"{{{ATOM_NS}}}category"):
            term = cat.get("term")
            if term:
                categories.append(term)

        # Get primary category
        primary_cat_elem = entry.find(f"{{{ARXIV_NS}}}primary_category")
        primary_category = ""
        if primary_cat_elem is not None:
            primary_category = primary_cat_elem.get("term", "")

        # Get links
        pdf_url = ""
        html_url = ""
        for link in entry.findall(f"{{{ATOM_NS}}}link"):
            rel = link.get("rel", "")
            href = link.get("href", "")
            title_attr = link.get("title", "")

            if title_attr == "pdf":
                pdf_url = href
            elif rel == "alternate":
                html_url = href

        # Get dates
        published_elem = entry.find(f"{{{ATOM_NS}}}published")
        updated_elem = entry.find(f"{{{ATOM_NS}}}updated")

        published = datetime.now()
        updated = datetime.now()
        if published_elem is not None and published_elem.text:
            published = parse_datetime(published_elem.text)
        if updated_elem is not None and updated_elem.text:
            updated = parse_datetime(updated_elem.text)

        return ArxivPaper(
            arxiv_id=arxiv_id,
            title=title,
            authors=authors,
            abstract=abstract,
            categories=categories,
            primary_category=primary_category,
            pdf_url=pdf_url,
            html_url=html_url,
            published=published,
            updated=updated
        )
    except Exception as e:
        print(f"Warning: Failed to parse entry: {e}")
        return None


def fetch_papers_by_category(
    category: str,
    max_results: int = 100,
    delay_seconds: float = 3.0
) -> list[ArxivPaper]:
    """Fetch recent papers from a single arXiv category.

    Args:
        category: arXiv category (e.g., "astro-ph.CO")
        max_results: Maximum number of papers to fetch
        delay_seconds: Delay before the request (for rate limiting)

    Returns:
        List of ArxivPaper objects
    """
    # Build query URL
    query = f"cat:{category}"
    params = {
        "search_query": query,
        "sortBy": "submittedDate",
        "sortOrder": "descending",
        "start": 0,
        "max_results": max_results
    }
    url = f"{ARXIV_API_BASE}?{urllib.parse.urlencode(params)}"

    # Rate limiting: arXiv API requires minimum 3 seconds between requests
    # See: ~/.claude/skills/arxiv-public-api/references/terms_of_use.md
    if delay_seconds > 0:
        time.sleep(delay_seconds)

    # Make request
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            xml_data = response.read()
    except Exception as e:
        print(f"Error fetching {category}: {e}")
        return []

    # Parse XML
    try:
        root = ET.fromstring(xml_data)
    except ET.ParseError as e:
        print(f"Error parsing XML for {category}: {e}")
        return []

    # Extract papers
    papers = []
    for entry in root.findall(f"{{{ATOM_NS}}}entry"):
        paper = parse_entry(entry)
        if paper:
            papers.append(paper)

    return papers


def fetch_papers(
    categories: list[str],
    max_results_per_category: int = 100,
    delay_seconds: float = 3.0,
    days: int = 1,
    since_date: Optional[datetime] = None
) -> list[ArxivPaper]:
    """Fetch papers from multiple arXiv categories.

    Args:
        categories: List of arXiv categories
        max_results_per_category: Max papers per category
        delay_seconds: Delay between requests (arXiv rate limit)
        days: Filter papers from the last N days (used when since_date is None)
        since_date: Explicit cutoff date; when provided, overrides days-based calculation

    Returns:
        Deduplicated list of ArxivPaper objects, sorted by published date
    """
    if since_date is not None:
        cutoff_date = since_date
    else:
        cutoff_date = datetime.now() - timedelta(days=days)

    # Fetch from all categories
    category_set = set(categories)
    seen_ids: set[str] = set()
    all_papers: list[ArxivPaper] = []

    for i, category in enumerate(categories):
        # Skip delay for first request
        delay = delay_seconds if i > 0 else 0
        papers = fetch_papers_by_category(
            category=category,
            max_results=max_results_per_category,
            delay_seconds=delay
        )

        # Deduplicate, filter by date, and reject cross-listings
        for paper in papers:
            if paper.arxiv_id not in seen_ids:
                seen_ids.add(paper.arxiv_id)
                if paper.primary_category not in category_set:
                    continue
                if paper.updated.replace(tzinfo=None) >= cutoff_date:
                    all_papers.append(paper)

        print(f"Fetched {len(papers)} papers from {category}, {len(all_papers)} total unique")

    # Sort by published date (newest first)
    all_papers.sort(key=lambda p: p.published, reverse=True)

    return all_papers


def search_papers(
    query: str,
    max_results: int = 50,
    delay_seconds: float = 3.0
) -> list[ArxivPaper]:
    """Search arXiv with a free-form query.

    Args:
        query: Search query (e.g., "ti:galaxy AND au:Huang")
        max_results: Maximum number of results
        delay_seconds: Delay before request

    Returns:
        List of ArxivPaper objects
    """
    params = {
        "search_query": query,
        "sortBy": "relevance",
        "sortOrder": "descending",
        "start": 0,
        "max_results": max_results
    }
    url = f"{ARXIV_API_BASE}?{urllib.parse.urlencode(params)}"

    if delay_seconds > 0:
        time.sleep(delay_seconds)

    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            xml_data = response.read()
    except Exception as e:
        print(f"Error searching arXiv: {e}")
        return []

    try:
        root = ET.fromstring(xml_data)
    except ET.ParseError as e:
        print(f"Error parsing search results: {e}")
        return []

    papers = []
    for entry in root.findall(f"{{{ATOM_NS}}}entry"):
        paper = parse_entry(entry)
        if paper:
            papers.append(paper)

    return papers


# ---------------------------------------------------------------------------
# RSS-based fetching (announcement-date ordering)
# ---------------------------------------------------------------------------

ARXIV_RSS_BASE = "https://rss.arxiv.org/rss"

# Pattern: "arXiv:2602.07109v1 Announce Type: new \nAbstract: ..."
_RSS_DESC_RE = re.compile(
    r"arXiv:\S+\s+Announce\s+Type:\s*(\S+)\s+Abstract:\s*(.*)",
    re.DOTALL,
)


def parse_rss_description(raw_description: str) -> tuple[str, str]:
    """Parse the RSS <description> field.

    Returns:
        (announce_type, abstract) where announce_type is one of
        'new', 'cross', 'replace', 'replace-cross'.
    """
    match = _RSS_DESC_RE.search(raw_description)
    if not match:
        return ("unknown", "")
    announce_type = match.group(1).strip().lower()
    abstract = clean_text(match.group(2))
    return (announce_type, abstract)


def parse_rss_entry(entry) -> Optional[tuple[ArxivPaper, str]]:
    """Convert one feedparser entry to (ArxivPaper, announce_type).

    Returns None if the entry cannot be parsed.
    """
    try:
        link = entry.get("link", "")
        arxiv_id = extract_arxiv_id(link)
        if not arxiv_id:
            return None

        title = clean_text(entry.get("title", ""))

        # Authors: comma-separated string
        raw_author = entry.get("author", "")
        authors = [a.strip() for a in raw_author.split(",") if a.strip()]

        # Categories from tags
        tags = entry.get("tags", [])
        categories = [t["term"] for t in tags if t.get("term")]
        primary_category = categories[0] if categories else ""

        # Abstract and announce type from description
        announce_type, abstract = parse_rss_description(
            entry.get("description", "")
        )

        # Construct URLs
        pdf_url = f"https://arxiv.org/pdf/{arxiv_id}"
        html_url = f"https://arxiv.org/abs/{arxiv_id}"

        # Published date from the RSS feed (announcement date)
        published_parsed = entry.get("published_parsed")
        if published_parsed:
            published = datetime(*published_parsed[:6])
        else:
            published = datetime.now()

        paper = ArxivPaper(
            arxiv_id=arxiv_id,
            title=title,
            authors=authors,
            abstract=abstract,
            categories=categories,
            primary_category=primary_category,
            pdf_url=pdf_url,
            html_url=html_url,
            published=published,
            updated=published,  # RSS only has announcement date
        )
        return (paper, announce_type)
    except Exception as e:
        print(f"Warning: Failed to parse RSS entry: {e}")
        return None


def fetch_papers_from_rss(
    categories: list[str],
    keep_types: tuple[str, ...] = ("new", "cross"),
) -> list[ArxivPaper]:
    """Fetch today's papers from arXiv RSS feed.

    The RSS feed returns papers by announcement date, matching the
    website's "new listings" page. A single HTTP request fetches all
    categories at once.

    Args:
        categories: arXiv categories (e.g. ["astro-ph.GA", "astro-ph.CO"])
        keep_types: announce_type values to keep (default: new + cross)

    Returns:
        Deduplicated list of ArxivPaper objects, sorted by published date
    """
    url = f"{ARXIV_RSS_BASE}/{'+'.join(categories)}"
    print(f"Fetching RSS feed: {url}")

    feed = feedparser.parse(url)

    if feed.bozo and not feed.entries:
        print(f"Error fetching RSS feed: {feed.bozo_exception}")
        return []

    category_set = set(categories)
    seen_ids: set[str] = set()
    papers: list[ArxivPaper] = []

    for entry in feed.entries:
        result = parse_rss_entry(entry)
        if result is None:
            continue

        paper, announce_type = result

        # Filter by announce type
        if announce_type not in keep_types:
            continue

        # Deduplicate
        if paper.arxiv_id in seen_ids:
            continue
        seen_ids.add(paper.arxiv_id)

        # Keep only papers whose primary category is in our set
        if paper.primary_category not in category_set:
            continue

        papers.append(paper)

    print(f"RSS feed: {len(feed.entries)} entries, {len(papers)} papers after filtering")

    # Sort by published date (newest first)
    papers.sort(key=lambda p: p.published, reverse=True)
    return papers

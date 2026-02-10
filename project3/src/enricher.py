"""Abstract enrichment for feeds with incomplete abstracts.

Fetches full abstracts from article HTML pages:
- A&A (EDP Sciences): Structured abstract with Context/Aims/Methods/Results.
- OJA: Extracts arXiv ID from article page, fetches abstract via arXiv API.
"""

import re
import time
import xml.etree.ElementTree as ET

from p3_config import EnrichmentConfig
from fetcher import fetch_url
from models import JournalArticle


def enrich_articles(
    articles: list[JournalArticle],
    enrichment_config: EnrichmentConfig,
    user_agent: str = "yuzhe-journal-summary/1.0",
) -> list[JournalArticle]:
    """Enrich abstracts for articles from supported publishers.

    Dispatches to publisher-specific enrichers. Only processes articles
    whose journal publisher is in the enrichment config's publisher list.

    Args:
        articles: Articles to potentially enrich.
        enrichment_config: Enrichment settings (delay, enabled publishers).
        user_agent: User-Agent for HTTP requests.

    Returns:
        The same articles list (modified in place) with enriched abstracts.
    """
    if not enrichment_config.enabled:
        return articles

    enabled_publishers = set(enrichment_config.publishers)
    delay = enrichment_config.delay_seconds

    # Map journal acronym -> publisher for lookup
    publisher_map = {
        "aa": "edp",
        "oja": "oja",
    }

    enricher_map = {
        "edp": _enrich_edp,
        "oja": _enrich_oja,
    }

    count = 0
    for article in articles:
        publisher = publisher_map.get(article.journal)
        if publisher not in enabled_publishers:
            continue

        enricher = enricher_map.get(publisher)
        if enricher is None:
            continue

        if not article.url:
            continue

        try:
            new_abstract = enricher(article, user_agent, delay)
            if new_abstract and len(new_abstract) > len(article.abstract):
                article.abstract = new_abstract
                article.abstract_enriched = True
                count += 1
        except Exception as e:
            print(f"  Warning: enrichment failed for {article.doi}: {e}")

    if count > 0:
        print(f"  Enriched {count} article abstracts")

    return articles


def _enrich_edp(
    article: JournalArticle,
    user_agent: str,
    delay: float,
) -> str:
    """Fetch structured abstract from A&A article HTML.

    A&A stores the abstract in a citation_abstract meta tag with HTML
    entities encoding the Context/Aims/Methods/Results structure.

    Args:
        article: The A&A article (needs article.url).
        user_agent: User-Agent header.
        delay: Delay between requests.

    Returns:
        Full abstract text, or empty string on failure.
    """
    import html as html_mod

    url = article.url
    if not url:
        return ""

    try:
        html_bytes = fetch_url(url, user_agent=user_agent, delay=delay)
        html_text = html_bytes.decode("utf-8", errors="replace")
    except Exception:
        return ""

    # Primary: citation_abstract meta tag (contains HTML-encoded abstract)
    meta_match = re.search(
        r'<meta\s+name="citation_abstract"\s+[^>]*content="([^"]+)"',
        html_text,
        re.IGNORECASE,
    )
    if meta_match:
        raw = html_mod.unescape(meta_match.group(1))
        clean = re.sub(r"<[^>]+>", "", raw).strip()
        if clean:
            return clean

    # Fallback: DC.description or description meta tag
    meta_match = re.search(
        r'<meta\s+name="(?:DC\.description|description)"\s+content="([^"]+)"',
        html_text,
        re.IGNORECASE,
    )
    if meta_match:
        return html_mod.unescape(meta_match.group(1)).strip()

    return ""


def _enrich_oja(
    article: JournalArticle,
    user_agent: str,
    delay: float,
) -> str:
    """Extract arXiv ID from OJA page, fetch abstract via arXiv API.

    OJA articles link to arXiv with "Read article at ArXiv".

    Args:
        article: The OJA article (needs article.url).
        user_agent: User-Agent header.
        delay: Delay between requests.

    Returns:
        Abstract from arXiv, or empty string on failure.
    """
    url = article.url
    if not url:
        return ""

    try:
        html_bytes = fetch_url(url, user_agent=user_agent, delay=delay)
        html_text = html_bytes.decode("utf-8", errors="replace")
    except Exception:
        return ""

    # Find arXiv link: "Read article at ArXiv" or arxiv.org/abs/{id}
    arxiv_match = re.search(
        r'href="https?://arxiv\.org/abs/([^"]+)"',
        html_text,
    )
    if not arxiv_match:
        return ""

    arxiv_id = arxiv_match.group(1).strip()
    article.arxiv_id = arxiv_id

    # Fetch abstract from arXiv API
    return _fetch_arxiv_abstract(arxiv_id, user_agent, delay)


def _fetch_arxiv_abstract(
    arxiv_id: str,
    user_agent: str,
    delay: float,
) -> str:
    """Fetch a paper's abstract from the arXiv API.

    Args:
        arxiv_id: The arXiv paper ID (e.g. "2401.12345").
        user_agent: User-Agent header.
        delay: Delay before request (rate limiting).

    Returns:
        Abstract text, or empty string on failure.
    """
    api_url = f"https://export.arxiv.org/api/query?id_list={arxiv_id}"

    try:
        data = fetch_url(api_url, user_agent=user_agent, delay=delay)
        root = ET.fromstring(data)
    except Exception:
        return ""

    ns = {"atom": "http://www.w3.org/2005/Atom"}
    entry = root.find("atom:entry", ns)
    if entry is None:
        return ""

    summary = entry.find("atom:summary", ns)
    if summary is not None and summary.text:
        # Clean up whitespace
        return " ".join(summary.text.split())

    return ""

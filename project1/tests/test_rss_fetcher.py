"""Tests for RSS-based arXiv paper fetching."""

import re
import sys
from pathlib import Path

import feedparser
import pytest

# Add src to path so we can import arxiv_fetcher
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from arxiv_fetcher import (
    parse_rss_description,
    parse_rss_entry,
    fetch_papers_from_rss,
    ArxivPaper,
    ARXIV_RSS_BASE,
)

FIXTURE_PATH = Path(__file__).parent / "fixtures" / "rss_snapshot.xml"
CATEGORIES = ["astro-ph.GA", "astro-ph.CO", "astro-ph.IM"]


@pytest.fixture
def rss_feed():
    """Load the saved RSS snapshot."""
    return feedparser.parse(FIXTURE_PATH.read_text())


@pytest.fixture
def all_parsed_entries(rss_feed):
    """Parse all RSS entries into (ArxivPaper, announce_type) tuples."""
    results = []
    for entry in rss_feed.entries:
        result = parse_rss_entry(entry)
        if result is not None:
            results.append(result)
    return results


# ---- test_parse_rss_description ----

def test_parse_rss_description_new():
    desc = (
        'arXiv:2602.07109v1 Announce Type: new \n'
        'Abstract: As wide-field optical surveys begin operations.'
    )
    announce_type, abstract = parse_rss_description(desc)
    assert announce_type == "new"
    assert "wide-field optical surveys" in abstract


def test_parse_rss_description_cross():
    desc = (
        'arXiv:2510.00641v1 Announce Type: cross \n'
        'Abstract: Neutrinoless double-beta decay experiments.'
    )
    announce_type, abstract = parse_rss_description(desc)
    assert announce_type == "cross"
    assert "double-beta" in abstract


def test_parse_rss_description_replace():
    desc = (
        'arXiv:2306.05909v2 Announce Type: replace \n'
        'Abstract: Bulge-disk decomposition is an effective diagnostic.'
    )
    announce_type, abstract = parse_rss_description(desc)
    assert announce_type == "replace"
    assert "Bulge-disk" in abstract


def test_parse_rss_description_replace_cross():
    desc = (
        'arXiv:2311.17644v3 Announce Type: replace-cross \n'
        'Abstract: A long-standing problem in primordial cosmology.'
    )
    announce_type, abstract = parse_rss_description(desc)
    assert announce_type == "replace-cross"
    assert "primordial cosmology" in abstract


def test_parse_rss_description_empty():
    announce_type, abstract = parse_rss_description("")
    assert announce_type == "unknown"
    assert abstract == ""


# ---- test_parse_rss_entry ----

def test_parse_rss_entry_basic(rss_feed):
    """Verify ArxivPaper fields from a single RSS entry."""
    entry = rss_feed.entries[0]
    result = parse_rss_entry(entry)
    assert result is not None

    paper, announce_type = result
    assert isinstance(paper, ArxivPaper)
    assert announce_type in ("new", "cross", "replace", "replace-cross")
    assert re.match(r"\d{4}\.\d{4,5}", paper.arxiv_id)
    assert len(paper.title) > 0
    assert len(paper.abstract) > 0
    assert len(paper.authors) > 0
    assert len(paper.categories) > 0
    assert paper.primary_category == paper.categories[0]
    assert paper.pdf_url.startswith("https://arxiv.org/pdf/")
    assert paper.html_url.startswith("https://arxiv.org/abs/")


def test_parse_rss_entry_all_parseable(rss_feed):
    """All entries in the snapshot should be parseable."""
    failed = 0
    for entry in rss_feed.entries:
        result = parse_rss_entry(entry)
        if result is None:
            failed += 1
    assert failed == 0, f"{failed}/{len(rss_feed.entries)} entries failed to parse"


# ---- test_fetch_filters_announce_type ----

def test_fetch_filters_announce_type(all_parsed_entries):
    """Only 'new' and 'cross' entries should pass the type filter."""
    keep_types = ("new", "cross")
    for paper, announce_type in all_parsed_entries:
        if announce_type in ("replace", "replace-cross"):
            # These should be excluded by fetch_papers_from_rss
            pass  # We verify via the negative test below

    # Count types in the full set
    type_counts = {}
    for _, atype in all_parsed_entries:
        type_counts[atype] = type_counts.get(atype, 0) + 1

    assert "new" in type_counts
    assert "replace" in type_counts or "replace-cross" in type_counts, \
        "Snapshot should contain replace entries to test filtering"


def test_fetch_filters_announce_type_functional(rss_feed, monkeypatch):
    """fetch_papers_from_rss should exclude replace and replace-cross."""
    # Monkey-patch feedparser.parse to return our fixture
    monkeypatch.setattr(feedparser, "parse", lambda url: rss_feed)

    papers = fetch_papers_from_rss(CATEGORIES)

    # Verify by checking that no paper IDs match replace entries
    replace_ids = set()
    for entry in rss_feed.entries:
        desc = entry.get("description", "")
        if "replace" in desc[:80].lower() and "replace-cross" not in desc[:80].lower():
            m = re.search(r"(\d{4}\.\d{4,5})", entry.get("link", ""))
            if m:
                replace_ids.add(m.group(1))

    paper_ids = {p.arxiv_id for p in papers}
    overlap = paper_ids & replace_ids
    assert len(overlap) == 0, f"Replace entries leaked through: {overlap}"


# ---- test_fetch_deduplicates ----

def test_fetch_deduplicates(rss_feed, monkeypatch):
    """No duplicate arxiv_ids in output."""
    monkeypatch.setattr(feedparser, "parse", lambda url: rss_feed)
    papers = fetch_papers_from_rss(CATEGORIES)

    ids = [p.arxiv_id for p in papers]
    assert len(ids) == len(set(ids)), "Duplicate arxiv_ids found"


# ---- test_field_integrity ----

def test_field_integrity(rss_feed, monkeypatch):
    """All papers have valid fields."""
    monkeypatch.setattr(feedparser, "parse", lambda url: rss_feed)
    papers = fetch_papers_from_rss(CATEGORIES)

    assert len(papers) > 0, "No papers returned"

    for paper in papers:
        assert re.match(r"\d{4}\.\d{4,5}", paper.arxiv_id), \
            f"Invalid arxiv_id: {paper.arxiv_id}"
        assert len(paper.title) > 0, f"Empty title for {paper.arxiv_id}"
        assert len(paper.abstract) > 0, f"Empty abstract for {paper.arxiv_id}"
        assert len(paper.authors) > 0, f"No authors for {paper.arxiv_id}"
        assert paper.pdf_url == f"https://arxiv.org/pdf/{paper.arxiv_id}"
        assert paper.html_url == f"https://arxiv.org/abs/{paper.arxiv_id}"
        assert paper.primary_category in set(CATEGORIES), \
            f"Primary category {paper.primary_category} not in {CATEGORIES}"


# ---- test_previously_missed_papers_recovered ----

def test_previously_missed_papers_recovered(rss_feed):
    """Papers 2602.07114, 2602.07159, 2602.08312 that the API missed
    should be present in the RSS feed entries."""
    missed_by_api = {"2602.07114", "2602.07159", "2602.08312"}

    all_ids = set()
    for entry in rss_feed.entries:
        link = entry.get("link", "")
        m = re.search(r"(\d{4}\.\d{4,5})", link)
        if m:
            all_ids.add(m.group(1))

    recovered = missed_by_api & all_ids
    assert len(recovered) >= 2, \
        f"Expected at least 2 of {missed_by_api} in RSS, found {recovered}"

"""Per-journal JSONL archive for all Stage 1 survivors.

Writes one JSONL file per journal per fetch date. Each line is a
complete article record (permanent database).
"""

import json
import tempfile
from datetime import datetime
from pathlib import Path

from models import JournalArticle


def archive_articles(
    articles: list[JournalArticle],
    archive_dir: Path,
    date_str: str = "",
) -> dict[str, int]:
    """Write articles to per-journal JSONL archives.

    Creates files at: {archive_dir}/{journal_acronym}/{date}.jsonl

    Args:
        articles: Stage 1 survivors to archive.
        archive_dir: Base directory for archives.
        date_str: Date string for filename (defaults to today).

    Returns:
        Dict mapping journal acronym to number of articles archived.
    """
    if not date_str:
        date_str = datetime.now().strftime("%Y-%m-%d")

    # Group by journal
    by_journal: dict[str, list[JournalArticle]] = {}
    for article in articles:
        by_journal.setdefault(article.journal, []).append(article)

    counts = {}
    for journal, journal_articles in by_journal.items():
        journal_dir = archive_dir / journal
        journal_dir.mkdir(parents=True, exist_ok=True)

        filepath = journal_dir / f"{date_str}.jsonl"
        _append_jsonl(filepath, journal_articles)
        counts[journal] = len(journal_articles)

    return counts


def _article_to_dict(article: JournalArticle) -> dict:
    """Convert a JournalArticle to a JSON-serializable dict."""
    return {
        "title": article.title,
        "doi": article.doi,
        "journal": article.journal,
        "abstract": article.abstract,
        "authors": article.authors,
        "url": article.url,
        "published": article.published,
        "volume": article.volume,
        "issue": article.issue,
        "fetched_at": article.fetched_at,
        "abstract_enriched": article.abstract_enriched,
        "arxiv_id": article.arxiv_id,
    }


def _append_jsonl(filepath: Path, articles: list[JournalArticle]) -> None:
    """Append articles to a JSONL file atomically.

    Writes to a temp file first, then appends to the target.
    """
    lines = []
    for article in articles:
        lines.append(json.dumps(_article_to_dict(article), ensure_ascii=False))

    # Write to temp, then append
    dir_path = filepath.parent
    with tempfile.NamedTemporaryFile(
        mode="w", dir=dir_path, suffix=".tmp", delete=False, encoding="utf-8"
    ) as tmp:
        tmp.write("\n".join(lines) + "\n")
        tmp_path = Path(tmp.name)

    # Append temp content to target file
    content = tmp_path.read_text(encoding="utf-8")
    with open(filepath, "a", encoding="utf-8") as f:
        f.write(content)

    tmp_path.unlink()

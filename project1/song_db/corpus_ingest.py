"""Corpus ingestion: read, normalize, deduplicate historical papers.

Pure Python + json — no ML dependencies needed.
"""

import json
import re
from pathlib import Path

from .models import PaperRecord


def normalize_arxiv_id(raw_id: str) -> str:
    """Strip URL prefix, arXiv: prefix, .pdf suffix, and version suffix.

    Examples:
        http://arxiv.org/abs/2602.12345v1  -> 2602.12345
        arXiv:1705.00637v2                 -> 1705.00637
        1705.00637                         -> 1705.00637
    """
    clean = raw_id.strip()
    # Strip common URL prefixes
    for prefix in ("http://arxiv.org/abs/", "https://arxiv.org/abs/",
                   "http://arxiv.org/pdf/", "https://arxiv.org/pdf/"):
        if clean.startswith(prefix):
            clean = clean[len(prefix):]
    # Strip arXiv: prefix
    if clean.lower().startswith("arxiv:"):
        clean = clean[6:]
    # Strip .pdf suffix
    if clean.endswith(".pdf"):
        clean = clean[:-4]
    # Strip version suffix (e.g., v1, v23)
    clean = re.sub(r'v\d+$', '', clean)
    return clean


def normalize_whitespace(text: str) -> str:
    """Collapse all whitespace (including newlines) to single spaces."""
    return " ".join(text.split())


def parse_jsonl_record(line: str) -> PaperRecord | None:
    """Parse a single JSONL line into a PaperRecord."""
    try:
        data = json.loads(line)
    except json.JSONDecodeError:
        return None

    arxiv_id = normalize_arxiv_id(data.get("arxiv_id", ""))
    if not arxiv_id:
        return None

    title = normalize_whitespace(data.get("title", ""))
    abstract = normalize_whitespace(data.get("abstract", ""))

    if not title or not abstract:
        return None

    categories = data.get("categories", [])
    if isinstance(categories, str):
        categories = [c.strip() for c in categories.split(",")]

    return PaperRecord(
        arxiv_id=arxiv_id,
        title=title,
        abstract=abstract,
        categories=categories,
        primary_category=data.get("primary_category", ""),
        published=data.get("published", ""),
        updated=data.get("updated", ""),
    )


def ingest_corpus(input_path: Path, output_path: Path) -> list[PaperRecord]:
    """Read corpus JSONL, normalize, deduplicate, and write cleaned output.

    Deduplication keeps the record with the latest 'updated' timestamp.

    Returns:
        List of deduplicated PaperRecord objects.
    """
    records_by_id: dict[str, PaperRecord] = {}
    total_lines = 0
    skipped = 0

    with open(input_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            total_lines += 1

            record = parse_jsonl_record(line)
            if record is None:
                skipped += 1
                continue

            existing = records_by_id.get(record.arxiv_id)
            if existing is None:
                records_by_id[record.arxiv_id] = record
            else:
                # Keep the one with the later 'updated' timestamp
                if record.updated > existing.updated:
                    records_by_id[record.arxiv_id] = record

    records = list(records_by_id.values())

    # Write cleaned output
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        for rec in records:
            obj = {
                "arxiv_id": rec.arxiv_id,
                "title": rec.title,
                "abstract": rec.abstract,
                "categories": rec.categories,
                "primary_category": rec.primary_category,
                "published": rec.published,
                "updated": rec.updated,
            }
            f.write(json.dumps(obj, ensure_ascii=False) + "\n")

    print("Corpus ingestion complete:")
    print(f"  Total lines read:   {total_lines}")
    print(f"  Skipped (invalid):  {skipped}")
    print(f"  Unique papers:      {len(records)}")
    print(f"  Output written to:  {output_path}")

    return records


def load_clean_corpus(path: Path) -> list[PaperRecord]:
    """Load a previously cleaned corpus JSONL into PaperRecord list."""
    records = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            record = parse_jsonl_record(line)
            if record is not None:
                records.append(record)
    return records

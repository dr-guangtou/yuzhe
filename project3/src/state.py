"""State management: track last-fetch timestamps and seen DOIs.

Persists to a JSON file with atomic writes (temp -> rename).
"""

import json
import os
import tempfile
from datetime import datetime
from pathlib import Path


class State:
    """Track pipeline state across runs."""

    def __init__(self, state_path: Path):
        self.state_path = state_path
        self.last_fetch: dict[str, str] = {}  # journal_acronym -> ISO timestamp
        self.seen_dois: set[str] = set()
        self._load()

    def _load(self) -> None:
        """Load state from disk."""
        if not self.state_path.exists():
            return

        try:
            with open(self.state_path, "r", encoding="utf-8") as f:
                data = json.load(f)
            self.last_fetch = data.get("last_fetch", {})
            self.seen_dois = set(data.get("seen_dois", []))
        except (json.JSONDecodeError, OSError) as e:
            print(f"Warning: failed to load state from {self.state_path}: {e}")

    def save(self) -> None:
        """Save state to disk atomically (temp file -> rename)."""
        data = {
            "last_fetch": self.last_fetch,
            "seen_dois": sorted(self.seen_dois),
            "updated_at": datetime.now().isoformat(),
        }

        # Write to temp file in same directory, then rename
        dir_path = self.state_path.parent
        dir_path.mkdir(parents=True, exist_ok=True)

        fd, tmp_path = tempfile.mkstemp(dir=str(dir_path), suffix=".tmp")
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2, ensure_ascii=False)
            os.replace(tmp_path, self.state_path)
        except Exception:
            # Clean up temp file on failure
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)
            raise

    def mark_fetched(self, journal_acronym: str) -> None:
        """Record that a journal was fetched now."""
        self.last_fetch[journal_acronym] = datetime.now().isoformat()

    def add_seen_dois(self, dois: list[str]) -> None:
        """Add DOIs to the seen set."""
        self.seen_dois.update(d for d in dois if d)

    def filter_unseen(self, articles: list) -> list:
        """Filter out articles whose DOI is already seen.

        Args:
            articles: List of JournalArticle objects.

        Returns:
            Articles with unseen DOIs (or no DOI).
        """
        result = []
        for article in articles:
            if article.doi and article.doi in self.seen_dois:
                continue
            result.append(article)
        return result

#!/usr/bin/env python3
"""Compatibility entry point for running the pipeline from project root.

Preferred usage remains:
    uv run python src/main.py
"""

from pathlib import Path
import runpy


def main() -> None:
    """Execute the real CLI entry point at src/main.py."""
    entrypoint = Path(__file__).parent / "src" / "main.py"
    runpy.run_path(str(entrypoint), run_name="__main__")


if __name__ == "__main__":
    main()

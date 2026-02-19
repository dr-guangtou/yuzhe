"""Tests for digest filename parsing and default naming."""

from datetime import datetime
import importlib.util
from pathlib import Path

from config import load_config
from formatter import save_digest


def test_parse_digest_date_supports_legacy_and_prefixed_names():
    """Both legacy and new digest filename styles should parse."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    legacy = module.parse_digest_date_from_path(Path("2026-02-19.md"))
    prefixed = module.parse_digest_date_from_path(Path("arxiv-2026-02-19.md"))

    assert legacy is not None
    assert prefixed is not None
    assert legacy.strftime("%Y-%m-%d") == "2026-02-19"
    assert prefixed.strftime("%Y-%m-%d") == "2026-02-19"


def test_save_digest_uses_arxiv_prefix(tmp_path):
    """Default saved digest filename should include the arxiv- prefix."""
    config = load_config(Path("config.yaml"))
    config.config_path = tmp_path / "config.yaml"
    config.output.digest_dir = "arxiv_digest"
    config.output.archive_subdir = "archive"

    output_path = save_digest(
        content="# test digest",
        config=config,
        date=datetime(2026, 2, 19),
    )

    assert output_path.name == "arxiv-2026-02-19.md"
    assert output_path.exists()

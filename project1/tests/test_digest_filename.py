"""Tests for digest filename parsing and default naming."""

from datetime import datetime
import importlib.util
from pathlib import Path
from types import SimpleNamespace
import pytest

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


def test_get_latest_digest_date_scans_across_years(tmp_path):
    """Latest digest discovery should not be limited to current year."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    config = load_config(Path("config.yaml"))
    config.config_path = tmp_path / "config.yaml"
    config.output.digest_dir = "arxiv_digest"
    config.output.archive_subdir = "archive"

    digest_2025 = tmp_path / "arxiv_digest" / "archive" / "2025" / "arxiv-2025-12-31.md"
    digest_2026 = tmp_path / "arxiv_digest" / "archive" / "2026" / "arxiv-2026-01-01.md"
    digest_2025.parent.mkdir(parents=True, exist_ok=True)
    digest_2026.parent.mkdir(parents=True, exist_ok=True)
    digest_2025.write_text("# older", encoding="utf-8")
    digest_2026.write_text("# newer", encoding="utf-8")

    latest = module.get_latest_digest_date(config)

    assert latest is not None
    assert latest.strftime("%Y-%m-%d") == "2026-01-01"


def test_get_previous_digest_ids_reads_multiple_files_in_window(tmp_path):
    """Dedup ID extraction should include all digest files in the date window."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    config = load_config(Path("config.yaml"))
    config.config_path = tmp_path / "config.yaml"
    config.output.digest_dir = "arxiv_digest"
    config.output.archive_subdir = "archive"

    file_old = tmp_path / "arxiv_digest" / "archive" / "2026" / "arxiv-2026-02-10.md"
    file_new = tmp_path / "arxiv_digest" / "archive" / "2026" / "arxiv-2026-02-12.md"
    file_old.parent.mkdir(parents=True, exist_ok=True)
    file_old.write_text(
        "[old](https://arxiv.org/abs/2602.00001)\n",
        encoding="utf-8",
    )
    file_new.write_text(
        "[new](https://arxiv.org/abs/2602.00002)\n",
        encoding="utf-8",
    )

    ids, count = module.get_previous_digest_ids(
        config,
        since_date=datetime(2026, 2, 11),
    )

    assert count == 1
    assert ids == {"2602.00002"}


def test_get_latest_digest_date_uses_custom_output_directory(tmp_path):
    """Latest digest lookup should use custom output directory when provided."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    config = load_config(Path("config.yaml"))
    config.config_path = tmp_path / "config.yaml"

    custom_output_dir = tmp_path / "obsidian" / "arxiv-digests"
    digest_old = custom_output_dir / "arxiv-2026-02-18.md"
    digest_new = custom_output_dir / "arxiv-2026-02-19.md"
    digest_old.parent.mkdir(parents=True, exist_ok=True)
    digest_old.write_text("# old", encoding="utf-8")
    digest_new.write_text("# new", encoding="utf-8")

    latest = module.get_latest_digest_date(config, history_root=custom_output_dir)

    assert latest is not None
    assert latest.strftime("%Y-%m-%d") == "2026-02-19"


def test_resolve_digest_history_root_uses_output_path_parent(tmp_path):
    """When explicit output path is set, dedup should scan that parent directory."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    config = load_config(Path("config.yaml"))
    config.config_path = tmp_path / "config.yaml"

    explicit_output_path = tmp_path / "vault" / "digests" / "arxiv-2026-02-19.md"
    history_root = module.resolve_digest_history_root(config, output_path=explicit_output_path)

    assert history_root == explicit_output_path.parent


def test_build_dated_output_path_uses_default_filename_in_custom_directory():
    """Custom output directory should keep the standard dated filename format."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    output_path = module.build_dated_output_path(
        Path("custom/digests"),
        datetime(2026, 2, 19),
    )

    assert output_path == Path("custom/digests/arxiv-2026-02-19.md")


def test_get_dedup_since_date_uses_midnight_boundary():
    """Dedup window should start at local midnight, inclusive."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    now = datetime(2026, 2, 19, 15, 30, 10)
    cutoff = module.get_dedup_since_date(2, now=now)

    assert cutoff == datetime(2026, 2, 17, 0, 0, 0)


def test_get_dedup_since_date_rejects_negative_days():
    """Negative dedup window values should fail fast."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    with pytest.raises(ValueError, match="--dedup-days must be >= 0"):
        module.get_dedup_since_date(-1)


def test_count_papers_by_primary_category_preserves_requested_order():
    """Category count output should follow configured category ordering."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    papers = [
        SimpleNamespace(arxiv_id="2602.00001", primary_category="astro-ph.GA"),
        SimpleNamespace(arxiv_id="2602.00002", primary_category="astro-ph.CO"),
        SimpleNamespace(arxiv_id="2602.00003", primary_category="astro-ph.GA"),
    ]

    counts = module.count_papers_by_primary_category(
        papers,
        categories=["astro-ph.CO", "astro-ph.GA", "astro-ph.IM"],
    )

    assert list(counts.keys())[:3] == ["astro-ph.CO", "astro-ph.GA", "astro-ph.IM"]
    assert counts["astro-ph.CO"] == 1
    assert counts["astro-ph.GA"] == 2
    assert counts["astro-ph.IM"] == 0


def test_split_duplicate_papers_returns_fresh_and_duplicate_lists():
    """Dedup partitioning should keep order and split by arxiv_id membership."""
    spec = importlib.util.spec_from_file_location("pipeline_main", Path("src/main.py"))
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)

    paper_one = SimpleNamespace(arxiv_id="2602.00001", title="paper one")
    paper_two = SimpleNamespace(arxiv_id="2602.00002", title="paper two")
    paper_three = SimpleNamespace(arxiv_id="2602.00003", title="paper three")

    fresh, duplicates = module.split_duplicate_papers(
        [paper_one, paper_two, paper_three],
        previous_ids={"2602.00002"},
    )

    assert [paper.arxiv_id for paper in fresh] == ["2602.00001", "2602.00003"]
    assert [paper.arxiv_id for paper in duplicates] == ["2602.00002"]

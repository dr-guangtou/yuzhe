"""Configuration loader for Periodic Journal Summary.

Loads project3/config.yaml and references project1/config.yaml for
topics, scoring settings, and interest model configuration.
"""

import sys
from dataclasses import dataclass, field
from pathlib import Path

import yaml

from models import JournalFeedConfig


@dataclass
class FetchConfig:
    delay_seconds: float = 2.0
    timeout_seconds: float = 30.0
    user_agent: str = "yuzhe-journal-summary/1.0"


@dataclass
class EnrichmentConfig:
    enabled: bool = True
    delay_seconds: float = 1.0
    publishers: list[str] = field(default_factory=lambda: ["edp", "oja"])


@dataclass
class LocalFilterConfig:
    threshold: float = 0.5


@dataclass
class ScoringConfig:
    min_tier: str = "could_be_interesting"


@dataclass
class OutputConfig:
    reminder_dir: str = "summary/reminder"
    archive_dir: str = "summary"


@dataclass
class Project3Config:
    """Main configuration for project3."""

    journals: list[JournalFeedConfig]
    fetch: FetchConfig
    enrichment: EnrichmentConfig
    local_filter: LocalFilterConfig
    scoring: ScoringConfig
    output: OutputConfig
    project1_config_path: Path
    config_path: Path


def load_config(config_path: Path) -> Project3Config:
    """Load project3 configuration from YAML.

    Args:
        config_path: Path to project3/config.yaml.

    Returns:
        Project3Config with all settings.

    Raises:
        FileNotFoundError: If config file does not exist.
        ValueError: If config is invalid.
    """
    if not config_path.exists():
        raise FileNotFoundError(f"Config file not found: {config_path}")

    with open(config_path, "r", encoding="utf-8") as f:
        raw = yaml.safe_load(f)

    if raw is None:
        raise ValueError("Config file is empty")

    # Resolve project1 config path relative to project3 config
    p1_rel = raw.get("project1_config", "../project1/config.yaml")
    p1_config_path = (config_path.parent / p1_rel).resolve()
    if not p1_config_path.exists():
        raise FileNotFoundError(
            f"project1 config not found: {p1_config_path} "
            f"(resolved from '{p1_rel}')"
        )

    # Parse journals
    journals = []
    for j in raw.get("journals", []):
        journals.append(JournalFeedConfig(
            name=j["name"],
            acronym=j["acronym"],
            publisher=j["publisher"],
            feed_url=j["feed_url"],
            feed_url_alt=j.get("feed_url_alt", ""),
            enabled=j.get("enabled", True),
        ))

    if not journals:
        raise ValueError("At least one journal must be configured")

    # Parse fetch config
    fetch_raw = raw.get("fetch", {})
    fetch = FetchConfig(
        delay_seconds=fetch_raw.get("delay_seconds", 2.0),
        timeout_seconds=fetch_raw.get("timeout_seconds", 30.0),
        user_agent=fetch_raw.get("user_agent", "yuzhe-journal-summary/1.0"),
    )

    # Parse enrichment config
    enrich_raw = raw.get("enrichment", {})
    enrichment = EnrichmentConfig(
        enabled=enrich_raw.get("enabled", True),
        delay_seconds=enrich_raw.get("delay_seconds", 1.0),
        publishers=enrich_raw.get("publishers", ["edp", "oja"]),
    )

    # Parse local filter config
    lf_raw = raw.get("local_filter", {})
    local_filter = LocalFilterConfig(
        threshold=lf_raw.get("threshold", 0.5),
    )

    # Parse scoring config
    sc_raw = raw.get("scoring", {})
    scoring = ScoringConfig(
        min_tier=sc_raw.get("min_tier", "could_be_interesting"),
    )

    # Parse output config
    out_raw = raw.get("output", {})
    output = OutputConfig(
        reminder_dir=out_raw.get("reminder_dir", "summary/reminder"),
        archive_dir=out_raw.get("archive_dir", "summary"),
    )

    return Project3Config(
        journals=journals,
        fetch=fetch,
        enrichment=enrichment,
        local_filter=local_filter,
        scoring=scoring,
        output=output,
        project1_config_path=p1_config_path,
        config_path=config_path.resolve(),
    )


def setup_project1_imports(p1_config_path: Path) -> None:
    """Add project1 directories to sys.path for importing its modules.

    Adds both project1/src and project1/ (for song_db package).

    Args:
        p1_config_path: Resolved path to project1/config.yaml.
    """
    p1_root = p1_config_path.parent
    p1_src = p1_root / "src"

    for path in [str(p1_src), str(p1_root)]:
        if path not in sys.path:
            sys.path.insert(0, path)

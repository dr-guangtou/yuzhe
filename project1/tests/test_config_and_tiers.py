"""Tests for config compatibility and tier threshold behavior."""

from pathlib import Path

from config import load_config
from scorer import Tier, assign_tier


def test_load_config_uses_summary_tiers():
    """Current config should expose summary tiers under the new field name."""
    config = load_config(Path("config.yaml"))
    assert config.llm_scoring.summary_tiers == ["most_relevant", "somewhat_relevant"]


def test_load_config_accepts_legacy_keep_tiers(tmp_path):
    """Legacy keep_tiers key should remain backward compatible."""
    config_path = tmp_path / "config.yaml"
    config_path.write_text(
        """
category:
  primary: [astro-ph.GA]
topics:
  primary: ["Galaxy formation"]
llm_scoring:
  keep_tiers:
    - most_relevant
  tier_thresholds:
    most_relevant: 8.0
    somewhat_relevant: 6.0
    could_be_interesting: 3.0
""".strip(),
        encoding="utf-8",
    )

    config = load_config(config_path)
    assert config.llm_scoring.summary_tiers == ["most_relevant"]


def test_assign_tier_requires_score_strictly_above_somewhat_threshold():
    """Score equal to 6.0 should stay in Could Be Interesting."""
    config = load_config(Path("config.yaml"))

    assert assign_tier(6.0, project_match=False, config=config) == Tier.COULD_BE_INTERESTING
    assert assign_tier(6.01, project_match=False, config=config) == Tier.SOMEWHAT_RELEVANT

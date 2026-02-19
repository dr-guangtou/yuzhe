"""Tests for get_llm_score provider resolution."""

from config import (
    APIConfig,
    CategoryConfig,
    Config,
    LLMConfig,
    LLMScoringConfig,
    LocalFilterConfig,
    OutputConfig,
    SummaryConfig,
    TierThresholdsConfig,
    TopicConfig,
    TopicScorerConfig,
)
from get_llm_score import get_provider_candidates


def make_config() -> Config:
    """Create a minimal config for provider-order tests."""
    return Config(
        category=CategoryConfig(primary=["astro-ph.GA"], secondary=[]),
        topics=TopicConfig(primary=["Galaxy formation"], secondary=[]),
        projects=[],
        llm=LLMConfig(provider="kimi"),
        output=OutputConfig(),
        api=APIConfig(),
        local_filter=LocalFilterConfig(),
        topic_scorer=TopicScorerConfig(),
        llm_scoring=LLMScoringConfig(
            enabled=False,
            tier_thresholds=TierThresholdsConfig(),
        ),
        summary=SummaryConfig(),
        providers={},
        llm_fallback=["moonshot", "nvidia"],
    )


def test_provider_candidates_default_primary_first():
    """Default provider order should prefer primary before fallback."""
    config = make_config()
    assert get_provider_candidates(config, explicit_provider=None) == [
        "kimi",
        "moonshot",
        "nvidia",
    ]


def test_provider_candidates_explicit_provider_only():
    """Explicit provider should bypass fallback chain."""
    config = make_config()
    assert get_provider_candidates(config, explicit_provider="openai") == ["openai"]

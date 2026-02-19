"""Tests for summary fallback behavior."""

from datetime import datetime

from arxiv_fetcher import ArxivPaper
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
from scorer import ScoredPaper, Tier
from summarizer import generate_summaries


class FailingClient:
    """Client stub that should never be called in skip_llm mode."""

    def generate(self, *args, **kwargs):
        raise AssertionError("generate() should not be called when skip_llm=True")


def make_config() -> Config:
    """Create a minimal config object for summary tests."""
    return Config(
        category=CategoryConfig(primary=["astro-ph.GA"], secondary=[]),
        topics=TopicConfig(primary=["Galaxy formation"], secondary=[]),
        projects=[],
        llm=LLMConfig(provider="mock"),
        output=OutputConfig(),
        api=APIConfig(llm_retry_attempts=1, llm_retry_delay_seconds=0.0),
        local_filter=LocalFilterConfig(),
        topic_scorer=TopicScorerConfig(),
        llm_scoring=LLMScoringConfig(
            enabled=False,
            tier_thresholds=TierThresholdsConfig(),
        ),
        summary=SummaryConfig(enabled=True, fallback_to_abstract=True),
    )


def make_scored_paper(arxiv_id: str, tier: Tier) -> ScoredPaper:
    """Create a minimal scored paper for the requested tier."""
    paper = ArxivPaper(
        arxiv_id=arxiv_id,
        title="Test Paper",
        authors=["Author A"],
        abstract="Short abstract for fallback testing.",
        categories=["astro-ph.GA"],
        primary_category="astro-ph.GA",
        pdf_url=f"https://arxiv.org/pdf/{arxiv_id}",
        html_url=f"https://arxiv.org/abs/{arxiv_id}",
        published=datetime(2026, 2, 19),
        updated=datetime(2026, 2, 19),
    )
    return ScoredPaper(paper=paper, score=8.0, tier=tier)


def test_generate_summaries_skip_llm_uses_fallback_without_llm_calls():
    """skip_llm=True should bypass client calls and return fallback text."""
    config = make_config()
    scored_papers = [
        make_scored_paper("2602.00001", Tier.MOST_RELEVANT),
        make_scored_paper("2602.00002", Tier.SOMEWHAT_RELEVANT),
    ]

    summaries = generate_summaries(
        scored_papers=scored_papers,
        llm_client=FailingClient(),
        config=config,
        skip_llm=True,
    )

    assert "2602.00001" in summaries
    assert "ONE-SENTENCE SUMMARY" in summaries["2602.00001"]
    assert summaries["2602.00002"] == "Short abstract for fallback testing."

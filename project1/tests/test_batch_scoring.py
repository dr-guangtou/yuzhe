"""Tests for batch LLM scoring functionality."""

import json
from datetime import datetime
from unittest.mock import MagicMock, patch

import pytest

from arxiv_fetcher import ArxivPaper
from config import (
    APIConfig,
    CategoryConfig,
    Config,
    LLMConfig,
    LLMScoringConfig,
    LocalFilterConfig,
    OutputConfig,
    Project,
    SummaryConfig,
    TierThresholdsConfig,
    TopicConfig,
    TopicScorerConfig,
)
from llm_client import LLMResponse, MockLLMClient
from scorer import (
    build_papers_block,
    load_batch_scoring_prompt,
    score_papers,
    score_papers_batch_llm,
)


def make_paper(arxiv_id: str, title: str = "Test Paper", primary_cat: str = "astro-ph.GA") -> ArxivPaper:
    """Create a minimal ArxivPaper for testing."""
    return ArxivPaper(
        arxiv_id=arxiv_id,
        title=title,
        authors=["Author A"],
        abstract=f"Abstract for {arxiv_id}.",
        categories=[primary_cat],
        primary_category=primary_cat,
        pdf_url=f"https://arxiv.org/pdf/{arxiv_id}",
        html_url=f"https://arxiv.org/abs/{arxiv_id}",
        published=datetime(2026, 2, 10),
        updated=datetime(2026, 2, 10),
    )


def make_config() -> Config:
    """Create a minimal Config for testing."""
    return Config(
        category=CategoryConfig(primary=["astro-ph.GA"], secondary=["astro-ph.CO"]),
        topics=TopicConfig(
            primary=["Galaxy formation and evolution"],
            secondary=["Dark matter halos"],
        ),
        projects=[Project(name="LSST", acronym="LSST")],
        llm=LLMConfig(provider="mock"),
        output=OutputConfig(),
        api=APIConfig(llm_retry_attempts=1, llm_retry_delay_seconds=0.0),
        local_filter=LocalFilterConfig(),
        topic_scorer=TopicScorerConfig(),
        llm_scoring=LLMScoringConfig(
            enabled=True,
            tier_thresholds=TierThresholdsConfig(),
        ),
        summary=SummaryConfig(),
    )


class TestBuildPapersBlock:
    """Test the papers_block formatter."""

    def test_includes_all_papers(self):
        papers = [make_paper(f"2602.{i:05d}") for i in range(3)]
        block = build_papers_block(papers)

        for i, paper in enumerate(papers, 1):
            assert f"[Paper {i}]" in block
            assert f"arxiv_id: {paper.arxiv_id}" in block
            assert paper.title in block
            assert paper.abstract in block

    def test_single_paper(self):
        papers = [make_paper("2602.00001")]
        block = build_papers_block(papers)
        assert "[Paper 1]" in block
        assert "---" not in block  # no separator for single paper


class TestBatchPromptTemplate:
    """Test the batch prompt template."""

    def test_loads_successfully(self):
        template = load_batch_scoring_prompt()
        assert "{papers_block}" in template
        assert "{primary_topics}" in template
        assert "{secondary_topics}" in template
        assert "{projects}" in template
        assert "JSON array" in template


class TestParseBatchJsonResponse:
    """Test parsing of batch LLM JSON array responses."""

    def test_parse_valid_response(self):
        config = make_config()
        papers = [make_paper(f"2602.{i:05d}") for i in range(3)]

        response_entries = [
            {"arxiv_id": p.arxiv_id, "score": 7.0 + i, "matched_topics": ["Galaxy formation and evolution"], "reasoning": f"reason {i}"}
            for i, p in enumerate(papers)
        ]

        mock_client = MagicMock()
        mock_client.generate.return_value = LLMResponse(
            content=json.dumps(response_entries),
            model="mock",
            provider="mock",
        )

        template = load_batch_scoring_prompt()
        result = score_papers_batch_llm(papers, config, mock_client, template)

        assert len(result) == 3
        for paper in papers:
            assert paper.arxiv_id in result
            score, topics, reasoning = result[paper.arxiv_id]
            assert 0.0 <= score <= 10.0
            assert isinstance(topics, list)

    def test_parse_markdown_wrapped_response(self):
        """Response wrapped in ```json ... ``` should still parse."""
        config = make_config()
        papers = [make_paper("2602.00001")]

        raw = json.dumps([{"arxiv_id": "2602.00001", "score": 8.0, "matched_topics": [], "reasoning": "good"}])
        wrapped = f"```json\n{raw}\n```"

        mock_client = MagicMock()
        mock_client.generate.return_value = LLMResponse(content=wrapped, model="mock", provider="mock")

        template = load_batch_scoring_prompt()
        result = score_papers_batch_llm(papers, config, mock_client, template)
        assert "2602.00001" in result
        assert result["2602.00001"][0] == 8.0

    def test_non_array_response_raises(self):
        """Non-array JSON should raise ValueError."""
        config = make_config()
        papers = [make_paper("2602.00001")]

        mock_client = MagicMock()
        mock_client.generate.return_value = LLMResponse(
            content='{"score": 8.0}',
            model="mock",
            provider="mock",
        )

        template = load_batch_scoring_prompt()
        with pytest.raises(ValueError, match="Expected JSON array"):
            score_papers_batch_llm(papers, config, mock_client, template)


class TestBatchFallback:
    """Test fallback from batch to individual scoring on parse failure."""

    def test_fallback_on_garbage_response(self):
        config = make_config()
        papers = [make_paper(f"2602.{i:05d}") for i in range(3)]

        call_count = 0

        def mock_generate(prompt, **kwargs):
            nonlocal call_count
            call_count += 1
            if "[Paper 1]" in prompt:
                # Batch call returns garbage
                return LLMResponse(content="GARBAGE NOT JSON", model="mock", provider="mock")
            # Individual fallback calls
            return LLMResponse(
                content='{"score": 5.0, "matched_topics": [], "reasoning": "fallback"}',
                model="mock",
                provider="mock",
            )

        mock_client = MagicMock()
        mock_client.generate.side_effect = mock_generate

        scored = score_papers(papers, config, mock_client, batch_size=5)

        # All papers should be scored (via fallback)
        assert len(scored) == 3
        for sp in scored:
            assert sp.score > 0.0

        # Should have been called 1 (batch) + 3 (individual fallback) = 4 times
        assert call_count == 4


class TestBatchSizeOne:
    """Test that batch_size=1 uses the single-paper path."""

    def test_single_path_used(self):
        config = make_config()
        papers = [make_paper("2602.00001")]

        mock_client = MockLLMClient()

        with patch("scorer.score_papers_batch_llm") as mock_batch:
            scored = score_papers(papers, config, mock_client, batch_size=1)

            # batch function should never be called
            mock_batch.assert_not_called()
            assert len(scored) == 1


class TestBatchArxivIdMatching:
    """Test that batch results match papers by arxiv_id, not position."""

    def test_reversed_order_response(self):
        config = make_config()
        papers = [make_paper(f"2602.{i:05d}") for i in range(3)]

        # Response is in reversed order with different scores per ID
        response_entries = [
            {"arxiv_id": "2602.00002", "score": 9.0, "matched_topics": ["t1"], "reasoning": "r2"},
            {"arxiv_id": "2602.00001", "score": 7.0, "matched_topics": ["t2"], "reasoning": "r1"},
            {"arxiv_id": "2602.00000", "score": 3.0, "matched_topics": [], "reasoning": "r0"},
        ]

        mock_client = MagicMock()
        mock_client.generate.return_value = LLMResponse(
            content=json.dumps(response_entries),
            model="mock",
            provider="mock",
        )

        scored = score_papers(papers, config, mock_client, batch_size=5)

        # Build a lookup by arxiv_id
        scores_by_id = {sp.paper.arxiv_id: sp for sp in scored}

        assert scores_by_id["2602.00002"].score == 9.0
        assert scores_by_id["2602.00001"].score == 7.0
        assert scores_by_id["2602.00000"].score == 3.0


class TestMockLLMClientBatch:
    """Test that MockLLMClient handles batch prompts."""

    def test_mock_returns_array_for_batch_prompt(self):
        client = MockLLMClient()
        prompt = (
            "### [Paper 1] arxiv_id: 2602.00001\nTitle...\n"
            "### [Paper 2] arxiv_id: 2602.00002\nTitle...\n"
            "score relevance"
        )
        response = client.generate(prompt)
        result = json.loads(response.content)

        assert isinstance(result, list)
        assert len(result) == 2
        assert result[0]["arxiv_id"] == "2602.00001"
        assert result[1]["arxiv_id"] == "2602.00002"

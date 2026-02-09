"""Stage 2 (no-LLM path): Score papers by embedding similarity to config topics.

Embeds the user's topic descriptions (from config.yaml) once, then computes
cosine similarity against each paper's title+abstract embedding. A piecewise-
linear mapping converts raw cosine similarity into the 0-10 score scale used
by the rest of the pipeline.

This replaces the crude `local_score * 10.0` linear mapping with a
semantically meaningful, topic-aware scoring that mirrors the LLM prompt logic.
"""

import re
from dataclasses import dataclass, field

import numpy as np

from arxiv_fetcher import ArxivPaper
from config import Config, TopicScorerConfig
from scorer import ScoredPaper, Tier, assign_tier, check_project_match


@dataclass
class TopicScoreResult:
    """Intermediate scoring result for a single paper."""
    score: float
    matched_topics: list[str] = field(default_factory=list)
    matched_projects: list[str] = field(default_factory=list)
    matched_negative: list[str] = field(default_factory=list)
    reasoning: str = ""
    best_primary_sim: float = 0.0
    best_secondary_sim: float = 0.0
    best_negative_sim: float = 0.0
    n_primary_matches: int = 0
    negative_penalty: float = 0.0


def piecewise_map(value: float, breakpoints: dict[float, float]) -> float:
    """Map a value through piecewise-linear breakpoints.

    Args:
        value: Input value (e.g. cosine similarity).
        breakpoints: Sorted mapping {input: output}. Values below the
            lowest key map to the lowest output; above the highest key
            map to the highest output.

    Returns:
        Interpolated output value.
    """
    xs = sorted(breakpoints.keys())
    ys = [breakpoints[x] for x in xs]

    if value <= xs[0]:
        return ys[0]
    if value >= xs[-1]:
        return ys[-1]

    for i in range(len(xs) - 1):
        if xs[i] <= value <= xs[i + 1]:
            t = (value - xs[i]) / (xs[i + 1] - xs[i])
            return ys[i] + t * (ys[i + 1] - ys[i])

    return ys[-1]


class TopicScorer:
    """Score papers by embedding similarity to configured topics.

    Embeds topic descriptions once on init, then scores papers in batch.
    Reuses the same Embedder instance as the Stage 1 corpus filter.
    """

    def __init__(self, config: Config, embedder):
        """Initialize with config topics and a pre-loaded Embedder.

        Args:
            config: Pipeline config with topics and topic_scorer settings.
            embedder: song_db.embeddings.Embedder instance (already loaded).
        """
        self.config = config
        self.embedder = embedder
        self.ts_config = config.topic_scorer

        self.primary_topics = config.topics.primary
        self.secondary_topics = config.topics.secondary
        self.negative_topics = config.topics.negative

        # Embed all topic descriptions (cheap: ~50 short strings, <1s)
        all_topics = self.primary_topics + self.secondary_topics + self.negative_topics
        all_embeddings = embedder.embed_texts(all_topics)

        n_primary = len(self.primary_topics)
        n_secondary = len(self.secondary_topics)
        self.primary_embeddings = all_embeddings[:n_primary]                       # [P, 384]
        self.secondary_embeddings = all_embeddings[n_primary:n_primary+n_secondary]  # [S, 384]
        self.negative_embeddings = all_embeddings[n_primary+n_secondary:]           # [N, 384]

    def _score_single(
        self,
        paper_embedding: np.ndarray,
        paper: ArxivPaper,
    ) -> TopicScoreResult:
        """Score a single paper against topic embeddings.

        Args:
            paper_embedding: L2-normalized embedding vector [384].
            paper: The ArxivPaper (for project matching).

        Returns:
            TopicScoreResult with score, matched topics, and reasoning.
        """
        ts = self.ts_config

        # Cosine similarity (embeddings are L2-normalized, so dot = cosine)
        primary_sims = paper_embedding @ self.primary_embeddings.T    # [P]
        secondary_sims = paper_embedding @ self.secondary_embeddings.T  # [S]

        best_primary_sim = float(np.max(primary_sims)) if len(primary_sims) > 0 else 0.0
        best_secondary_sim = float(np.max(secondary_sims)) if len(secondary_sims) > 0 else 0.0

        # Negative topic similarity
        best_negative_sim = 0.0
        negative_sims = np.array([])
        if len(self.negative_embeddings) > 0:
            negative_sims = paper_embedding @ self.negative_embeddings.T  # [N]
            best_negative_sim = float(np.max(negative_sims))

        # Piecewise-linear mapping: cosine sim -> [0, 10] score
        primary_score = piecewise_map(best_primary_sim, ts.primary_breakpoints)
        secondary_score = piecewise_map(best_secondary_sim, ts.secondary_breakpoints)

        # OR gate: best of primary or secondary (mirrors LLM prompt logic)
        raw_score = max(primary_score, secondary_score)

        # Multi-match bonus: count primary topics above match_threshold
        n_primary_matches = int(np.sum(primary_sims >= ts.match_threshold))
        if n_primary_matches >= 2:
            raw_score += min(1.0, ts.multi_match_bonus * (n_primary_matches - 1))

        # Negative penalty: apply when a negative topic matches well (above
        # match_threshold). Full penalty when negative sim >= primary sim
        # (the scorer can't tell if this is really "Galactic" vs "galaxy").
        # Tapers to zero when primary exceeds negative by 0.15+ (primary
        # clearly dominates, so the paper is genuinely on-topic).
        neg_penalty = 0.0
        if best_negative_sim >= ts.match_threshold:
            gap = best_primary_sim - best_negative_sim
            damping = max(0.0, min(1.0, 1.0 - gap / 0.15))
            neg_penalty = ts.negative_penalty * damping
            raw_score -= neg_penalty

        # Project floor: mentions tracked project -> score >= 3
        matched_projects = check_project_match(paper, self.config)
        if matched_projects and raw_score < 3.0:
            raw_score = 3.0

        score = max(0.0, min(10.0, raw_score))

        # Collect matched topics (above match_threshold)
        matched_topics = []
        for i, sim in enumerate(primary_sims):
            if sim >= ts.match_threshold:
                matched_topics.append(self.primary_topics[i])
        for i, sim in enumerate(secondary_sims):
            if sim >= ts.match_threshold:
                matched_topics.append(self.secondary_topics[i])

        # Collect matched negative topics
        matched_negative = []
        for i, sim in enumerate(negative_sims):
            if sim >= ts.match_threshold:
                matched_negative.append(self.negative_topics[i])

        # Build reasoning string
        parts = [f"Topic-embedding score: {score:.1f}/10"]
        parts.append(f"best primary sim={best_primary_sim:.3f} -> {primary_score:.1f}")
        parts.append(f"best secondary sim={best_secondary_sim:.3f} -> {secondary_score:.1f}")
        if n_primary_matches >= 2:
            parts.append(f"multi-match bonus ({n_primary_matches} primary topics)")
        if neg_penalty > 0:
            parts.append(f"negative penalty: -{neg_penalty:.1f} (best neg sim={best_negative_sim:.3f})")
        if matched_negative:
            parts.append(f"negative topics: {', '.join(matched_negative[:3])}")
        if matched_projects:
            parts.append(f"project match: {', '.join(matched_projects)}")

        return TopicScoreResult(
            score=score,
            matched_topics=matched_topics,
            matched_projects=matched_projects,
            matched_negative=matched_negative,
            reasoning="; ".join(parts),
            best_primary_sim=best_primary_sim,
            best_secondary_sim=best_secondary_sim,
            best_negative_sim=best_negative_sim,
            n_primary_matches=n_primary_matches,
            negative_penalty=neg_penalty,
        )

    def score_papers(
        self,
        papers: list[ArxivPaper],
        config: Config,
    ) -> list[ScoredPaper]:
        """Score a batch of papers using topic-embedding similarity.

        Embeds all papers in one batch, computes similarity against cached
        topic embeddings, applies breakpoint mapping, and returns ScoredPaper
        objects ready for the rest of the pipeline.

        Args:
            papers: Papers that passed Stage 1 (corpus filter).
            config: Pipeline config (for tier thresholds and projects).

        Returns:
            List of ScoredPaper, sorted by tier then score.
        """
        if not papers:
            return []

        # Embed all papers in one batch
        texts = [f"{p.title}\n\n{p.abstract}" for p in papers]
        paper_embeddings = self.embedder.embed_texts(texts)  # [N, 384]

        scored_papers = []
        for paper, embedding in zip(papers, paper_embeddings):
            result = self._score_single(embedding, paper)

            project_match = bool(result.matched_projects)
            tier = assign_tier(result.score, project_match, config)

            scored_papers.append(ScoredPaper(
                paper=paper,
                score=result.score,
                tier=tier,
                matched_topics=result.matched_topics,
                matched_projects=result.matched_projects,
                reasoning=result.reasoning,
                prefilter_passed=True,
                project_match=project_match,
            ))

        # Sort by tier (most relevant first) then by score (highest first)
        tier_order = {
            Tier.MOST_RELEVANT: 0,
            Tier.SOMEWHAT_RELEVANT: 1,
            Tier.COULD_BE_INTERESTING: 2,
            Tier.NOT_RELEVANT: 3,
        }
        scored_papers.sort(key=lambda p: (tier_order[p.tier], -p.score))

        return scored_papers

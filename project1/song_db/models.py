"""Shared data schemas for the local interest model pipeline."""

from dataclasses import dataclass, field


@dataclass
class PaperRecord:
    """Normalized paper record, used throughout the pipeline.

    Mirrors the corpus JSONL schema with only the fields needed for scoring.
    """
    arxiv_id: str
    title: str
    abstract: str
    categories: list[str] = field(default_factory=list)
    primary_category: str = ""
    published: str = ""
    updated: str = ""


@dataclass
class TopicCluster:
    """A discovered topic cluster from the corpus embeddings."""
    topic_id: int
    centroid: list[float]
    top_keywords: list[str] = field(default_factory=list)
    exemplar_ids: list[str] = field(default_factory=list)
    size: int = 0


@dataclass
class InterestModel:
    """The distilled interest model artifact.

    Self-contained: everything needed to score new papers without
    the full corpus (only requires an Embedder at runtime).
    """
    model_id: str
    dim: int
    n_corpus: int
    category_priors: dict[str, float] = field(default_factory=dict)
    global_centroid: list[float] = field(default_factory=list)
    topic_clusters: list[TopicCluster] = field(default_factory=list)
    scoring_weights: dict[str, float] = field(default_factory=lambda: {
        "w_topic": 0.60,
        "w_global": 0.30,
        "w_category": 0.10,
    })
    created_at: str = ""


@dataclass
class LocalScore:
    """Scoring result for a single candidate paper."""
    score_total: float
    score_global: float
    score_topic_max: float
    best_topic_id: int
    score_category: float
    topic_scores_top3: list[tuple[int, float]] = field(default_factory=list)

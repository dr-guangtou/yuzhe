"""Distill corpus embeddings into a compact interest model.

Steps:
1. Category priors from paper metadata
2. Global centroid from mean of all embeddings
3. Topic centroids via KMeans clustering
4. Topic keywords via TF-IDF for interpretability
"""

import json
import math
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
from sklearn.cluster import KMeans
from sklearn.feature_extraction.text import TfidfVectorizer

from .embeddings import DEFAULT_MODEL, paper_text
from .models import InterestModel, PaperRecord, TopicCluster


def compute_category_priors(records: list[PaperRecord]) -> dict[str, float]:
    """Compute category weights from corpus frequency.

    Uses log(1 + count), normalized to [0, 1].
    """
    counter: Counter[str] = Counter()
    for rec in records:
        if rec.primary_category:
            counter[rec.primary_category] += 1
        for cat in rec.categories:
            counter[cat] += 1

    if not counter:
        return {}

    raw = {cat: math.log(1 + count) for cat, count in counter.items()}
    max_val = max(raw.values())
    if max_val == 0:
        return {}
    return {cat: val / max_val for cat, val in raw.items()}


def compute_global_centroid(embeddings: np.ndarray) -> np.ndarray:
    """Compute the mean of all embeddings, L2-normalized."""
    centroid = embeddings.mean(axis=0)
    norm = np.linalg.norm(centroid)
    if norm > 0:
        centroid /= norm
    return centroid


def compute_topic_clusters(
    embeddings: np.ndarray,
    ids: list[str],
    records: list[PaperRecord],
    k: int = 12,
    n_exemplars: int = 10,
    n_keywords: int = 15,
    random_state: int = 42,
) -> list[TopicCluster]:
    """Cluster embeddings and extract topic centroids + keywords.

    Args:
        embeddings: L2-normalized corpus embeddings, shape [N, D].
        ids: Aligned arxiv_id list.
        records: PaperRecord list (for keyword extraction).
        k: Number of topic clusters.
        n_exemplars: Number of exemplar papers per cluster.
        n_keywords: Number of TF-IDF keywords per cluster.
        random_state: Random seed for reproducibility.

    Returns:
        List of TopicCluster objects.
    """
    n_samples = len(ids)
    actual_k = min(k, n_samples)

    kmeans = KMeans(n_clusters=actual_k, random_state=random_state, n_init=10)
    labels = kmeans.fit_predict(embeddings)

    # Build TF-IDF for keyword extraction
    texts = [paper_text(r) for r in records]
    tfidf = TfidfVectorizer(
        max_features=10000,
        ngram_range=(1, 2),
        stop_words="english",
    )
    tfidf_matrix = tfidf.fit_transform(texts)
    feature_names = tfidf.get_feature_names_out()

    clusters = []
    for topic_id in range(actual_k):
        mask = labels == topic_id
        cluster_embeddings = embeddings[mask]
        cluster_indices = np.where(mask)[0]

        # Compute centroid and normalize
        centroid = cluster_embeddings.mean(axis=0)
        norm = np.linalg.norm(centroid)
        if norm > 0:
            centroid /= norm

        # Find exemplars: papers closest to centroid
        sims = cluster_embeddings @ centroid
        top_indices = np.argsort(-sims)[:n_exemplars]
        exemplar_ids = [ids[cluster_indices[i]] for i in top_indices]

        # Extract top keywords via mean TF-IDF within cluster
        cluster_tfidf = tfidf_matrix[cluster_indices].toarray().mean(axis=0)
        top_keyword_indices = np.argsort(-cluster_tfidf)[:n_keywords]
        top_keywords = [feature_names[i] for i in top_keyword_indices]

        clusters.append(TopicCluster(
            topic_id=topic_id,
            centroid=centroid.tolist(),
            top_keywords=top_keywords,
            exemplar_ids=exemplar_ids,
            size=int(mask.sum()),
        ))

    return clusters


def distill_interest_model(
    records: list[PaperRecord],
    embeddings: np.ndarray,
    ids: list[str],
    k_topics: int = 12,
    model_name: str = DEFAULT_MODEL,
    output_path: Path | None = None,
) -> InterestModel:
    """Build the full interest model from corpus data.

    Args:
        records: Cleaned corpus PaperRecords.
        embeddings: L2-normalized embeddings, shape [N, D].
        ids: Aligned arxiv_id list.
        k_topics: Number of topic clusters.
        model_name: Sentence-transformer model name.
        output_path: If given, save the model as JSON.

    Returns:
        InterestModel instance.
    """
    print("Computing category priors...")
    category_priors = compute_category_priors(records)

    print("Computing global centroid...")
    global_centroid = compute_global_centroid(embeddings)

    print(f"Computing {k_topics} topic clusters...")
    topic_clusters = compute_topic_clusters(
        embeddings, ids, records, k=k_topics,
    )

    model = InterestModel(
        model_id=model_name,
        dim=int(embeddings.shape[1]),
        n_corpus=len(records),
        category_priors=category_priors,
        global_centroid=global_centroid.tolist(),
        topic_clusters=topic_clusters,
        created_at=datetime.now(timezone.utc).isoformat(),
    )

    if output_path:
        save_interest_model(model, output_path)

    return model


def save_interest_model(model: InterestModel, path: Path) -> None:
    """Serialize the interest model to JSON."""
    data = {
        "model_id": model.model_id,
        "dim": model.dim,
        "n_corpus": model.n_corpus,
        "created_at": model.created_at,
        "category_priors": model.category_priors,
        "global_centroid": model.global_centroid,
        "scoring_weights": model.scoring_weights,
        "topic_clusters": [
            {
                "topic_id": tc.topic_id,
                "centroid": tc.centroid,
                "top_keywords": tc.top_keywords,
                "exemplar_ids": tc.exemplar_ids,
                "size": tc.size,
            }
            for tc in model.topic_clusters
        ],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
    print(f"Interest model saved to {path}")


def load_interest_model(path: Path) -> InterestModel:
    """Load an interest model from JSON."""
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)

    topic_clusters = [
        TopicCluster(
            topic_id=tc["topic_id"],
            centroid=tc["centroid"],
            top_keywords=tc.get("top_keywords", []),
            exemplar_ids=tc.get("exemplar_ids", []),
            size=tc.get("size", 0),
        )
        for tc in data.get("topic_clusters", [])
    ]

    return InterestModel(
        model_id=data["model_id"],
        dim=data["dim"],
        n_corpus=data["n_corpus"],
        category_priors=data.get("category_priors", {}),
        global_centroid=data["global_centroid"],
        topic_clusters=topic_clusters,
        scoring_weights=data.get("scoring_weights", {}),
        created_at=data.get("created_at", ""),
    )

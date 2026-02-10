"""Local ranker: score candidate papers against the interest model.

No network calls. Uses embedding similarity + category priors.
"""

import numpy as np

from .embeddings import Embedder, paper_text
from .models import InterestModel, LocalScore, PaperRecord


class LocalRanker:
    """Scores papers against a distilled interest model.

    Loads the model once, reuses for all scoring calls.
    Requires an Embedder instance for encoding candidate texts.
    """

    def __init__(self, model: InterestModel, embedder: Embedder):
        self.model = model
        self.embedder = embedder

        # Pre-convert centroids to numpy for fast dot products
        self.global_centroid = np.array(
            model.global_centroid, dtype=np.float32
        )
        self.topic_centroids = np.array(
            [tc.centroid for tc in model.topic_clusters],
            dtype=np.float32,
        )  # shape [K, D]

        self.weights = model.scoring_weights

    def score_papers(
        self,
        papers: list[PaperRecord],
        batch_size: int = 128,
    ) -> list[LocalScore]:
        """Score a batch of candidate papers.

        Embeds all papers at once, then computes scores via matrix ops.

        Returns:
            List of LocalScore, one per input paper.
        """
        if not papers:
            return []

        texts = [paper_text(p) for p in papers]
        embeddings = self.embedder.embed_texts(texts, batch_size=batch_size)

        # Global similarity: [N]
        global_sims = embeddings @ self.global_centroid

        # Topic similarities: [N, K]
        topic_sims = embeddings @ self.topic_centroids.T

        w_topic = self.weights.get("w_topic", 0.60)
        w_global = self.weights.get("w_global", 0.30)
        w_category = self.weights.get("w_category", 0.10)

        scores = []
        for i, paper in enumerate(papers):
            s_global = float(global_sims[i])
            topic_scores = topic_sims[i]
            s_topic_max = float(np.max(topic_scores))
            best_topic_id = int(np.argmax(topic_scores))

            # Top-3 topics
            top3_indices = np.argsort(-topic_scores)[:3]
            top3 = [(int(idx), float(topic_scores[idx])) for idx in top3_indices]

            # Category prior score
            s_cat = 0.0
            priors = self.model.category_priors
            for cat in paper.categories:
                s_cat = max(s_cat, priors.get(cat, 0.0))
            if paper.primary_category:
                s_cat = max(s_cat, priors.get(paper.primary_category, 0.0))

            # When a paper has no category info (e.g. journal articles),
            # redistribute category weight proportionally to topic + global
            if s_cat == 0.0 and not paper.categories and not paper.primary_category:
                w_sum = w_topic + w_global
                score_total = (w_topic / w_sum) * s_topic_max + (w_global / w_sum) * s_global
            else:
                score_total = w_topic * s_topic_max + w_global * s_global + w_category * s_cat

            scores.append(LocalScore(
                score_total=score_total,
                score_global=s_global,
                score_topic_max=s_topic_max,
                best_topic_id=best_topic_id,
                score_category=s_cat,
                topic_scores_top3=top3,
            ))

        return scores

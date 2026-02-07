"""Compute and persist semantic embeddings for the corpus.

Uses sentence-transformers for encoding. Embeddings are L2-normalized
so that cosine similarity = dot product.
"""

import json
from pathlib import Path

import numpy as np

from .models import PaperRecord

DEFAULT_MODEL = "sentence-transformers/all-MiniLM-L6-v2"


class Embedder:
    """Wrapper around a sentence-transformers model for text embedding."""

    def __init__(self, model_name: str = DEFAULT_MODEL):
        from sentence_transformers import SentenceTransformer
        self.model_name = model_name
        self.model = SentenceTransformer(model_name)
        self.dim = self.model.get_sentence_embedding_dimension()

    def embed_texts(self, texts: list[str], batch_size: int = 128) -> np.ndarray:
        """Embed a list of texts, returning L2-normalized float32 vectors.

        Returns:
            np.ndarray of shape (len(texts), self.dim), dtype float32.
        """
        embeddings = self.model.encode(
            texts,
            batch_size=batch_size,
            show_progress_bar=True,
            normalize_embeddings=True,
        )
        return np.asarray(embeddings, dtype=np.float32)


def paper_text(record: PaperRecord) -> str:
    """Construct the text to embed from a paper record."""
    return f"{record.title}\n\n{record.abstract}"


def compute_and_save_embeddings(
    records: list[PaperRecord],
    output_dir: Path,
    model_name: str = DEFAULT_MODEL,
    batch_size: int = 128,
) -> tuple[np.ndarray, list[str]]:
    """Compute embeddings for all records and persist to disk.

    Saves:
        - embeddings.npy (float32, shape [N, D])
        - ids.json (list of arxiv_id, aligned with rows)
        - embeddings_meta.json (model info)

    Returns:
        Tuple of (embeddings array, ids list).
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    embedder = Embedder(model_name)
    texts = [paper_text(r) for r in records]
    ids = [r.arxiv_id for r in records]

    print(f"Embedding {len(texts)} papers with {model_name}...")
    embeddings = embedder.embed_texts(texts, batch_size=batch_size)

    # Verify unit norm
    norms = np.linalg.norm(embeddings, axis=1)
    assert np.allclose(norms, 1.0, atol=1e-5), "Embeddings are not unit-normalized"

    # Save
    np.save(output_dir / "embeddings.npy", embeddings)

    with open(output_dir / "ids.json", "w", encoding="utf-8") as f:
        json.dump(ids, f)

    meta = {
        "model_id": model_name,
        "dim": int(embedder.dim),
        "n_vectors": len(ids),
        "normalized": True,
        "dtype": "float32",
    }
    with open(output_dir / "embeddings_meta.json", "w", encoding="utf-8") as f:
        json.dump(meta, f, indent=2)

    print(f"Embeddings saved to {output_dir}/")
    print(f"  Shape: {embeddings.shape}, dtype: {embeddings.dtype}")

    return embeddings, ids


def load_embeddings(artifacts_dir: Path) -> tuple[np.ndarray, list[str]]:
    """Load previously computed embeddings and their IDs."""
    embeddings = np.load(artifacts_dir / "embeddings.npy")
    with open(artifacts_dir / "ids.json", "r", encoding="utf-8") as f:
        ids = json.load(f)
    return embeddings, ids

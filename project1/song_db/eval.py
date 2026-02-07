"""Evaluation harness for the local interest model.

Measures separation between known-positive (in-corpus) and
negative (out-of-corpus) papers using ROC-AUC, Precision@K, Recall@K.
"""

import json
import random
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

import numpy as np

from .corpus_ingest import normalize_arxiv_id, normalize_whitespace
from .embeddings import Embedder
from .models import InterestModel, PaperRecord
from .rank import LocalRanker

ARXIV_API_BASE = "https://export.arxiv.org/api/query"
NEGATIVE_CATEGORIES = ["astro-ph.EP", "astro-ph.HE", "hep-ph", "cs.AI", "cond-mat.str-el"]


def sample_positives(
    records: list[PaperRecord],
    n: int = 200,
    seed: int = 42,
) -> list[PaperRecord]:
    """Randomly sample n corpus papers as positive test set."""
    rng = random.Random(seed)
    return rng.sample(records, min(n, len(records)))


def fetch_negatives(
    corpus_ids: set[str],
    categories: list[str] | None = None,
    per_category: int = 50,
    cache_path: Path | None = None,
) -> list[PaperRecord]:
    """Fetch papers from non-corpus categories as negative set.

    If cache_path exists and is non-empty, loads from cache instead of fetching.
    """
    if cache_path and cache_path.exists() and cache_path.stat().st_size > 0:
        print(f"Loading cached negatives from {cache_path}")
        records = []
        with open(cache_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                data = json.loads(line)
                records.append(PaperRecord(
                    arxiv_id=data["arxiv_id"],
                    title=data["title"],
                    abstract=data["abstract"],
                    categories=data.get("categories", []),
                    primary_category=data.get("primary_category", ""),
                    published=data.get("published", ""),
                    updated=data.get("updated", ""),
                ))
        print(f"  Loaded {len(records)} cached negatives")
        return records

    if categories is None:
        categories = NEGATIVE_CATEGORIES

    all_negatives: list[PaperRecord] = []

    for cat in categories:
        print(f"Fetching negatives from {cat}...")
        params = {
            "search_query": f"cat:{cat}",
            "sortBy": "submittedDate",
            "sortOrder": "descending",
            "start": 0,
            "max_results": per_category * 2,  # fetch extra to allow filtering
        }
        url = f"{ARXIV_API_BASE}?{urllib.parse.urlencode(params)}"

        req = urllib.request.Request(
            url,
            headers={"User-Agent": "song-db-eval/1.0"},
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as response:
                xml_data = response.read()
        except Exception as e:
            print(f"  Warning: failed to fetch {cat}: {e}")
            continue

        try:
            root = ET.fromstring(xml_data)
        except ET.ParseError:
            continue

        ns = "http://www.w3.org/2005/Atom"
        arxiv_ns = "http://arxiv.org/schemas/atom"

        count = 0
        for entry in root.findall(f"{{{ns}}}entry"):
            if count >= per_category:
                break

            id_elem = entry.find(f"{{{ns}}}id")
            if id_elem is None or id_elem.text is None:
                continue
            arxiv_id = normalize_arxiv_id(id_elem.text)

            # Skip if in corpus
            if arxiv_id in corpus_ids:
                continue

            title_elem = entry.find(f"{{{ns}}}title")
            summary_elem = entry.find(f"{{{ns}}}summary")
            title = normalize_whitespace(title_elem.text) if title_elem is not None and title_elem.text else ""
            abstract = normalize_whitespace(summary_elem.text) if summary_elem is not None and summary_elem.text else ""

            if not title or not abstract:
                continue

            cats = []
            for cat_elem in entry.findall(f"{{{ns}}}category"):
                term = cat_elem.get("term")
                if term:
                    cats.append(term)

            primary_cat_elem = entry.find(f"{{{arxiv_ns}}}primary_category")
            primary_cat = primary_cat_elem.get("term", "") if primary_cat_elem is not None else ""

            all_negatives.append(PaperRecord(
                arxiv_id=arxiv_id,
                title=title,
                abstract=abstract,
                categories=cats,
                primary_category=primary_cat,
            ))
            count += 1

        print(f"  Got {count} negatives from {cat}")
        time.sleep(5)  # arXiv API: minimum 3s between requests (using 5s for safety)

    # Cache for future runs
    if cache_path and all_negatives:
        cache_path.parent.mkdir(parents=True, exist_ok=True)
        with open(cache_path, "w", encoding="utf-8") as f:
            for rec in all_negatives:
                obj = {
                    "arxiv_id": rec.arxiv_id,
                    "title": rec.title,
                    "abstract": rec.abstract,
                    "categories": rec.categories,
                    "primary_category": rec.primary_category,
                    "published": rec.published,
                    "updated": rec.updated,
                }
                f.write(json.dumps(obj, ensure_ascii=False) + "\n")
        print(f"Cached {len(all_negatives)} negatives to {cache_path}")

    return all_negatives


def compute_metrics(
    pos_scores: list[float],
    neg_scores: list[float],
) -> dict:
    """Compute evaluation metrics from positive and negative score lists.

    Returns dict with ROC-AUC, precision@K, recall@K, score statistics.
    """
    # Combine and create labels
    all_scores = pos_scores + neg_scores
    labels = [1] * len(pos_scores) + [0] * len(neg_scores)

    # Sort by score descending
    paired = sorted(zip(all_scores, labels), key=lambda x: -x[0])

    # ROC-AUC via trapezoidal rule
    n_pos = sum(labels)
    n_neg = len(labels) - n_pos

    if n_pos == 0 or n_neg == 0:
        auc = float("nan")
    else:
        tp = 0
        fp = 0
        auc = 0.0
        prev_fpr = 0.0
        prev_tpr = 0.0

        for score, label in paired:
            if label == 1:
                tp += 1
            else:
                fp += 1
            fpr = fp / n_neg
            tpr = tp / n_pos
            auc += (fpr - prev_fpr) * (tpr + prev_tpr) / 2
            prev_fpr = fpr
            prev_tpr = tpr

    # Precision@K and Recall@K
    k_values = [10, 25, 50, 100]
    precision_at_k = {}
    recall_at_k = {}
    for k in k_values:
        if k > len(paired):
            continue
        top_k_labels = [label for _, label in paired[:k]]
        tp_k = sum(top_k_labels)
        precision_at_k[f"P@{k}"] = tp_k / k
        recall_at_k[f"R@{k}"] = tp_k / n_pos if n_pos > 0 else 0.0

    pos_arr = np.array(pos_scores)
    neg_arr = np.array(neg_scores)

    return {
        "roc_auc": float(auc),
        "n_positives": len(pos_scores),
        "n_negatives": len(neg_scores),
        "pos_score_mean": float(pos_arr.mean()),
        "pos_score_median": float(np.median(pos_arr)),
        "pos_score_std": float(pos_arr.std()),
        "neg_score_mean": float(neg_arr.mean()),
        "neg_score_median": float(np.median(neg_arr)),
        "neg_score_std": float(neg_arr.std()),
        "score_separation": float(pos_arr.mean() - neg_arr.mean()),
        **precision_at_k,
        **recall_at_k,
    }


def run_evaluation(
    model: InterestModel,
    embedder: Embedder,
    records: list[PaperRecord],
    output_path: Path,
    n_positives: int = 200,
    seed: int = 42,
    negatives_cache: Path | None = None,
) -> dict:
    """Run full evaluation: sample positives, fetch negatives, compute metrics.

    Returns:
        Metrics dict (also saved to output_path).
    """
    ranker = LocalRanker(model, embedder)

    # Sample positives
    print(f"Sampling {n_positives} positive papers from corpus...")
    positives = sample_positives(records, n=n_positives, seed=seed)

    # Fetch or load negatives
    corpus_ids = {r.arxiv_id for r in records}
    negatives = fetch_negatives(
        corpus_ids=corpus_ids,
        cache_path=negatives_cache,
    )

    if not negatives:
        print("Warning: No negative papers available. Evaluation incomplete.")
        return {"error": "no negatives available"}

    # Score both sets
    print(f"Scoring {len(positives)} positives...")
    pos_local_scores = ranker.score_papers(positives)
    pos_scores = [s.score_total for s in pos_local_scores]

    print(f"Scoring {len(negatives)} negatives...")
    neg_local_scores = ranker.score_papers(negatives)
    neg_scores = [s.score_total for s in neg_local_scores]

    # Compute metrics
    metrics = compute_metrics(pos_scores, neg_scores)
    metrics["seed"] = seed
    metrics["model_id"] = model.model_id

    # Save
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(metrics, f, indent=2)

    print(f"\nEvaluation results:")
    print(f"  ROC-AUC:          {metrics['roc_auc']:.4f}")
    print(f"  Pos mean/median:  {metrics['pos_score_mean']:.4f} / {metrics['pos_score_median']:.4f}")
    print(f"  Neg mean/median:  {metrics['neg_score_mean']:.4f} / {metrics['neg_score_median']:.4f}")
    print(f"  Score separation: {metrics['score_separation']:.4f}")
    for key in sorted(metrics):
        if key.startswith("P@") or key.startswith("R@"):
            print(f"  {key}: {metrics[key]:.4f}")
    print(f"\nSaved to {output_path}")

    return metrics

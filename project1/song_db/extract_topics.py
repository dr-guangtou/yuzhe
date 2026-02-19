#!/usr/bin/env python3
"""Extract and name scientific topics from corpus embeddings.

Runs high-k KMeans on precomputed embeddings, extracts TF-IDF keywords
and exemplar titles per cluster, then uses an LLM to generate concise
human-readable topic names. Outputs a ranked Markdown file.

Usage:
    cd project1 && uv run python song_db/extract_topics.py
"""

import json
import os
import time
import urllib.request
from pathlib import Path

import numpy as np
from sklearn.cluster import KMeans
from sklearn.feature_extraction.text import TfidfVectorizer


ARTIFACTS_DIR = Path(__file__).parent / "artifacts"
CORPUS_PATH = ARTIFACTS_DIR / "corpus_clean.jsonl"
EMBEDDINGS_PATH = ARTIFACTS_DIR / "embeddings.npy"
IDS_PATH = ARTIFACTS_DIR / "ids.json"
OUTPUT_PATH = Path(__file__).parent / "interesting_topics.md"

N_CLUSTERS = 50
N_EXEMPLARS = 5
N_KEYWORDS = 15
RANDOM_STATE = 42

# LLM config — use kimi as primary, fallback to qwen
LLM_PROVIDERS = [
    {
        "name": "kimi",
        "api_key_env": "KIMI_API_KEY",
        "base_url": "https://api.moonshot.cn/v1",
        "model": "kimi-latest",
    },
    {
        "name": "qwen",
        "api_key_env": "QWEN_API_KEY",
        "base_url": "https://dashscope.aliyuncs.com/compatible-mode/v1",
        "model": "qwen-plus",
    },
]


def load_corpus(path: Path) -> list[dict]:
    """Load corpus JSONL into list of dicts."""
    records = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            records.append(json.loads(line))
    return records


def paper_text(record: dict) -> str:
    """Title + abstract for TF-IDF."""
    return f"{record['title']}\n\n{record['abstract']}"


def llm_call(prompt: str, system_prompt: str = "") -> str:
    """Call LLM with fallback across providers."""
    for provider in LLM_PROVIDERS:
        api_key = os.environ.get(provider["api_key_env"])
        if not api_key:
            continue

        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        payload = {
            "model": provider["model"],
            "messages": messages,
            "temperature": 0.2,
            "max_tokens": 200,
        }

        url = f"{provider['base_url'].rstrip('/')}/chat/completions"
        req = urllib.request.Request(
            url,
            data=json.dumps(payload).encode("utf-8"),
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {api_key}",
            },
            method="POST",
        )

        try:
            with urllib.request.urlopen(req, timeout=30) as response:
                result = json.loads(response.read().decode("utf-8"))
            return result["choices"][0]["message"]["content"].strip()
        except Exception as e:
            print(f"  [{provider['name']}] failed: {e}")
            continue

    raise RuntimeError("All LLM providers failed")


def name_cluster(keywords: list[str], exemplar_titles: list[str]) -> str:
    """Use LLM to generate a concise topic name for a cluster."""
    prompt = (
        "Given the following TF-IDF keywords and exemplar paper titles from a "
        "cluster of astrophysics/cosmology papers, provide a concise scientific "
        "topic name (5-15 words). The name should be specific enough to "
        "distinguish this topic from others, and general enough to cover the "
        "cluster.\n\n"
        f"Keywords: {', '.join(keywords)}\n\n"
        "Exemplar paper titles:\n"
    )
    for i, title in enumerate(exemplar_titles, 1):
        prompt += f"{i}. {title}\n"

    prompt += "\nRespond with ONLY the topic name, nothing else."

    return llm_call(prompt)


def main():
    # Load precomputed data
    print("Loading corpus...")
    records = load_corpus(CORPUS_PATH)
    print(f"  {len(records)} papers")

    print("Loading embeddings...")
    embeddings = np.load(EMBEDDINGS_PATH)
    with open(IDS_PATH, "r", encoding="utf-8") as f:
        ids = json.load(f)
    print(f"  Shape: {embeddings.shape}")

    assert len(records) == len(ids) == embeddings.shape[0], "Data alignment mismatch"

    # Run KMeans
    print(f"\nRunning KMeans with k={N_CLUSTERS}...")
    kmeans = KMeans(n_clusters=N_CLUSTERS, random_state=RANDOM_STATE, n_init=10)
    labels = kmeans.fit_predict(embeddings)
    print("  Done.")

    # Build TF-IDF
    print("Building TF-IDF matrix...")
    texts = [paper_text(r) for r in records]
    tfidf = TfidfVectorizer(
        max_features=10000,
        ngram_range=(1, 2),
        stop_words="english",
    )
    tfidf_matrix = tfidf.fit_transform(texts)
    feature_names = tfidf.get_feature_names_out()
    print("  Done.")

    # Extract cluster info
    print("\nExtracting cluster information...")
    clusters = []
    for topic_id in range(N_CLUSTERS):
        mask = labels == topic_id
        cluster_indices = np.where(mask)[0]
        cluster_embeddings = embeddings[mask]
        size = int(mask.sum())

        # Centroid
        centroid = cluster_embeddings.mean(axis=0)
        norm = np.linalg.norm(centroid)
        if norm > 0:
            centroid /= norm

        # Exemplar papers (closest to centroid)
        sims = cluster_embeddings @ centroid
        top_indices = np.argsort(-sims)[:N_EXEMPLARS]
        exemplar_global_indices = [cluster_indices[i] for i in top_indices]
        exemplar_titles = [records[i]["title"] for i in exemplar_global_indices]

        # TF-IDF keywords
        cluster_tfidf = tfidf_matrix[cluster_indices].toarray().mean(axis=0)
        top_keyword_indices = np.argsort(-cluster_tfidf)[:N_KEYWORDS]
        top_keywords = [feature_names[i] for i in top_keyword_indices]

        clusters.append({
            "topic_id": topic_id,
            "size": size,
            "keywords": top_keywords,
            "exemplar_titles": exemplar_titles,
            "name": "",  # filled by LLM
        })

    # Sort by size descending
    clusters.sort(key=lambda c: -c["size"])

    # Name clusters with LLM
    print(f"\nNaming {len(clusters)} clusters with LLM...")
    for i, cluster in enumerate(clusters):
        print(f"  [{i+1}/{len(clusters)}] size={cluster['size']}: ", end="", flush=True)
        try:
            name = name_cluster(cluster["keywords"], cluster["exemplar_titles"])
            cluster["name"] = name
            print(name)
        except Exception as e:
            # Fallback: use top 3 keywords
            cluster["name"] = ", ".join(cluster["keywords"][:3]).title()
            print(f"FALLBACK: {cluster['name']} ({e})")
        time.sleep(0.3)  # Rate limit

    # Split into primary and secondary
    # Primary: top ~25 clusters (or until size drops below median)
    median_size = sorted([c["size"] for c in clusters])[len(clusters) // 2]
    primary = []
    secondary = []
    for cluster in clusters:
        if len(primary) < 25 and cluster["size"] >= median_size:
            primary.append(cluster)
        else:
            secondary.append(cluster)

    # If we have too few primary, adjust
    if len(primary) < 20:
        primary = clusters[:25]
        secondary = clusters[25:]

    # Write Markdown output
    print(f"\nWriting to {OUTPUT_PATH}...")
    lines = []
    lines.append("# Interesting Scientific Topics")
    lines.append("")
    lines.append(f"Distilled from {len(records):,} arXiv preprints in the personal corpus.")
    lines.append(f"Method: KMeans clustering (k={N_CLUSTERS}) on sentence embeddings "
                 f"(all-MiniLM-L6-v2, 384-dim), with LLM-generated topic names.")
    lines.append("")
    total = sum(c["size"] for c in clusters)
    lines.append(f"**Total papers:** {total:,}")
    lines.append(f"**Primary topics:** {len(primary)}")
    lines.append(f"**Secondary topics:** {len(secondary)}")
    lines.append("")

    # Primary topics
    lines.append("## Primary Topics")
    lines.append("")
    lines.append("Core research interests, ranked by corpus frequency.")
    lines.append("")
    for i, cluster in enumerate(primary, 1):
        pct = 100 * cluster["size"] / total
        lines.append(f"### {i}. {cluster['name']}")
        lines.append("")
        lines.append(f"**Papers:** {cluster['size']} ({pct:.1f}%)")
        lines.append(f"**Keywords:** {', '.join(cluster['keywords'][:10])}")
        lines.append("")
        lines.append("**Exemplar papers:**")
        for title in cluster["exemplar_titles"]:
            lines.append(f"- {title}")
        lines.append("")

    # Secondary topics
    lines.append("## Secondary Topics")
    lines.append("")
    lines.append("Peripheral or niche interests, ranked by corpus frequency.")
    lines.append("")
    for i, cluster in enumerate(secondary, 1):
        pct = 100 * cluster["size"] / total
        lines.append(f"### {i}. {cluster['name']}")
        lines.append("")
        lines.append(f"**Papers:** {cluster['size']} ({pct:.1f}%)")
        lines.append(f"**Keywords:** {', '.join(cluster['keywords'][:10])}")
        lines.append("")
        lines.append("**Exemplar papers:**")
        for title in cluster["exemplar_titles"]:
            lines.append(f"- {title}")
        lines.append("")

    OUTPUT_PATH.write_text("\n".join(lines), encoding="utf-8")
    print(f"Done! Written {len(lines)} lines to {OUTPUT_PATH}")


if __name__ == "__main__":
    main()

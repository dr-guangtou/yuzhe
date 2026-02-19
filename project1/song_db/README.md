# song_db: Local Interest Model for arXiv Paper Filtering

A local, token-free first-round filtering system using sentence embeddings to score arXiv papers based on a historical corpus of ~14.5k papers labeled as "interesting."

## Overview

This module distills a corpus of papers into a compact **Interest Model** containing:
- Global centroid (mean embedding vector)
- Topic clusters (K=12 by default) with centroids and keywords
- Category priors (frequency-based weights for arXiv categories)

New papers are scored by computing semantic similarity to these centroids, providing a fast, cost-free alternative or complement to LLM-based scoring.

## Files

### Core Modules

| File | Purpose |
|------|---------|
| `models.py` | Data schemas: `PaperRecord`, `InterestModel`, `TopicCluster`, `LocalScore` |
| `corpus_ingest.py` | JSONL ingestion, arxiv_id normalization, deduplication |
| `embeddings.py` | Embedder wrapper for sentence-transformers, batch embedding |
| `distill.py` | Interest model distillation: category priors, centroids, KMeans clustering, TF-IDF keywords |
| `rank.py` | LocalRanker: score papers using semantic similarity (dot product) |
| `eval.py` | Evaluation harness: ROC-AUC, Precision@K, Recall@K with positives/negatives |
| `cli.py` | CLI with 5 subcommands: ingest, embed, distill, rank, eval |
| `__main__.py` | Entry point: `python -m song_db <command>` |

### Artifacts (generated)

| Path | Content |
|------|---------|
| `artifacts/corpus_clean.jsonl` | Deduplicated, normalized corpus (14,482 papers) |
| `artifacts/embeddings.npy` | L2-normalized embeddings (float32, shape [N, 384]) |
| `artifacts/ids.json` | Aligned list of arxiv_id |
| `artifacts/embeddings_meta.json` | Metadata: model_id, dim, normalized flag |
| `artifacts/interest_model.json` | Full interest model (centroids, topics, priors, weights) |
| `artifacts/negatives.jsonl` | Cached negative papers for evaluation (optional) |
| `artifacts/eval_report.json` | Evaluation metrics (ROC-AUC, P@K, R@K) |

## Workflow: Creating the Interest Model

### Prerequisites

```bash
# Ensure dependencies are installed
uv sync  # or: pip install sentence-transformers numpy scikit-learn
```

### Step 1: Ingest Corpus

**Input**: Raw corpus JSONL (one paper per line)
**Output**: `artifacts/corpus_clean.jsonl`

```bash
uv run python -m song_db ingest \
  --input <path/to/raw_corpus.jsonl> \
  --output artifacts/corpus_clean.jsonl
```

**What it does:**
- Normalizes arxiv_id (strips URL prefix, `arXiv:` prefix, version suffix `vN`)
- Normalizes whitespace in title/abstract
- Deduplicates by arxiv_id (keeps most recent by `updated` timestamp)
- Validates required fields (arxiv_id, title, abstract)

### Step 2: Compute Embeddings

**Input**: `artifacts/corpus_clean.jsonl`
**Output**: `artifacts/embeddings.npy`, `artifacts/ids.json`, `artifacts/embeddings_meta.json`

```bash
uv run python -m song_db embed \
  --corpus artifacts/corpus_clean.jsonl \
  --output-dir artifacts/ \
  --model sentence-transformers/all-MiniLM-L6-v2 \
  --batch-size 128
```

**What it does:**
- Concatenates title + "\n\n" + abstract for each paper
- Batch-encodes using sentence-transformers
- L2-normalizes each vector (so cosine similarity = dot product)
- Saves as NumPy array (float32) with aligned IDs

**Performance**: ~80 seconds for 14.5k papers on Apple Silicon (CPU)

### Step 3: Distill Interest Model

**Input**: `artifacts/corpus_clean.jsonl`, `artifacts/embeddings.npy`, `artifacts/ids.json`
**Output**: `artifacts/interest_model.json`

```bash
uv run python -m song_db distill \
  --corpus artifacts/corpus_clean.jsonl \
  --embeddings artifacts/embeddings.npy \
  --ids artifacts/ids.json \
  --output artifacts/interest_model.json \
  --k-topics 12
```

**What it does:**
1. **Category priors**: Computes `log(1 + count)` for each category, normalized to [0, 1]
2. **Global centroid**: Mean of all embeddings, L2-normalized
3. **Topic clusters**: KMeans (K=12) on embeddings
   - Each cluster gets a centroid (mean, L2-normalized)
   - TF-IDF (unigram+bigram) extracts top keywords for interpretability
4. **Scoring weights**: Default `w_topic=0.60, w_global=0.30, w_category=0.10`

### Step 4: Rank New Papers

**Input**: `candidates.jsonl` (same schema as corpus), `artifacts/interest_model.json`
**Output**: Ranked list (stdout or file)

```bash
uv run python -m song_db rank \
  --interest artifacts/interest_model.json \
  --candidates daily_candidates.jsonl \
  --topk 50
```

**Scoring formula:**
```
score = w_topic * max_topic_similarity
      + w_global * global_similarity
      + w_category * category_prior
```

**Output** (per paper):
- `score_total`: Final weighted score [0, 1]
- `score_global`: Similarity to global centroid
- `score_topic_max`: Max similarity across all 12 topics
- `best_topic_id`: ID of closest topic cluster
- `score_category`: Category prior weight

### Step 5: Evaluate (Optional)

**Input**: `artifacts/interest_model.json`, `artifacts/corpus_clean.jsonl`
**Output**: `artifacts/eval_report.json`

```bash
uv run python -m song_db eval \
  --interest artifacts/interest_model.json \
  --corpus artifacts/corpus_clean.jsonl \
  --n-positives 200 \
  --seed 42
```

**What it does:**
- Samples 200 random corpus papers as positives
- Fetches 250 negative papers from non-corpus categories (astro-ph.EP, astro-ph.HE, hep-ph, cs.AI, cond-mat.str-el)
  - **Important**: Respects arXiv rate limits (3 seconds minimum between requests)
  - Caches negatives to `artifacts/negatives.jsonl` to avoid re-fetching
- Computes ROC-AUC, Precision@K, Recall@K

**Example results** (current corpus):
- ROC-AUC: 0.967
- Positive mean score: 0.77, Negative mean score: 0.39
- P@50: 0.96

## Updating the Model with New Corpus

When you have an updated corpus:

1. **Replace the raw corpus** (keep a backup):
   ```bash
   cp new_corpus.jsonl backups/corpus_$(date +%Y%m%d).jsonl
   ```

2. **Re-run the pipeline** (steps 1-3):
   ```bash
   uv run python -m song_db ingest --input new_corpus.jsonl --output artifacts/corpus_clean.jsonl
   uv run python -m song_db embed --corpus artifacts/corpus_clean.jsonl --output-dir artifacts/
   uv run python -m song_db distill --corpus artifacts/corpus_clean.jsonl --embeddings artifacts/embeddings.npy --ids artifacts/ids.json --output artifacts/interest_model.json
   ```

3. **Re-evaluate** to verify performance:
   ```bash
   uv run python -m song_db eval --interest artifacts/interest_model.json --corpus artifacts/corpus_clean.jsonl --n-positives 200 --seed 42
   ```

4. **Compare metrics** with previous `eval_report.json` to ensure quality hasn't degraded

## Integration with Main Pipeline

The local filter integrates into `src/scorer.py` via `src/local_scorer.py`:

### Current modes

1. **Default (recommended): local scoring + summaries**
   ```bash
   uv run python src/main.py
   # Stage 1 local filter + Stage 2 topic-embedding scoring + Stage 3 summaries
   ```

2. **Enable LLM scoring path**
   ```bash
   uv run python src/main.py --use-llm-scoring
   # Stage 2 switches to LLM scoring (primary + fallback providers)
   ```

3. **No-summary scoring run**
   ```bash
   uv run python src/main.py --no-summary
   # Keeps scoring, skips Stage 3 summary generation
   ```

### Configuration

Edit `config.yaml`:
```yaml
local_filter:
  interest_model: "song_db/artifacts/interest_model.json"
  threshold: 0.5
  weights:
    w_topic: 0.60
    w_global: 0.30
    w_category: 0.10

llm_scoring:
  enabled: false  # enable with --use-llm-scoring

summary:
  enabled: true
```

Deprecated flags such as `--skip-llm` and `--use-local-filter` are accepted for backward compatibility only.

## Performance Notes

- **Embedding model**: `all-MiniLM-L6-v2` (384-dim, optimized for CPU)
- **Corpus size**: 14,482 papers (26.5 MB JSONL)
- **Embedding time**: ~80 seconds on Apple Silicon M-series
- **Model size**: interest_model.json is ~160 KB (very portable)
- **Inference**: Scoring 5 papers takes <0.2 seconds (batch)

## arXiv API Compliance

When fetching papers (e.g., in `eval.py`):
- **Minimum 3 seconds** between requests (arXiv rate limit)
- Single connection at a time
- No parallelization
- Cache results to avoid repeated fetching

See `/Users/mac/.claude/skills/arxiv-public-api/` for full API guidelines.

## Reproducibility

- All embeddings use `sentence-transformers/all-MiniLM-L6-v2` from Hugging Face
- KMeans uses fixed `random_state=42` for deterministic clustering
- Evaluation uses `seed=42` for reproducible sampling
- Model ID and creation timestamp stored in `interest_model.json`

To reproduce exact results:
- Use same sentence-transformers version
- Same corpus (order doesn't matter due to stable normalization)
- Same random seeds

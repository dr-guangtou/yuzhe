# Local arXiv Digest — Corpus Distillation + Local 1st-Round Selector (Development Plan)

## Goal

Build a **local**, token-free first-round selection mechanism for daily arXiv papers, using your historical corpus (~14.5k papers) to distill an **Interest Model** and score new papers with **semantic similarity** as the primary signal.  
LLM API calls remain optional for a second-stage summarization/reranking, but **not used** in the first-round selection.

Key requirements:
- **Semantic similarity is primary.**
- Design choices should be **pluggable / configurable** (easy to tweak later).
- Include **validation tests** using positives (in-corpus) and negatives (out-of-corpus).

Target environment:
- macOS on Apple Silicon (M-series). Default to **CPU** execution; optional acceleration (MPS) may be added later but is not required.

---

## Deliverables

1) **Distilled interest artifact**: `interest_model.json`
2) **Local embedding store** + optional index:
   - `embeddings.npy` (float32, L2-normalized)
   - `ids.json` (aligned list of arxiv_id)
   - optional: `index.*` (ANN backend)
3) **Local ranker CLI**: takes `daily_candidates.jsonl` and outputs ranked list (`ranked.jsonl`/`ranked.md`)
4) **Evaluation harness**: produces `eval_report.json` and optional plots

---

## Input Spec Requirement (Daily Candidates)

Your existing Python script fetches “today’s arXiv list” via the arXiv API. **As a required next development step**, it must persist the daily list into a JSONL file with the following schema.

### Required file name and format
- File: `daily_candidates.jsonl`
- Format: JSON Lines (**one JSON object per line**)
- Encoding: UTF-8

### Required fields per record (minimum)
Each line MUST be a JSON object with:

```json
{
  "arxiv_id": "2502.01234",
  "title": "Paper title ...",
  "abstract": "Abstract text ...",
  "categories": ["astro-ph.GA", "astro-ph.CO"],
  "primary_category": "astro-ph.GA",
  "published": "2026-02-07T00:00:00Z",
  "updated": "2026-02-07T00:00:00Z"
}
```

Field requirements:
- `arxiv_id` (str): canonical base arXiv id (no URL prefix; version suffix `vN` may be omitted or retained, but canonicalization must be consistent).
- `title` (str): non-empty.
- `abstract` (str): non-empty (critical for semantic scoring).
- `categories` (list[str]): may be empty list if unavailable, but should be present.
- `primary_category` (str or null): preferred.
- `published`, `updated` (ISO8601 str or null): recommended for auditing/recency filters.

**Strict requirement:** the local ranker must be able to operate with only these fields and no network calls.

---

## Module Overview

Implement the system as small modules with clear interfaces:

1) `corpus_ingest/` — ingest and normalize your historical corpus JSONL  
2) `embeddings/` — build and persist embeddings for corpus (local)  
3) `distill/` — build Interest Model (centroids, topics, category priors)  
4) `rank/` — score daily candidates locally, output ranked list  
5) `eval/` — validation with positives/negatives and metrics  
6) `cli/` — command-line entry points

---

## Step-by-Step Implementation Plan

### Step 0 — Define stable schemas & interfaces (do first)

#### 0.1 Paper record model
Define a dataclass or Pydantic model:

**`PaperRecord`**
- `arxiv_id: str`
- `title: str`
- `abstract: str`
- `primary_category: str | None`
- `categories: list[str]`
- `published: str | None`
- `updated: str | None`

Optional pass-through fields (not required by ranker):
- `authors: list[str] | None`, `doi: str | None`, `journal_ref: str | None`

#### 0.2 Embedder interface (semantic core)
**`Embedder`**
- `model_id: str`
- `dim: int`
- `embed_texts(texts: list[str]) -> np.ndarray  # shape (N, D), float32`

Default model option (CPU-friendly):
- `sentence-transformers/all-MiniLM-L6-v2` (D=384)

Configurable choices:
- `all-mpnet-base-v2` (higher quality, slower)

Implementation note (Mac):
- Default device: CPU. Add optional flag `--device cpu|mps` if desired, but keep CPU as the reference path.

#### 0.3 InterestModel artifact schema
Persist as JSON:

`interest_model.json` MUST contain:
- `model_id` (string)
- `dim` (int)
- `created_at` (ISO timestamp)
- `n_corpus` (int)
- `category_priors: {category: weight}`  (float weights)
- `centroids`:
  - `global: list[float]` length D (L2-normalized)
  - `topics: list[ {topic_id, centroid, top_keywords, exemplars} ]` (optional but recommended)
- `scoring_defaults`:
  - `w_topic`, `w_global`, `w_category` (and optional `w_lexical`)

---

### Step 1 — Corpus ingestion and cleaning

Input: your full historical corpus JSONL (14k+ lines).

1) Read JSONL → `PaperRecord`
2) Normalize:
   - `arxiv_id`: strip URL prefix, strip `arXiv:` prefix, strip `.pdf`, optionally strip version `vN` for canonicalization
   - normalize whitespace in title/abstract
3) Deduplicate by `arxiv_id`:
   - keep the most recent by `updated` if duplicates exist
4) Persist metadata:
   - Option A (recommended): SQLite `corpus.sqlite` table `papers`
   - Option B: cleaned JSONL `corpus_clean.jsonl`

Acceptance criteria:
- Deterministic number of unique `arxiv_id` after dedup.
- No empty abstracts in the final corpus (or explicitly flagged if kept).

---

### Step 2 — Compute corpus semantic embeddings (local)

Text to embed:
- `text = title + "\n\n" + abstract`

Process:
1) Batch embed all corpus texts.
2) Convert to float32.
3) **L2-normalize** each vector (so cosine similarity = dot product).
4) Persist:
   - `embeddings.npy` (shape [N, D])
   - `ids.json` (aligned arxiv_id list)
   - `embeddings_meta.json` with `model_id`, `dim`, `normalized=true`

Mac note:
- At 14.5k vectors, exact similarity computations are feasible without ANN. Keep ANN optional.

Acceptance criteria:
- Vectors have unit norm within tolerance.
- Stable results across runs given same model/version.

---

### Step 3 — Distill Interest Model (semantic-first, interpretable)

Goal: represent your interests compactly as centroid(s) + priors.

#### 3.1 Category priors
Compute from corpus:
- `count(primary_category)` and/or all `categories`
- Convert to weights (configurable), e.g.:
  - `w = log(1 + count)` then normalize to [0, 1]

Persist in `interest_model.json`.

#### 3.2 Global centroid
- `global_centroid = mean(embeddings)`
- L2-normalize; store as `centroids.global`

#### 3.3 Topic centroids (recommended)
Cluster embeddings to K topics.

Default: KMeans on embeddings.
- Config: `K=12` default, tweakable.
- For each topic:
  - centroid = mean(vectors in cluster), L2-normalize
  - exemplars: top 10–30 closest corpus papers to centroid

Persist in `interest_model.json`.

#### 3.4 Topic keywords (interpretability)
Build lexical TF-IDF over abstracts (unigram+bigrams).
For each topic cluster:
- compute mean TF-IDF and pick top 10–20 terms as `top_keywords`.

Note: This is for human inspection and debugging; semantic similarity remains the primary scoring signal.

Acceptance criteria:
- `interest_model.json` is self-contained and does not require the full corpus to score candidates (except optional neighbor evidence).

---

### Step 4 — Local first-round scoring of daily candidates (NO LLM)

Input: `daily_candidates.jsonl` (see required schema).

Output:
- `ranked.jsonl` (same records + scoring fields), and optional `ranked.md` summary.

#### 4.1 Scoring function
Embed each candidate text (title + abstract), L2-normalize.

Compute:
- `s_global = dot(e, centroid_global)`
- `s_topic = max_i dot(e, centroid_topic_i)` and keep `best_topic_id`
- `s_cat = max_{c in categories} category_priors.get(c, 0.0)` (or mean; configurable)

Default score (semantic-heavy):
- `score = w_topic*s_topic + w_global*s_global + w_category*s_cat`

Recommended defaults (tweakable via config):
- `w_topic=0.60, w_global=0.30, w_category=0.10`

#### 4.2 Output schema for ranked records
Each output record should include:
- `score_total`
- `score_global`
- `score_topic_max`
- `best_topic_id`
- `score_category`

Optional debug fields (recommended):
- `topic_scores_top3` (topic_id + score)

Acceptance criteria:
- Ranker runs offline (no network), deterministic given same interest model and embeddings model.

---

### Step 5 — Validation & evaluation (required)

Implement an evaluation harness to quantify separation between positives and negatives.

#### 5.1 Positive validation set
- Randomly sample `P=200` corpus papers as “candidates” (holdout).
- Score them using the ranker pipeline.
Expectation:
- High score distribution; e.g. median above a configurable threshold, and high recall at top-K.

#### 5.2 Negative validation set
Preferred approach (network required, but only for evaluation):
- Sample random arXiv papers from the **same date range** that are **not** in your corpus.
- Fetch title+abstract, form `daily_candidates.jsonl` for negatives.
- Score them.
Expectation:
- Low score distribution; low false-positive rate above threshold.

If you want evaluation fully offline:
- Maintain a cached “background set” JSONL (papers not in your corpus) built once.

#### 5.3 Metrics
Compute:
- ROC-AUC (positives vs negatives)
- Precision@K / Recall@K for mixed sets
- Score separation (mean/median pos vs neg)

Persist:
- `eval_report.json` with metrics, thresholds used, random seeds, model_id.

Acceptance criteria:
- Baseline metrics stored and regression tested.
- Reruns with same seed are reproducible.

---

## Recommended Mac-Friendly Implementation Choices (Defaults)

- Embeddings: `sentence-transformers` on CPU (MiniLM).
- Similarity search: exact dot-product against:
  - global centroid and topic centroids (fast)
  - optional neighbor retrieval using full corpus matrix multiply (optional, still feasible at 14.5k)

Avoid hard dependency on FAISS initially; add ANN only if corpus grows dramatically.

---

## CLI Commands (Required)

1) Ingest corpus:
```bash
python -m cli.ingest_corpus --input corpus.jsonl --out corpus.sqlite
```

2) Build embeddings:
```bash
python -m cli.build_embeddings --db corpus.sqlite --model all-MiniLM-L6-v2 --out embeddings/
```

3) Distill interest model:
```bash
python -m cli.distill_interest --db corpus.sqlite --emb embeddings/ --k-topics 12 --out interest_model.json
```

4) Rank daily candidates:
```bash
python -m cli.rank_daily --interest interest_model.json --candidates daily_candidates.jsonl --out ranked.jsonl --topk 100
```

5) Run evaluation:
```bash
python -m cli.run_eval --interest interest_model.json --db corpus.sqlite --out eval_report.json
```

All CLI tools must accept `--config config.yaml` for weights, K, thresholds, and seeds.

---

## Definition of Done

- You can run the pipeline end-to-end:
  1) corpus → embeddings → interest_model
  2) daily_candidates.jsonl → ranked.jsonl (offline)
  3) eval produces an ROC-AUC and a stable baseline report
- The first-round selection reduces daily arXiv lists to top-K without any paid API calls.
- The scoring is primarily semantic (embeddings), with category priors as a minor modifier.

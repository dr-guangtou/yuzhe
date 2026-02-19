# Daily arXiv Pipeline Design (3-Stage Architecture)

**Last Updated**: 2026-02-19

## Overview

The daily arXiv digest pipeline uses a **3-stage architecture** to efficiently filter, score, and summarize papers:

0. **Fetch** - RSS (default, announcement-date) or API (submission-date, multi-day)
1. **Stage 1: Local Filter** (MANDATORY) - Fast, token-free embedding-based filtering
2. **Stage 2: Scoring** (MANDATORY, two paths) - Topic-embedding scorer or LLM scorer
3. **Stage 3: Summary Generation** (OPTIONAL) - LLM-generated summaries with fallback

## Fetch Stage: RSS vs API

The pipeline supports two paper sources, selected via `--source {rss,api}` (default: `rss`).

### RSS Feed (default)

- **URL**: `https://rss.arxiv.org/rss/{cat1}+{cat2}+{cat3}` (single HTTP request)
- **Ordering**: By **announcement date** — matches the arXiv website's "new listings" page
- **Content**: Title, authors, abstract, categories, and `announce_type` metadata
- **Filtering**: Keeps `new` and `cross` listings, excludes `replace` and `replace-cross`
- **Limitation**: Only today's announcements. `--days` is not supported (auto-switches to API)

### Atom API (fallback)

- **URL**: `https://export.arxiv.org/api/query?search_query=cat:{category}` (one request per category)
- **Ordering**: By **submission date**, which can differ from announcement by 1-3 days
- **Content**: Full Atom XML with all metadata
- **Rate limiting**: 3s minimum between requests (arXiv terms of use)
- **Use case**: Multi-day lookback (`--days N`), specific date ranges

### Why RSS is the Default

The Atom API sorts by submission date, not announcement date. This causes systematic misses:

1. **Weekend batching**: Papers submitted Friday are announced Monday — a 3-day gap between submission and announcement dates. The API's date filter misses these.
2. **Indexing lag**: Newly announced papers may not appear in API search results for hours.
3. **Observed impact**: Papers 2602.07114, 2602.07159, 2602.08312 all appeared on arXiv's "new listings" page but were missed by the API-based fetcher.

The RSS feed returns exactly what the website shows under "new listings", eliminating these gaps. The tradeoff is that RSS only has today's papers — for historical lookback, the API remains available.

### Auto-Switch Behavior

When `--days` is specified with `--source rss` (the default), the pipeline automatically switches to the API source and logs the switch. This ensures `--days` always works regardless of the source setting.

---

## Pipeline Stages

### Stage 1: Local Filter (MANDATORY)

**Purpose**: Fast first-round filtering using pre-computed interest model

**Method**:
- Uses `song_db/` embedding-based interest model
- Scores papers via semantic similarity (cosine) to corpus centroids
- Threshold: configurable (default 0.5 = 5.0/10 scale)
- **No API calls** - runs locally with sentence-transformers

**Configuration** (`config.yaml`):
```yaml
local_filter:
  interest_model: "song_db/artifacts/interest_model.json"
  threshold: 0.5  # Papers scoring < 0.5 are filtered out
  weights:
    w_topic: 0.60
    w_global: 0.30
    w_category: 0.10
```

**CLI Override**:
```bash
--local-filter-threshold 0.6  # Raise threshold to 0.6
```

**Performance**:
- ~14,500 papers corpus → ~80s embedding time (one-time)
- Scoring 100 papers: <1 second
- ROC-AUC: 0.967 (excellent separation)

---

### Stage 2: LLM Scoring (OPTIONAL)

**Purpose**: Precise topic relevance scoring with LLM

**Method**:
- Re-scores papers that passed Stage 1
- Uses configured LLM API (with fallback chain)
- Assigns tiers: most_relevant, somewhat_relevant, could_be_interesting
- Filters papers based on `keep_tiers` configuration

**Configuration** (`config.yaml`):
```yaml
llm_scoring:
  enabled: false  # Default: OFF (override with --use-llm-scoring)
  keep_tiers:
    - most_relevant
    - somewhat_relevant
  tier_thresholds:
    most_relevant: 8.0
    somewhat_relevant: 5.0
    could_be_interesting: 3.0
```

**CLI**:
```bash
--use-llm-scoring  # Enable Stage 2
```

**When to use**:
- You want precise LLM-based topic relevance scoring
- Token cost is acceptable
- You need tier-based filtering

**When to skip**:
- Default mode (save tokens)
- Local filter provides sufficient accuracy
- Testing/development

---

### Stage 3: Summary Generation (OPTIONAL)

**Purpose**: Generate structured summaries or use abstracts

**Method**:
- Generates summaries for papers that passed previous stages
- Uses configured LLM API (with fallback chain)
- **Graceful fallback**: if LLM fails, outputs raw abstract with warning

**Configuration** (`config.yaml`):
```yaml
summary:
  enabled: true  # Default: ON (override with --no-summary)
  fallback_to_abstract: true  # Use abstract if LLM fails
```

**CLI**:
```bash
--no-summary  # Disable Stage 3, output abstracts only
```

**When to disable**:
- Testing tier distribution
- Quick relevance check
- Token budget constraints

---

## Execution Modes

### Default Mode: RSS + Local Filter + Summary

```bash
uv run python src/main.py
```

**Fetch**: RSS (today's announcements, single HTTP request)
**Pipeline**: Stage 1 → Stage 2 (topic-embedding) → Stage 3
**API Calls**: Summary generation only (most token-efficient)
**Update Check**: Yes (skips if no new papers)

---

### API Source Mode: Multi-Day Lookback

```bash
uv run python src/main.py --source api --days 3
```

**Fetch**: Atom API (submission-date ordering, one request per category)
**Pipeline**: Same as default mode
**Use Case**: Catching up after holidays, historical lookback

---

### LLM Scoring Mode: Full Pipeline

```bash
uv run python src/main.py --use-llm-scoring
```

**Pipeline**: Stage 1 → Stage 2 (LLM) → Stage 3
**API Calls**: Scoring + summaries
**Update Check**: Yes

---

### Debug Mode: Force Run

```bash
uv run python src/main.py --debug
```

**Effect**: Disables update check, runs regardless of new papers
**Combined**: `--debug --use-llm-scoring` for full pipeline without update check

---

### No Summary Mode: Filter Only

```bash
uv run python src/main.py --no-summary
```

**Pipeline**: Stage 1 → Stage 2 (topic-embedding, no LLM)
**Output**: Abstracts only, no summaries
**Use Case**: Quick tier distribution check

---

### Custom Output Location

```bash
# Explicit output file path
uv run python src/main.py --output outputs/digests/my-run.md

# Custom directory with default filename (arxiv-YYYY-MM-DD.md)
uv run python src/main.py --dir outputs/digests
```

`--output` and `--output-dir`/`--dir` are mutually exclusive.

---

## Update Mode

**Default Behavior** (when `--debug` is NOT used):

1. Check latest digest file date
2. Only fetch papers published after that date
3. Skip run if no new papers found

**Example**:
- Latest digest: `2026-02-06.md`
- Today: `2026-02-07`
- Result: Fetches 2 days of papers (with 1-day overlap for safety)

**Disable**:
```bash
--debug  # Force run regardless of digest date
```

---

## Configuration Summary

### Threshold Adjustments

| Parameter | Location | Default | Scale | Purpose |
|-----------|----------|---------|-------|---------|
| `local_filter.threshold` | `config.yaml` | 0.5 | 0-1 | Stage 1 cutoff (5.0/10) |
| `tier_thresholds.most_relevant` | `config.yaml` | 8.0 | 0-10 | LLM tier assignment |
| `tier_thresholds.somewhat_relevant` | `config.yaml` | 5.0 | 0-10 | LLM tier assignment |
| `tier_thresholds.could_be_interesting` | `config.yaml` | 3.0 | 0-10 | LLM tier assignment |

### Fetch Source

| Source | CLI Flag | Ordering | Scope | Rate Limit |
|--------|----------|----------|-------|------------|
| RSS (default) | `--source rss` | Announcement date | Today only | None (single request) |
| Atom API | `--source api` | Submission date | Multi-day (`--days`) | 3s between requests |

### Enable/Disable Stages

| Stage | Config Key | CLI Flag | Default |
|-------|------------|----------|---------|
| Stage 1: Local Filter | (always on) | - | ON |
| Stage 2: LLM Scoring | `llm_scoring.enabled` | `--use-llm-scoring` | OFF |
| Stage 3: Summary | `summary.enabled` | `--no-summary` | ON |

### Output Controls

| Output Mode | CLI Flag | Result |
|------------|----------|--------|
| Default archive path | (none) | `arxiv_digest/archive/YYYY/arxiv-YYYY-MM-DD.md` |
| Explicit file path | `--output PATH` | Writes digest to exactly `PATH` |
| Custom directory + default name | `--output-dir DIR` or `--dir DIR` | Writes to `DIR/arxiv-YYYY-MM-DD.md` |

---

## Migration from Previous Version

### Deprecated Flags

| Old Flag | New Approach | Notes |
|----------|--------------|-------|
| `--skip-llm` | (default mode) | Local filter is now default, LLM scoring is opt-in |
| `--use-local-filter` | (removed) | Local filter is now mandatory |
| `--skip-summary` | `--no-summary` | Renamed for clarity |
| `--mode update\|debug` | `--debug` flag | Use `--debug` to force run |

### Backward Compatibility

The deprecated flags are still accepted with warnings and automatically mapped to new behavior.

---

## Performance Characteristics

### Token Usage (100 papers)

| Mode | Stage 1 | Stage 2 | Stage 3 | Total Tokens (est.) |
|------|---------|---------|---------|---------------------|
| Default | ✓ | - | ✓ | ~50k (summaries only) |
| LLM Scoring | ✓ | ✓ | ✓ | ~150k (scoring + summaries) |
| No Summary | ✓ | - | - | 0 (local only) |
| No Summary + LLM | ✓ | ✓ | - | ~100k (scoring only) |

### Time (100 papers, Apple Silicon M-series)

| Stage | Time | Notes |
|-------|------|-------|
| Fetch (RSS) | ~2s | Single HTTP request for all categories |
| Fetch (API, 3 categories) | ~10s | 3s delay between categories |
| Stage 1: Local Filter | <1s | After initial model load (~3s) |
| Stage 2: LLM Scoring | 5-15 min | Depends on API latency |
| Stage 3: Summary | 10-20 min | Depends on API latency |

---

## Testing the Pipeline

### Unit Tests
```bash
uv sync --group dev   # Install pytest (one-time)
uv run pytest tests/ -v
```

### Quick Test (5 papers, all stages)
```bash
uv run python src/main.py --use-llm-scoring --limit 5 --mock-llm --debug
```

### Test RSS vs API Source
```bash
# RSS (default, today only)
uv run python src/main.py --no-summary --debug

# API (multi-day)
uv run python src/main.py --source api --days 3 --no-summary --debug
```

### Test Local Filter Only
```bash
uv run python src/main.py --no-summary --limit 10 --debug
```

### Test Summary Fallback
```bash
# Simulate API failure by using invalid provider
uv run python src/main.py --config config_broken.yaml --limit 5
```

### Test Individual Paper
```bash
uv run python test_local_filter.py --id 2301.07136
```

---

## Troubleshooting

### "Local filter failed"
- Ensure `song_db/artifacts/interest_model.json` exists
- Run: `uv run python -m song_db distill --help`
- See: `song_db/README.md`

### "No papers passed local filter threshold"
- Threshold may be too high
- Lower threshold: `--local-filter-threshold 0.3`
- Or in `config.yaml`: `local_filter.threshold: 0.3`

### "No papers passed LLM tier filter"
- Adjust `llm_scoring.keep_tiers` in config
- Or adjust `tier_thresholds` values

### Summary generation fails
- Check LLM API keys and providers
- Fallback will use abstracts if `summary.fallback_to_abstract: true`

---

## See Also

- `song_db/README.md` - Local interest model pipeline
- `config.yaml` - Full configuration reference
- `docs/lessons.md` - Lessons learned and best practices

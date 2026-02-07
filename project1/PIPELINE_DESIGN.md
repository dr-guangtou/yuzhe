# Daily arXiv Pipeline Design (3-Stage Architecture)

**Last Updated**: 2026-02-07

## Overview

The daily arXiv digest pipeline now uses a **3-stage architecture** to efficiently filter, score, and summarize papers:

1. **Stage 1: Local Filter** (MANDATORY) - Fast, token-free embedding-based filtering
2. **Stage 2: LLM Scoring** (OPTIONAL) - Precise LLM-based relevance scoring
3. **Stage 3: Summary Generation** (OPTIONAL) - LLM-generated summaries with fallback

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

### Default Mode: Local Filter + Summary

```bash
uv run python src/main.py
```

**Pipeline**: Stage 1 → Stage 3
**API Calls**: Summary generation only (most token-efficient)
**Update Check**: Yes (skips if no new papers)

---

### LLM Scoring Mode: Full Pipeline

```bash
uv run python src/main.py --use-llm-scoring
```

**Pipeline**: Stage 1 → Stage 2 → Stage 3
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

**Pipeline**: Stage 1 only (or Stage 1 → 2 if `--use-llm-scoring`)
**Output**: Abstracts only, no summaries
**Use Case**: Quick tier distribution check

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

### Enable/Disable Stages

| Stage | Config Key | CLI Flag | Default |
|-------|------------|----------|---------|
| Stage 1: Local Filter | (always on) | - | ON |
| Stage 2: LLM Scoring | `llm_scoring.enabled` | `--use-llm-scoring` | OFF |
| Stage 3: Summary | `summary.enabled` | `--no-summary` | ON |

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
| Stage 1: Local Filter | <1s | After initial model load (~3s) |
| Stage 2: LLM Scoring | 5-15 min | Depends on API latency |
| Stage 3: Summary | 10-20 min | Depends on API latency |

---

## Testing the Pipeline

### Quick Test (5 papers, all stages)
```bash
uv run python src/main.py --use-llm-scoring --limit 5 --mock-llm --debug
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

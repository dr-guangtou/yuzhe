# Project 3: Periodic Journal Summary

Monitor published papers in major astronomy journals via RSS feeds, filter by field relevance, score by topic interest, and maintain a permanent archive.

## Overview

Two-stage scoring pipeline:
1. **LocalRanker** (corpus filter) - Keeps all field-relevant papers in a permanent JSONL archive
2. **TopicScorer** (topic ranking) - Ranks papers by research topic similarity, generates a monthly digest

Reuses the interest model and topic embeddings from Project 1.

## Supported Journals

| Publisher | Journals | Abstract | Authors |
|-----------|----------|----------|---------|
| IOP Science | ApJ, AJ, ApJL, ApJS, PASP, JCAP | Full | Yes |
| OUP | MNRAS, MNRASL, PASJ | Full | Missing |
| EDP Sciences | A&A | Enriched from HTML | Yes |
| Nature | Nature Astronomy | Full | Yes |
| OJA | OJA | Enriched via arXiv | Varies |

## Setup

```bash
cd project3
uv venv
uv pip install feedparser pyyaml sentence-transformers numpy
```

Requires Project 1's interest model at `project1/song_db/artifacts/interest_model.json`.

## Usage

```bash
# Full run (all journals)
.venv/bin/python src/main.py

# Single journal
.venv/bin/python src/main.py --journal apj

# Dry run (fetch + parse only, no scoring)
.venv/bin/python src/main.py --dry-run

# Skip abstract enrichment (faster)
.venv/bin/python src/main.py --no-enrichment

# Debug mode (show top scored papers)
.venv/bin/python src/main.py --debug

# Custom config path
.venv/bin/python src/main.py --config /path/to/config.yaml
```

## Output

- `summary/reminder/YYYY-MM.md` - Monthly digest of topic-relevant papers, grouped by tier
- `summary/{journal}/YYYY-MM-DD.jsonl` - Per-journal daily archives of all field-relevant papers
- `.state.json` - Seen DOIs and last-fetch timestamps (ensures idempotency)

## Configuration

Edit `config.yaml` to:
- Enable/disable journals
- Set LocalRanker filter threshold (`local_filter.threshold`)
- Set minimum tier for reminders (`scoring.min_tier`)
- Configure abstract enrichment (`enrichment.enabled`, `enrichment.publishers`)
- Adjust fetch rate limiting (`fetch.delay_seconds`)

Topics, projects, and scoring breakpoints are shared from `project1/config.yaml`.

# Project 2: Personal Publication Record

Automatically generates bilingual (English + Chinese) summaries of publications for personal website and grant applications.

## Pipeline

1. **Fetch** publication list from ORCID API
2. **Enrich** metadata via NASA/ADS API (abstracts, authors, bibcodes, citations)
3. **Store** in a YAML database with merge/update detection
4. **Summarize** each paper via LLM (short + long, English + Chinese)
5. **Build** per-paper Markdown portfolio documents

## Quick Start

```bash
# Install dependencies
uv add pyyaml

# Configure user.yaml with your ORCID and info
# Set ADS_API_TOKEN in your shell profile

# Run (small-scale test with recent papers only)
uv run python project2/src/main.py --config project2/config.yaml --debug

# Fetch-only mode (no LLM calls)
uv run python project2/src/main.py --config project2/config.yaml --skip-summaries
```

## Configuration

- `user.yaml` - User identity (name, ORCID, affiliations)
- `config.yaml` - LLM providers, API settings, output paths

## Output Structure

```
publication_record/
  publication_list.yaml    # Full database
  publication_list.md      # Human-readable list grouped by year
  summary/{model}/         # LLM-generated summaries
  portfolio/               # Per-paper Markdown documents
    {slug}.md
    {slug}_figures/        # Placeholder for key figures
```

## Environment Variables

- `ADS_API_TOKEN` - NASA/ADS API token (required for metadata enrichment)
- LLM provider API keys (see config.yaml for full list)

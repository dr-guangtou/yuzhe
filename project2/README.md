# Project 2: Personal Publication Record

A tool that automatically fetches an astronomer's publications, enriches them with metadata, and generates bilingual (English + Chinese) summaries for use on personal websites and in grant applications.

## Motivation

Maintaining an up-to-date, well-summarized publication portfolio is time-consuming but essential for grant proposals and personal websites. This tool automates the entire workflow: it pulls the authoritative publication list from ORCID, enriches each entry with abstracts, citation counts, and links from NASA/ADS, then uses LLM APIs to generate both concise "punch-line" and detailed summaries in English and Chinese.

## Pipeline

The tool runs a 5-stage pipeline:

1. **Fetch** - Query the ORCID public API for the user's publication list (title, year, journal, DOI, arXiv ID), filtered by a configurable year cutoff.
2. **Enrich** - For each publication, query the NASA/ADS API (by DOI, falling back to arXiv ID) to obtain abstracts, full author lists, bibcodes, citation counts, and PDF links.
3. **Store** - Merge results into a YAML database (`publication_list.yaml`). Matching is by DOI (primary) or normalized title (fallback). Existing entries are updated with richer metadata; new entries are appended. A human-readable Markdown list grouped by year is also generated.
4. **Summarize** - For each publication with an abstract, make 4 LLM API calls: short English summary, detailed English summary, short Chinese translation, detailed Chinese translation. Results are saved as YAML files under `summary/{model_name}/`.
5. **Build Portfolio** - Generate a per-paper Markdown document with title, authors, citation info, links, and all four summaries. Placeholder figure directories are created for future use.

## Project Structure

```
project2/
  user.yaml                 # User identity (name, ORCID, affiliations)
  config.yaml               # LLM providers, API settings, output paths
  README.md
  PLANS.md
  .gitignore
  docs/journal/             # Development journal entries
  src/
    config.py               # Configuration loader (merges config.yaml + user.yaml)
    llm_client.py           # LLM abstraction with fallback support (from project1)
    orcid_fetcher.py        # ORCID API client + Publication dataclass
    ads_fetcher.py          # NASA/ADS API enrichment
    database.py             # YAML publication store with merge logic
    summarizer.py           # LLM summary generation + Chinese translation
    portfolio_builder.py    # Per-paper Markdown document builder
    state.py                # Run state tracking
    main.py                 # CLI entry point
  prompts/
    summary_short.md        # Prompt template for punch-line summaries
    summary_long.md         # Prompt template for detailed summaries
    translate.md            # Prompt template for English-to-Chinese translation
  publication_record/       # Output directory
    publication_list.yaml   # Full publication database
    publication_list.md     # Human-readable list grouped by year
    summary/{model}/        # LLM-generated summary YAML files
    portfolio/              # Per-paper Markdown documents + figure directories
  temp/                     # Temporary files (gitignored)
```

## Prerequisites

- **Python 3.10+** with `PyYAML` installed
- **NASA/ADS API token**: Set the `ADS_API_TOKEN` environment variable (get one at https://ui.adsabs.harvard.edu/user/settings/token)
- **At least one LLM API key**: The tool supports multiple providers (Kimi, Qwen, GLM, OpenAI, Gemini, DeepSeek, etc.). Set the corresponding environment variable (e.g. `KIMI_API_KEY`). See `config.yaml` for the full provider registry.

## Usage

All commands are run from the repository root (`yuzhe/`).

```bash
# Full pipeline: fetch publications, enrich, summarize, build portfolio
python3 project2/src/main.py --config project2/config.yaml --debug

# Fetch-only mode: no LLM calls, just fetch/enrich/store publications
python3 project2/src/main.py --config project2/config.yaml --skip-summaries

# Update mode (default): only process newly added publications
python3 project2/src/main.py --config project2/config.yaml
```

### CLI Flags

| Flag | Description |
|------|-------------|
| `--config PATH` | Path to config.yaml (default: `project2/config.yaml`) |
| `--debug` | Force re-process all publications regardless of state |
| `--skip-summaries` | Only fetch and store publications, skip LLM summary generation |

### Typical Workflow

1. **First run**: Use `--debug` to process all publications and generate summaries.
2. **Subsequent runs**: Run without flags. The tool detects new publications via database merge and only summarizes those.
3. **Metadata refresh**: Use `--skip-summaries` to update citation counts and metadata without spending LLM tokens.

## Configuration

### `user.yaml`

Contains user identity information (name, ORCID, affiliations). This file is not modified by the pipeline.

### `config.yaml`

- `user.orcid` / `user.year_cutoff` - Which publications to fetch
- `providers` - LLM provider registry (API key env vars, base URLs, models)
- `llm` - Primary LLM provider and settings (temperature, max_tokens)
- `llm_fallback` - Ordered list of providers to try if the primary fails
- `api` - Timeout and rate limit settings for ORCID, ADS, and LLM APIs
- `output` - Output directory paths

## Development Status

- v1 complete: full pipeline working end-to-end
- Figure extraction deferred to v2 (placeholder directories created)
- Tested with 37 publications (2021-2026), 36 enriched and summarized

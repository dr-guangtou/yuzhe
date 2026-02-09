# Daily arXiv Summary

Automated 3-stage pipeline that monitors arXiv daily, filters papers with local embeddings, optionally scores with LLM, and generates tiered Markdown summaries.

## Features

- **3-Stage Architecture** (see `PIPELINE_DESIGN.md` for details):
  1. **Local Filter** (MANDATORY) - Fast embedding-based filtering (song_db), no API calls
  2. **LLM Scoring** (OPTIONAL) - Precise topic relevance scoring (default: OFF to save tokens)
  3. **Summary Generation** (OPTIONAL) - LLM summaries with graceful fallback to abstracts
- **Three-Tier System**:
  - **Most Relevant**: Detailed summaries with methodology, key findings, and references
  - **Somewhat Relevant**: 3-5 sentence summaries
  - **Could Be Interesting**: Title and link only
- **Update Mode**: Only processes papers newer than latest digest
- **Multi-LLM**: Config-driven provider system (Moonshot, NVidia, Gemini, OpenAI, etc.)
- **Obsidian Integration**: Optionally copies digests to your Obsidian vault

## Installation

1. Navigate to the project directory:
   ```bash
   cd project1
   ```

2. Install dependencies with uv:
   ```bash
   uv sync
   ```

3. Set up at least one LLM API key (see providers in `config.yaml`):
   ```bash
   export MOONSHOT_API_KEY="sk-..."   # Moonshot/KIMI
   # or
   export GEMINI_API_KEY="..."        # Google Gemini
   # or any other provider listed in config.yaml
   ```

## Configuration

Edit `config.yaml` to customize. The three main sections have distinct purposes:

### Categories (Fetch Filter)

Which arXiv categories to monitor for new papers. Only papers whose **primary category** matches one of these are kept; cross-listed papers from other fields are filtered out.

```yaml
category:
  primary:    # Main research areas
    - astro-ph.CO
    - astro-ph.GA
  secondary:  # Broader interests
    - astro-ph.SR
```

### Topics (Core Scoring)

**This is the heart of the system.** Describe your research interests as short phrases. The LLM evaluates each paper against these descriptions semantically (not keyword matching):

```yaml
topics:
  primary:    # Your active research areas
    - "Galaxy formation and evolution in general"
    - "Galaxy-halo connection and its application in cosmology"
    - "Method: machine learning applications in astronomy"
  secondary:  # Broader interests
    - "Chemical evolution of galaxies"
    - "Large-scale structure of the universe"
```

### Projects (Floor Booster)

Astronomical surveys/missions you follow. Papers mentioning these get a small boost and guaranteed minimum visibility:

```yaml
projects:
  - name: "Dark Energy Survey"
    acronym: "DES"
  - name: "James Webb Space Telescope"
    acronym: "JWST"
```

### LLM Providers

All provider definitions live in `config.yaml` under `providers`. Each entry defines `api_key_env`, `base_url`, `default_model`, and `client_type`. To add a new provider, just add an entry - no code changes needed.

```yaml
providers:
  moonshot:
    api_key_env: "MOONSHOT_API_KEY"
    base_url: "https://api.moonshot.cn/v1"
    default_model: "kimi-latest"
  gemini:
    api_key_env: "GEMINI_API_KEY"
    default_model: "gemini-2.0-flash"
    client_type: "gemini"            # only Gemini needs this
  # ... see config.yaml for all 8 providers ...

llm:
  provider: "kimi"           # must exist in providers
  model: ""                  # empty = use provider default
  temperature: 0.3

llm_fallback:                # tried in order if primary fails
  - moonshot
  - nvidia
  - gemini
```

### Scoring Thresholds
```yaml
scoring:
  most_relevant_threshold: 8.0
  somewhat_relevant_threshold: 5.0
  could_be_interesting_threshold: 3.0
```

## Usage

**📖 For complete pipeline documentation, see `PIPELINE_DESIGN.md`**

All commands run from the `project1/` directory.

### Default Mode: Local Filter + Summary (Most Efficient)

```bash
uv run python src/main.py
```

Runs **Stage 1** (local filter) + **Stage 3** (summaries). No LLM scoring = minimal token usage. Update mode enabled (skips if no new papers).

### With LLM Scoring: Full 3-Stage Pipeline

```bash
uv run python src/main.py --use-llm-scoring
```

Runs all 3 stages: local filter → LLM scoring → summaries. More precise but uses more tokens.

### Debug Mode: Force Run

```bash
uv run python src/main.py --debug
```

Disable update check, run regardless of whether there are new papers. Combine with `--use-llm-scoring` for full pipeline in debug mode.

### No Summary: Filter/Score Only

```bash
uv run python src/main.py --no-summary
```

Skip summary generation, output abstracts only. Useful for checking tier distribution.

### Key Options

| Option | Description |
|--------|-------------|
| `--debug` | Force run without update check |
| `--use-llm-scoring` | Enable Stage 2 (LLM scoring, default: OFF) |
| `--no-summary` | Disable Stage 3 (summaries, default: ON) |
| `--limit N` | Process only the first N papers (testing) |
| `--days N` | Look back N days instead of 1 |
| `--category CAT` | Fetch from a single category (e.g., `astro-ph.GA`) |
| `--local-filter-threshold T` | Override local filter threshold (0-1, default: 0.5) |
| `--mock-llm` | Use mock LLM for testing (no API calls) |
| `--output PATH` | Write digest to a custom path |
| `-v` | Verbose output (debug-level logging) |
| `--no-log-file` | Don't write to the log file |

### Quick Test Runs

```bash
# Test local filter only (5 papers, no API calls)
uv run python src/main.py --no-summary --limit 5 --debug

# Test with LLM scoring (5 papers, mock LLM)
uv run python src/main.py --use-llm-scoring --limit 5 --mock-llm --debug

# Test individual paper scoring
uv run python test_local_filter.py --id 2301.07136
```

## Output

Daily digests are saved to `arxiv_digest/archive/YYYY/YYYY-MM-DD.md`.

Each digest contains:
- **Summary** with paper counts per tier
- **Most Relevant** (score >= 8): structured summaries with key findings, methods, datasets
- **Somewhat Relevant** (score >= 5): 3-5 sentence paragraphs
- **Could Be Interesting** (score >= 3): title and link only

## How the Pipeline Works

```
Fetch by CATEGORY → Stage 1: Local Filter (embeddings) →
    [Optional] Stage 2: LLM Scoring →
    [Optional] Stage 3: Summary Generation → Digest
```

**Stage 1** filters papers using pre-computed interest model (song_db), no API calls.
**Stage 2** (optional) re-scores with LLM for precise topic relevance.
**Stage 3** (optional) generates summaries or uses abstracts as fallback.

See `PIPELINE_DESIGN.md` for full architecture details and `song_db/README.md` for local filter details.

| Config Section | Purpose | Role in Scoring |
|----------------|---------|-----------------|
| **category** | Fetch filter | Which arXiv categories to monitor |
| **topics** | **Core scoring** | LLM evaluates semantic relevance to these research descriptions |
| **projects** | Floor booster | +0.5 score boost, guaranteed "Could Be Interesting" minimum |

The LLM returns a 0-10 score for each paper. Topics are semantic descriptions, not keywords - a paper about "stellar mass functions at z>3" matches "High-redshift galaxies" even without exact keyword overlap.

### Tier Thresholds

| Score | Tier | Action |
|-------|------|--------|
| >= 8 | Most Relevant | Detailed structured summary |
| >= 5 | Somewhat Relevant | 3-5 sentence summary |
| >= 3 | Could Be Interesting | Title and link only |
| < 3 | Not Relevant | Excluded from digest |

Thresholds are configurable in `config.yaml` under `scoring`.

### Degraded Mode (--skip-llm)

When LLM is unavailable, uses category as a rough proxy:
- Primary category: score 5.0 (Somewhat Relevant)
- Secondary category: score 3.5 (Could Be Interesting)
- Project boost: +1.5

### Score Calibration Tool

Use `get_llm_score.py` to check how specific papers score against your config. Useful for tuning thresholds and refining topic descriptions.

```bash
# Score a single paper
uv run python src/get_llm_score.py 2602.04962

# Score multiple papers
uv run python src/get_llm_score.py 2602.04962 2602.04974 2602.05396

# Use a specific LLM provider
uv run python src/get_llm_score.py 2602.04962 --provider nvidia

# A/B test a modified prompt
uv run python src/get_llm_score.py 2602.04962 --prompt prompts/match_preprint_v2.md

# JSON output for scripting
uv run python src/get_llm_score.py 2602.04962 2602.04974 --json
```

Accepts arXiv IDs (e.g., `2602.04962`) or full URLs (e.g., `https://arxiv.org/abs/2602.04962`).

## Project Structure

```
project1/
├── README.md           # This file
├── PLAN.md             # Implementation plan
├── config.yaml         # Configuration
├── pyproject.toml      # Python dependencies
├── src/
│   ├── main.py          # CLI entry point
│   ├── get_llm_score.py # Score calibration tool
│   ├── config.py        # Config loader
│   ├── arxiv_fetcher.py # arXiv API client
│   ├── llm_client.py    # LLM abstraction
│   ├── scorer.py        # Paper scoring
│   ├── summarizer.py    # Summary generation
│   ├── formatter.py     # Markdown output
│   ├── state.py         # Run state tracking
│   └── logger.py        # Logging setup
├── prompts/
│   ├── match_preprint.md
│   ├── summary_detailed.md
│   └── summary_brief.md
├── arxiv_digest/
│   └── archive/        # Daily outputs
├── logs/               # Log files
└── temp/               # Temporary files
```

## Troubleshooting

### "No papers found"
- arXiv may be down; try again later
- Try `--days 3` to widen the date window
- Verify categories in `config.yaml` are valid arXiv category codes

### LLM errors (401, 429, 400)
- **401 Unauthorized**: Check that the API key env var is exported (e.g., `echo $MOONSHOT_API_KEY`)
- **429 Too Many Requests**: Rate limited; wait a minute and retry, or switch provider
- **400 Bad Request**: Model may not support your temperature setting (e.g., `kimi-k2.5` requires `temperature=1`; use `kimi-latest` instead)
- The fallback chain will automatically try the next provider on failure

### "Already processed papers"
- Use `--mode debug` to force re-run regardless of state
- Or delete `.state.json` to reset

## License

MIT

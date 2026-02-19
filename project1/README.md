# Daily arXiv Summary

Automated pipeline that monitors arXiv daily, filters and scores papers locally, and generates tiered Markdown summaries.

## Features

- **RSS-First Fetching**: Uses arXiv RSS feeds by default, which list papers by **announcement date** (matching the website's "new listings"). Falls back to the Atom API for multi-day lookback.
- **3-Stage Architecture** (see `docs/pipeline-architecture.md` for details):
  1. **Local Filter** (MANDATORY) - Corpus-based embedding filter removes off-topic papers
  2. **Scoring** (MANDATORY, two paths) - Topic-embedding scorer (default) or LLM scorer
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

**This is the heart of the system.** Describe your research interests as short phrases. Both the local topic-embedding scorer and the LLM scorer evaluate each paper against these descriptions semantically (not keyword matching):

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

**For complete pipeline documentation, see `docs/pipeline-architecture.md`**

All commands run from the `project1/` directory.

### Default Mode (Recommended)

```bash
uv run python src/main.py
```

Fetches today's papers via **RSS feed** (announcement-date ordering), runs **Stage 1** (corpus filter) + **Stage 2** (topic-embedding scorer) + **Stage 3** (LLM summaries). The scoring is fully local (no LLM tokens spent on scoring). LLM is only used for summary generation. Update mode enabled (skips if no new papers).

### With API Source (Multi-Day Lookback)

```bash
uv run python src/main.py --source api --days 3
```

Uses the arXiv Atom API instead of RSS. Required for multi-day lookback since RSS only contains today's announcements. When `--days` is specified with the default RSS source, it **auto-switches** to the API.

### With LLM Scoring (Higher Precision)

```bash
uv run python src/main.py --use-llm-scoring
```

**Replaces** the local topic-embedding scorer with LLM-based scoring. Papers are **not scored twice** — `--use-llm-scoring` switches Stage 2 from the embedding path to the LLM path. More precise but uses significantly more tokens (one LLM call per paper for scoring + one for summary).

### Debug Mode: Force Run

```bash
uv run python src/main.py --debug
```

Disable update check, run regardless of whether there are new papers.

### No Summary: Score Only

```bash
uv run python src/main.py --no-summary
```

Skip Stage 3 (summary generation), output abstracts only. Useful for checking tier distribution without spending LLM tokens.

### Key Options

| Option | Description |
|--------|-------------|
| `--source {rss,api}` | Paper source (default: `rss`). RSS uses announcement date; API uses submission date |
| `--debug` | Force run without update check |
| `--use-llm-scoring` | Enable Stage 2 (LLM scoring, default: OFF) |
| `--no-summary` | Disable Stage 3 (summaries, default: ON) |
| `--limit N` | Process only the first N papers (testing) |
| `--days N` | Look back N days instead of 1 (auto-switches to API source) |
| `--category CAT` | Fetch from a single category (e.g., `astro-ph.GA`) |
| `--local-filter-threshold T` | Override local filter threshold (0-1, default: 0.5) |
| `--mock-llm` | Use mock LLM for testing (no API calls) |
| `--output PATH` | Write digest to a custom path |
| `-v` | Verbose output (debug-level logging) |
| `--no-log-file` | Don't write to the log file |

### Quick Test Runs

```bash
# Run pipeline with no LLM at all (local scoring + abstracts only)
uv run python src/main.py --no-summary --debug

# Run full pipeline (local scoring + LLM summaries)
uv run python src/main.py --debug

# Use API source with 3-day lookback
uv run python src/main.py --source api --days 3 --debug

# Run with LLM scoring (replaces local topic scorer with LLM scorer)
uv run python src/main.py --use-llm-scoring --debug

# Test individual paper: corpus filter (Stage 1) only
uv run python test_local_filter.py --id 2301.07136

# Test individual papers: full local scoring (Stage 1 + Stage 2)
uv run python test_local_score.py 2602.06904,2602.06439

# LLM scoring calibration tool
uv run python src/get_llm_score.py 2602.04962
```

## Output

Daily digests are saved to `arxiv_digest/archive/YYYY/arxiv-YYYY-MM-DD.md`.

Each digest contains:
- **Summary** with paper counts per tier and categories monitored
- **Most Relevant** (score >= 8): title (linked to abs page + HTML rendering), authors, score, summary/abstract
- **Somewhat Relevant** (score >= 5): same format as Most Relevant
- **Could Be Interesting** (score >= 3): title and link only

## How the Pipeline Works

```
Fetch (RSS or API)
    → Dedup against previous digest
    → Stage 1: Corpus Filter (local embeddings, pass/fail gate)
    → Stage 2: Scoring (one of two mutually exclusive paths)
        ├── Default: Topic-Embedding Scorer (local, no LLM tokens)
        └── --use-llm-scoring: LLM Scorer (sends prompt per paper)
    → Stage 3: Summary Generation (optional, LLM)
    → Digest
```

### Fetch: RSS (default) vs API

**RSS feed** (`rss.arxiv.org/rss/{categories}`): Lists papers by **announcement date**, matching the website's "new listings" page. A single HTTP GET request returns all categories joined with `+`. Only today's announcements are available, so `--days` is not supported (auto-switches to API). Filters by `announce_type` — keeps `new` and `cross` listings, excludes `replace` and `replace-cross`.

**Atom API** (`export.arxiv.org/api/query`): Sorts by **submission date**, which can differ from announcement date by 1-3 days (weekend batching, indexing lag). Supports multi-day lookback via `--days`. One HTTP request per category with rate limiting (3s between requests).

The RSS feed is the default because it avoids systematic misses caused by the submission-vs-announcement date gap. See `docs/lessons.md` lesson #20 for details.

### Stage 1: Corpus Filter (always runs)

Embeds each paper's title+abstract with `all-MiniLM-L6-v2` and computes cosine similarity against 12 KMeans centroids derived from the user's ~14.5k paper library (`song_db`). Papers below the threshold (default 0.5) are removed. This is a domain filter — it answers "is this paper in my general field?" No LLM calls.

### Stage 2: Scoring (always runs, two paths)

Stage 2 assigns each surviving paper a 0-10 relevance score and a tier. **Only one scoring path runs** — they are mutually exclusive. Papers are never scored by both.

**Default path (topic-embedding scorer):**
Embeds the ~40 topic descriptions from `config.yaml` using the same sentence-transformer, computes cosine similarity between each paper and each topic, and maps the best similarity to a 0-10 score via piecewise-linear breakpoints (configurable in `config.yaml` under `topic_scorer`). Primary topics map to higher scores than secondary topics. Multi-match bonuses and project floor boosts are applied. No LLM calls.

**LLM path (`--use-llm-scoring`):**
Sends each paper's title+abstract to an LLM along with the topic descriptions. The LLM returns a JSON with score, matched_topics, and reasoning. More precise but costs ~1 LLM call per paper. Enable with `--use-llm-scoring`.

### Stage 3: Summary Generation (optional)

Generates LLM-written summaries for scored papers. Uses LLM tokens regardless of which Stage 2 path ran. Disable with `--no-summary` (outputs abstracts instead).

### Config Roles

| Config Section | Purpose | Used By |
|----------------|---------|---------|
| **category** | Fetch filter | Which arXiv categories to monitor |
| **topics** | **Core scoring** | Both local topic scorer and LLM scorer |
| **projects** | Floor booster | Guaranteed "Could Be Interesting" minimum |
| **topic_scorer** | Breakpoint calibration | Local topic scorer only |

Topics are semantic descriptions, not keywords — a paper about "stellar mass functions at z>3" matches "High-redshift galaxies" even without exact keyword overlap.

### Tier Thresholds

| Score | Tier | Action |
|-------|------|--------|
| >= 8 | Most Relevant | Detailed structured summary |
| >= 5 | Somewhat Relevant | 3-5 sentence summary |
| >= 3 | Could Be Interesting | Title and link only |
| < 3 | Not Relevant | Excluded from digest |

Thresholds are configurable in `config.yaml` under `llm_scoring.tier_thresholds`.

### Score Calibration Tools

**Local scoring (no LLM, Stage 1 + Stage 2):**

Use `test_local_score.py` to see how papers score with the local pipeline. Shows corpus filter score, topic-embedding score, matched topics, cosine similarities, and tier assignment. Useful for calibrating breakpoints in `config.yaml`.

```bash
# Score one or more papers (comma or space separated)
uv run python test_local_score.py 2602.06904,2602.06439
uv run python test_local_score.py 2602.06904 2602.06439 2602.06119

# Accept full URLs
uv run python test_local_score.py https://arxiv.org/abs/2602.06904
```

**LLM scoring:**

Use `get_llm_score.py` to check how papers score via the LLM path. Useful for comparing LLM scores against local scores.

```bash
uv run python src/get_llm_score.py 2602.04962
uv run python src/get_llm_score.py 2602.04962 --provider nvidia
uv run python src/get_llm_score.py 2602.04962 --json
```

**Corpus filter only (Stage 1):**

Use `test_local_filter.py` to check whether a paper passes the corpus filter.

```bash
uv run python test_local_filter.py --id 2602.06904
```

All tools accept arXiv IDs (e.g., `2602.04962`) or full URLs (e.g., `https://arxiv.org/abs/2602.04962`).

**LLM API health check:**

Use `check_llm_apis.py` to test all configured LLM providers. Sends a minimal prompt to each and reports status, latency, and model info. Useful for diagnosing API key issues or provider outages.

```bash
uv run python src/check_llm_apis.py              # Test all providers
uv run python src/check_llm_apis.py kimi glm      # Test specific providers
```

## Project Structure

```
project1/
├── README.md                # This file
├── docs/
│   └── pipeline-architecture.md  # Detailed pipeline architecture
├── PLAN.md                  # Implementation plan
├── config.yaml              # Configuration (topics, breakpoints, providers)
├── pyproject.toml           # Python dependencies
├── test_local_filter.py     # Calibration: Stage 1 corpus filter
├── test_local_score.py      # Calibration: Stage 1 + Stage 2 local scoring
├── src/
│   ├── main.py              # CLI entry point (pipeline orchestrator)
│   ├── config.py            # Config loader
│   ├── arxiv_fetcher.py     # arXiv RSS + API client
│   ├── local_scorer.py      # Bridge: ArxivPaper <-> song_db
│   ├── topic_scorer.py      # Stage 2: topic-embedding scorer (no-LLM path)
│   ├── scorer.py            # Stage 2: LLM scorer + tier assignment
│   ├── get_llm_score.py     # Calibration: LLM scoring
│   ├── check_llm_apis.py   # LLM API health check
│   ├── llm_client.py        # LLM abstraction
│   ├── summarizer.py        # Stage 3: summary generation
│   ├── formatter.py         # Markdown output
│   ├── state.py             # Run state tracking
│   └── logger.py            # Logging setup
├── tests/
│   ├── test_rss_fetcher.py  # RSS fetcher tests
│   └── fixtures/
│       └── rss_snapshot.xml # Saved RSS feed for deterministic testing
├── song_db/                 # Local interest model (corpus pipeline)
├── prompts/                 # LLM prompt templates
├── arxiv_digest/
│   └── archive/             # Daily outputs (YYYY/arxiv-YYYY-MM-DD.md)
└── logs/                    # Log files
```

## Running Tests

```bash
uv sync --group dev   # Install pytest (one-time)
uv run ruff check .   # Lint check
uv run pytest tests/ -v
```

## Troubleshooting

### "No papers found"
- **RSS source**: arXiv RSS may be down or have no new papers today. Try `--source api --days 1` as fallback.
- **API source**: arXiv API may be down or rate-limiting. Try again later or use `--source rss`.
- Try `--days 3` to widen the date window (automatically uses API source)
- Verify categories in `config.yaml` are valid arXiv category codes

### LLM errors (401, 429, 400)
- **401 Unauthorized**: Check that the API key env var is exported (e.g., `echo $MOONSHOT_API_KEY`)
- **429 Too Many Requests**: Rate limited; wait a minute and retry, or switch provider
- **400 Bad Request**: Model may not support your temperature setting (e.g., `kimi-k2.5` requires `temperature=1`; use `kimi-latest` instead)
- The fallback chain will automatically try the next provider on failure

### "Already processed papers"
- Use `--debug` to force re-run regardless of state
- Or delete `.state.json` to reset

## License

MIT

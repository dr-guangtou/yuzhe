# Daily arXiv Summary

Automated pipeline that monitors arXiv daily, scores paper relevance against your research interests, and generates tiered Markdown summaries.

## Features

- **Smart Filtering**: Papers are scored based on category match, topic relevance, and project mentions
- **Three-Tier System**:
  - **Most Relevant**: Detailed summaries with methodology, key findings, and references
  - **Somewhat Relevant**: 3-5 sentence summaries
  - **Could Be Interesting**: Title and link only
- **Update Mode**: Skips weekends and avoids duplicate processing
- **LLM-Powered**: Uses Gemini (or other providers) for intelligent scoring and summarization
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

3. Set up your LLM API key:
   ```bash
   export GEMINI_API_KEY="your-api-key"
   ```

## Configuration

Edit `config.yaml` to customize. The three main sections have distinct purposes:

### Categories (Fetch Filter)

Which arXiv categories to monitor for new papers:

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

### LLM Settings
```yaml
llm:
  provider: "gemini"
  model: "gemini-2.0-flash"
  api_key_env: "GEMINI_API_KEY"
  temperature: 0.3
```

### Scoring Thresholds
```yaml
scoring:
  most_relevant_threshold: 7.0
  somewhat_relevant_threshold: 5.0
  could_be_interesting_threshold: 3.0
```

## Usage

### Daily Update Mode
```bash
uv run python src/main.py --mode update
```
Checks if arXiv has new papers since last run. Skips weekends.

### Debug Mode
```bash
uv run python src/main.py --mode debug
```
Always runs full pipeline, ignoring previous state.

### Common Options
```bash
# Fetch from specific category only
uv run python src/main.py --category astro-ph.GA

# Look back multiple days
uv run python src/main.py --days 3

# Limit papers for testing
uv run python src/main.py --limit 10

# Skip LLM calls (use prefilter scoring only)
uv run python src/main.py --skip-llm

# Use mock LLM for testing
uv run python src/main.py --mock-llm

# Verbose output
uv run python src/main.py -v

# Custom output path
uv run python src/main.py --output /path/to/digest.md
```

## Output

Daily digests are saved to `arxiv_digest/archive/YYYY/YYYY-MM-DD.md`.

Example structure:
```markdown
# arXiv Daily Digest: 2026-02-05

## Summary
- **Most Relevant:** 3 papers
- **Somewhat Relevant:** 15 papers
- **Could Be Interesting:** 8 papers

## Most Relevant Papers
[Detailed summaries...]

## Somewhat Relevant Papers
[Brief summaries...]

## Could Be Interesting
[Title links...]
```

## How Scoring Works

The three configuration sections serve different purposes:

| Config | Purpose | Role |
|--------|---------|------|
| **category** | Fetch filter | Which arXiv categories to monitor |
| **topic** | **Core scoring** | LLM judges relevance to these research area descriptions |
| **project** | Floor booster | Ensures tracked projects reach "Could Be Interesting" minimum |

### Scoring Flow

```
Fetch by CATEGORY → LLM scores against TOPICS → PROJECT boosts floor → Tier
```

1. **Category Filter**: Papers are fetched from your configured arXiv categories
2. **Topic Scoring (Core)**: LLM evaluates how relevant each paper is to your topic descriptions. Topics are semantic descriptions, not keywords - the LLM judges meaning, not exact matches
3. **Project Boost**: Papers mentioning tracked projects get a small score boost (+0.5) and a guaranteed floor of "Could Be Interesting"

### Tier Thresholds

| Score | Tier | Meaning |
|-------|------|---------|
| ≥ 7 | Most Relevant | Strong topic match - definitely read |
| ≥ 5 | Somewhat Relevant | Moderate topic match - worth skimming |
| ≥ 3 | Could Be Interesting | Weak match or project boost |
| < 3 | Not Relevant | Filtered out (unless project match) |

### Degraded Mode (--skip-llm)

When LLM is unavailable, uses category as a rough proxy:
- Primary category: score 5.0 (Somewhat Relevant)
- Secondary category: score 3.5 (Could Be Interesting)
- Project boost: +1.5

## Project Structure

```
project1/
├── README.md           # This file
├── PLAN.md             # Implementation plan
├── config.yaml         # Configuration
├── pyproject.toml      # Python dependencies
├── src/
│   ├── main.py         # CLI entry point
│   ├── config.py       # Config loader
│   ├── arxiv_fetcher.py# arXiv API client
│   ├── llm_client.py   # LLM abstraction
│   ├── scorer.py       # Paper scoring
│   ├── summarizer.py   # Summary generation
│   ├── formatter.py    # Markdown output
│   ├── state.py        # Run state tracking
│   └── logger.py       # Logging setup
├── prompts/
│   ├── match_preprint.md
│   ├── summary_detailed.md
│   └── summary_brief.md
├── arxiv_digest/
│   └── archive/        # Daily outputs
├── logs/               # Log files
└── temp/               # Temporary files
```

## API Rate Limits

- **arXiv**: Minimum 3 seconds between requests (automatic)
- **Gemini**: Respect API quotas, retries with exponential backoff

## Troubleshooting

### "No papers found"
- Check if arXiv is accessible
- Verify categories in config.yaml exist
- Try increasing `--days`

### "LLM scoring failed"
- Verify API key is set: `echo $GEMINI_API_KEY`
- Check API quotas
- Use `--skip-llm` as fallback

### "Already processed papers"
- Use `--mode debug` to force re-run
- Delete `.state.json` to reset

## License

MIT

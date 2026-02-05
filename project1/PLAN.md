# Project 1: Daily arXiv Summary - Implementation Plan

## Overview

Automated pipeline that monitors arXiv daily, scores paper relevance against configured research interests, and generates tiered summaries.

**Output**: Daily Markdown digest with papers categorized as:
- **Most Relevant**: Detailed summary with methodology, key findings, and data references
- **Somewhat Relevant**: 3-5 sentence summary
- **Could Be Interesting**: Title and link only

## Architecture

```
config.yaml → Fetcher → Scorer → Summarizer → Formatter → [Notifier]
                ↓          ↓          ↓            ↓
            [Papers]  [Scored]   [Summaries]   [Markdown]
```

## Module Structure

```
project1/
├── README.md              # User documentation
├── PLAN.md                # This file
├── config.yaml            # Configuration (topics, LLM, thresholds)
├── docs/journal/          # Development journal
├── skills/arxiv-api/      # Reusable arXiv API skill
├── src/
│   ├── __init__.py
│   ├── config.py          # Load and validate YAML config
│   ├── arxiv_fetcher.py   # Query arXiv API, parse Atom XML
│   ├── llm_client.py      # Abstract interface for LLM providers
│   ├── scorer.py          # Three-tier relevance scoring
│   ├── summarizer.py      # Generate tiered summaries
│   ├── formatter.py       # Output Markdown digest
│   ├── state.py           # Track last run for update mode
│   ├── logger.py          # Logging setup
│   └── main.py            # CLI entry point
├── prompts/
│   ├── match_preprint.md  # Scoring prompt
│   ├── summary_detailed.md
│   └── summary_brief.md
├── arxiv_digest/
│   └── archive/2026/      # Daily outputs
└── temp/                  # Downloaded files (gitignored)
```

## Scoring Strategy

The three configuration sections serve distinct purposes:

| Config | Purpose | Role in Scoring |
|--------|---------|-----------------|
| **category** | Fetch filter | Determines which arXiv categories to monitor |
| **topic** | **CORE scoring** | LLM evaluates paper relevance to these descriptions |
| **project** | Booster | Ensures minimum tier floor for tracked projects |

### Flow

```
Fetch by CATEGORY → LLM scores against TOPICS → PROJECT boosts floor → Tier
```

### Phase 1: Category Filter
- Fetch papers from primary and secondary arXiv categories
- Simple pass/fail - no scoring at this stage

### Phase 2: Topic Scoring (CORE)
- LLM evaluates title + abstract against topic descriptions
- Topics are research area descriptions, not rigid keywords
- LLM judges semantic relevance (e.g., "stellar mass functions at z>3" matches "High-redshift galaxies")
- Returns 0-10 score based on topic match

### Phase 3: Project Boost
- Check if paper title mentions tracked projects
- Add small score boost (+0.5) for project mentions
- Ensure minimum tier floor of "Could Be Interesting"

### Phase 4: Tier Assignment
| Score | Tier | Notes |
|-------|------|-------|
| ≥ 7 | Most Relevant | Strong topic match |
| ≥ 5 | Somewhat Relevant | Moderate topic match |
| ≥ 3 | Could Be Interesting | Weak topic match |
| < 3 | Could Be Interesting | If project match (floor) |
| < 3 | Not Relevant | No topic or project match |

## Modes

- **Update mode** (`--mode update`): Check last run time, skip if already processed today
- **Debug mode** (`--mode debug`): Always run full pipeline regardless of state

## Development Phases

- [x] Phase 1: Directory structure and arXiv API skill
- [x] Phase 1: config.py implementation
- [x] Phase 1: arxiv_fetcher.py implementation
- [x] Phase 1: Basic main.py CLI
- [x] Phase 1: Verification (arXiv fetching works)
- [x] Phase 2: LLM client and scoring
- [x] Phase 2: Scoring verification (tier distribution reasonable)
- [x] Phase 3: Summary prompts and summarizer
- [x] Phase 3: Formatter and digest output
- [x] Phase 4: State management and logging
- [x] Phase 4: README.md documentation
- [ ] Phase 4: Update mode verification

## Implementation Notes

### Completed (2026-02-05)

**Phase 1:**
- Created full directory structure including `skills/arxiv-api/` with Claude Code skill
- Implemented `config.py` with dataclasses for all config sections
- Implemented `arxiv_fetcher.py` with Atom XML parsing and rate limiting
- Created `main.py` CLI with all options

**Phase 2:**
- Implemented `llm_client.py` with Gemini API support and mock client
- Created `prompts/match_preprint.md` scoring prompt
- Implemented `scorer.py` with three-phase scoring algorithm
- Prefilter + LLM scoring + tier assignment working

**Phase 3:**
- Created `prompts/summary_detailed.md` and `summary_brief.md`
- Implemented `summarizer.py` with fallback handling
- Implemented `formatter.py` for Markdown digest generation
- Digest output verified: proper sections, links, metadata

**Phase 4:**
- Implemented `state.py` for run tracking
- Implemented `logger.py` for file+console logging
- Created comprehensive `README.md`

### Known Limitations

- Gemini API rate limits may require fallback to prefilter scoring
- Weekend detection is timezone-naive (uses local time)
- No notification system yet (Slack/WhatsApp integration planned)

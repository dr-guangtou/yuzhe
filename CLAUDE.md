# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Yuzhe** is a personal research assistant for automating academic workflows for an astronomy researcher. The project uses LLM APIs and web services to:
- Monitor arXiv for relevant preprints and generate summaries (Project 1)
- Generate bilingual publication summaries from ORCID/ADS (Project 2)
- Future: RSS feeds from major journals, Twitter/X integration

**Domain**: Astronomy/astrophysics - topics include galaxy formation, dark matter, cosmology, observational surveys (LSST, DES, Euclid, DESI, JWST).

## Commands

No build system yet. Run Python scripts directly:
```bash
uv run python project1/src/script.py   # When dependencies exist
python3 project1/src/script.py         # For stdlib-only scripts
```

## Architecture

```
yuzhe/
├── roadmap/                  # Human-only - specifications and planning
│   ├── ROADMAP.md           # Project index with status
│   └── PROJECT[N]_SPEC.md   # Detailed specifications
├── project1/                 # Daily arXiv Summary
│   ├── config.yaml          # Topics, categories, projects to follow
│   ├── src/                 # Python scripts
│   ├── prompts/             # LLM prompt templates (separate files)
│   ├── docs/journal/        # Development journal (YYYY-MM-DD.md)
│   └── arxiv_digest/        # Output summaries
├── project2/                 # Personal Publication Record
│   └── (same structure)
└── personal_publication/     # Experimental reference implementation
```

## Workflow Rules

### The `roadmap/` folder is off-limits
Never edit files in `roadmap/`. If specification changes are needed, inform the human.

### Status-dependent workflow
Check `roadmap/ROADMAP.md` for project status:
- **"under planning" or "just beginning"**: Read the spec file (`PROJECT[N]_SPEC.md`), then draft `PLAN.md` in the project folder
- **"being tested" or "finished"**: Ignore specs, use `README.md` and `PLAN.md` in the project folder

### Development sequence
1. Study the specification file
2. Draft `PLAN.md` and get user approval
3. Create file structure per spec's "Proposed Directory Structure"
4. Create `README.md` (rephrase spec + user guide)
5. Develop with journal tracking in `docs/journal/`

### Journal and prompts
- Keep dated development journal entries: `docs/journal/2026-02-05.md`
- Log bugs, decisions, and lessons to avoid repeating mistakes
- Store all LLM prompts as separate Markdown files in `prompts/`

## External APIs

- **arXiv API**: Respect rate limits - add pauses between requests. Reference: https://info.arxiv.org/help/api/
- **ORCID API**: XML-based, for fetching publication metadata
- **Crossref API**: Fallback for publication data

## Reference Implementation

The `personal_publication/` folder contains experimental scripts worth studying:
- `orcid_fetch.py` - ORCID XML API usage pattern
- `generate_summaries.py` - Gemini CLI integration for bilingual summaries

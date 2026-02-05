# arXiv API Skill

A reusable skill for querying the arXiv API to search and fetch academic preprints.

## Overview

This skill provides two commands for interacting with arXiv:

1. **arxiv-search**: Search for papers by query terms, authors, or categories
2. **arxiv-fetch**: Fetch recent papers from specific categories

## Usage Examples

### Search for papers by topic
```
Search arXiv for papers about "galaxy formation" in the last month
```

### Search by author
```
Find all arXiv papers by author "Huang" in astro-ph.GA
```

### Fetch recent papers
```
Fetch papers from astro-ph.CO and astro-ph.GA from the last 2 days
```

## API Reference

- Base URL: `http://export.arxiv.org/api/query`
- Documentation: https://info.arxiv.org/help/api/user-manual.html

## Rate Limiting

**IMPORTANT**: arXiv requires a minimum of 3 seconds between API requests. Failure to comply may result in temporary IP blocking.

## Response Format

All responses are in Atom XML format. The skill parses this into structured data:

| Field | Description |
|-------|-------------|
| arxiv_id | Unique identifier (e.g., "2602.12345") |
| title | Paper title |
| authors | List of author names |
| abstract | Paper abstract |
| categories | All arXiv categories |
| primary_category | Main classification |
| pdf_url | Direct PDF download link |
| published | First submission date |
| updated | Last revision date |

## Astronomy Categories

| Category | Description |
|----------|-------------|
| astro-ph.CO | Cosmology and Nongalactic Astrophysics |
| astro-ph.GA | Astrophysics of Galaxies |
| astro-ph.EP | Earth and Planetary Astrophysics |
| astro-ph.HE | High Energy Astrophysical Phenomena |
| astro-ph.IM | Instrumentation and Methods |
| astro-ph.SR | Solar and Stellar Astrophysics |

## Files

```
skills/arxiv-api/
├── .claude-plugin/
│   └── plugin.json      # Skill metadata
├── README.md            # This file
└── commands/
    ├── arxiv-search.md  # Search command
    └── arxiv-fetch.md   # Fetch command
```

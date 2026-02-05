---
allowed-tools: Bash(curl:*), Bash(python3:*)
description: Fetch recent arXiv papers by category and date
---

Fetch papers from specified arXiv categories for a date range.

## Parameters
- categories: List of arXiv categories (e.g., astro-ph.CO, astro-ph.GA)
- days: Number of days to look back (default: 1)
- max_per_category: Max papers per category (default: 50)

## arXiv Category Reference (Astrophysics)
- astro-ph.CO: Cosmology and Nongalactic Astrophysics
- astro-ph.GA: Astrophysics of Galaxies
- astro-ph.EP: Earth and Planetary Astrophysics
- astro-ph.HE: High Energy Astrophysical Phenomena
- astro-ph.IM: Instrumentation and Methods
- astro-ph.SR: Solar and Stellar Astrophysics

## Date Format
arXiv uses the format: YYYYMMDDHHMMSS
Example: 202602050000 for Feb 5, 2026 at midnight

## Instructions
1. For each category, construct query: `cat:{category}`
2. Use `sortBy=submittedDate` and `sortOrder=descending` to get recent papers
3. Wait 3 seconds between requests (arXiv rate limit requirement)
4. Deduplicate papers appearing in multiple categories (use arXiv ID as key)
5. Return combined list with paper metadata

## Example curl command
```bash
# Fetch recent papers from astro-ph.CO
curl -s "http://export.arxiv.org/api/query?search_query=cat:astro-ph.CO&sortBy=submittedDate&sortOrder=descending&start=0&max_results=50" \
  -H "User-Agent: arXiv-fetch/1.0" \
  --max-time 30
```

## Rate Limiting
CRITICAL: arXiv requires at least 3 seconds between requests.
Failure to comply may result in IP blocking.

```bash
sleep 3  # Between each request
```

## Deduplication Strategy
Papers often cross-list in multiple categories. Use the arXiv ID (e.g., "2602.12345") as the unique key. When a paper appears in multiple categories:
1. Keep the first occurrence
2. Merge the category lists

## Expected Output
For each paper, return:
- arxiv_id: Unique identifier (e.g., "2602.12345")
- title: Paper title (cleaned of newlines)
- authors: List of author names
- abstract: Paper abstract
- categories: List of all categories
- primary_category: The primary classification
- pdf_url: Direct PDF link
- published: Publication timestamp
- updated: Last update timestamp

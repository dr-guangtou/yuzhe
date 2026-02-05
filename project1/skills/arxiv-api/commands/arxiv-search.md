---
allowed-tools: Bash(curl:*), Bash(python3:*)
description: Search arXiv for papers matching a query
---

Search arXiv API for papers matching the given query.

## Parameters
- query: Search terms (e.g., "galaxy formation", "au:Huang")
- max_results: Maximum papers to return (default: 10, max: 100)
- category: Optional category filter (e.g., astro-ph.GA)

## arXiv API Reference
- Base URL: http://export.arxiv.org/api/query
- Field prefixes: ti (title), au (author), abs (abstract), cat (category)
- Boolean operators: AND, OR, ANDNOT
- Rate limit: 1 request per 3 seconds

## Query Examples
```
# Search by title
ti:galaxy+formation

# Search by author
au:Huang

# Search by category
cat:astro-ph.GA

# Combined query
ti:dark+matter+AND+cat:astro-ph.CO

# Multiple authors
au:Huang+AND+au:Wang
```

## Instructions
1. Construct query URL with proper URL encoding
2. Execute curl with 30s timeout and appropriate headers
3. Parse Atom XML response
4. Return structured results: arxiv_id, title, authors, abstract, categories, pdf_url, published_date

## Example curl command
```bash
curl -s "http://export.arxiv.org/api/query?search_query=ti:galaxy+formation&start=0&max_results=10" \
  -H "User-Agent: arXiv-fetch/1.0" \
  --max-time 30
```

## Response Format
The API returns Atom XML. Key elements:
- `<entry>`: Each paper
- `<id>`: arXiv URL (extract ID from it)
- `<title>`: Paper title
- `<summary>`: Abstract
- `<author><name>`: Author names
- `<arxiv:primary_category>`: Primary category
- `<category>`: All categories
- `<link rel="alternate" type="text/html">`: Abstract page
- `<link title="pdf">`: PDF download link
- `<published>`: Publication date
- `<updated>`: Last update date

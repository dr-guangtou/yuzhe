# Lessons Learned - Project 1: Daily arXiv Summary

## Scoring and Relevance Detection

### 1. Word Boundary Matching for Acronyms
**Problem**: Substring matching causes false positives (e.g., "WST" matches "JWST").

**Solution**: Always use word boundary regex for acronym matching:
```python
pattern = r'\b' + re.escape(acronym.lower()) + r'\b'
if re.search(pattern, text_lower):
    # Match found
```

## API Integration

### 2. arXiv Uses HTTPS
The arXiv API silently redirects HTTP to HTTPS. Use HTTPS directly:
```python
ARXIV_API_BASE = "https://export.arxiv.org/api/query"
```

### 3. Python String Format vs JSON
Prompt templates containing JSON examples need escaped braces:
- Use `{{` and `}}` in templates that will be processed with `.format()`
- Or use a different templating approach (Jinja2, etc.)

### 4. LLM Rate Limits Need Graceful Fallback
When LLM scoring fails (rate limits, timeouts), fall back to prefilter scoring with neutral values rather than failing the entire pipeline.

## Architecture

### 5. Config-Driven Over Hardcoded
**Problem**: Adding a new LLM provider required editing Python code (adding to a dict in `llm_client.py`).

**Solution**: Move provider definitions to `config.yaml`. The Python code reads `(api_key_env, base_url, default_model, client_type)` from config and constructs the appropriate client. Nearly all providers use the OpenAI-compatible format, so `client_type` defaults to "openai" and only needs explicit override for Gemini.

**Result**: 8 providers defined in YAML, zero hardcoded provider knowledge in Python.

## Testing

### 6. Test with Real Papers from User
Ask users to provide examples of papers they consider relevant. Use these as ground truth for scoring validation.

### 7. Skip-LLM Mode is Not Just for Testing
The `--skip-llm` mode should produce reasonable results on its own. It's a valid production mode when:
- LLM API is unavailable
- Processing large batches quickly
- Cost-sensitive scenarios

### 8. OpenAI-Compatible base_url Must Include /v1
**Problem**: Setting base_url to `https://api.moonshot.cn` (without `/v1`) results in 404 because our code appends `/chat/completions`, making the final URL `https://api.moonshot.cn/chat/completions`.

**Solution**: Always include the version path in base_url: `https://api.moonshot.cn/v1`. The convention is `{base_url}/chat/completions`.

### 9. Moonshot kimi-k2.5 Has Temperature Restrictions
**Problem**: `kimi-k2.5` model only accepts `temperature=1` and returns empty responses for short prompts.

**Solution**: Use `kimi-latest` instead - it supports `temperature=0.3`, handles system prompts, and produces reliable responses.

## Local Interest Model (song_db)

### 10. arXiv API Rate Limiting Is Aggressive
**Problem**: Fetching papers from multiple categories in sequence triggers HTTP 429 rate limiting. Even 3-5 second delays between requests are insufficient. The rate limiter appears to track request frequency within a time window.

**Solution**: For batch fetching (e.g., evaluation negatives), use 10+ second delays between categories. Cache results to `negatives.jsonl` so subsequent runs don't re-fetch. Consider using the RSS endpoint (`rss.arxiv.org/rss/{category}`) as an alternative for recent papers, though it has fewer items and less detail.

### 11. Sentence-Transformer Embeddings Are Highly Effective for Paper Similarity
With `all-MiniLM-L6-v2` (384-dim, CPU-friendly):
- 14.5k paper embeddings computed in ~80 seconds on Apple Silicon
- KMeans K=12 produces interpretable topic clusters
- ROC-AUC 0.967 for separating corpus vs non-corpus papers
- Score separation: positives mean=0.77, negatives mean=0.39

### 12. Additive Integration Preserves Backward Compatibility
When adding a local filter to an existing scoring pipeline, make it strictly additive:
- Keep the existing LLM path fully intact
- Add `local_ranker=None` and `local_threshold=0.0` default params
- Use mode flags (`--skip-llm`, `--use-local-filter`) to opt in
- Fall back gracefully when dependencies are missing (try/except imports)

### 13. Multi-Stage Pipelines Need Clear Stage Boundaries
When refactoring to a 3-stage pipeline:
- **Make stages explicit** in code structure and documentation
- **Clearly define inputs/outputs** for each stage
- **Allow independent stage control** via CLI flags and config
- **Document execution modes** (which stages run in each mode)
- Example: Stage 1 (mandatory filter) → Stage 2 (optional LLM) → Stage 3 (optional summary)

### 14. Update Mode Should Check Output Dates, Not Just State
For digest generation systems:
- Don't just check "did we run today" (state.json)
- Check **latest output file date** and only process papers newer than that
- This prevents missing papers if state file is corrupted/deleted
- Allows recovering from failed runs by just deleting bad output file

### 15. Fallback Chain Must Include Primary Provider
**Problem**: `create_fallback_client()` used `config.llm_fallback` directly when non-empty, skipping the primary provider (`kimi`). The fallback list `[moonshot, nvidia, gemini]` didn't include `kimi`, so `moonshot` was always tried first.

**Solution**: Always prepend the primary provider to the fallback chain:
```python
primary = config.llm.provider
fallback = config.llm_fallback if config.llm_fallback else []
provider_names = [primary] + [name for name in fallback if name != primary]
```

### 16. Use Source Timestamps, Not Local Time, for Date Boundaries
**Problem**: Date cutoff used `datetime.now()` (local time) with compounding `+1` buffers in two places, causing 2 extra days of lookback and timezone-dependent behavior.

**Solution**: Pass the last digest date directly as `since_date` to the fetcher. Compare against arXiv's own UTC timestamps — both sides are in the same timezone. Keep `days` parameter only as fallback for debug/first-run mode.

### 17. Filter by Primary Category to Reject Cross-Listings
**Problem**: arXiv API returns papers cross-listed in a queried category even if their primary category is unrelated (e.g., `hep-ph` paper cross-listed to `astro-ph.CO`). This creates noise.

**Solution**: After fetching, check `paper.primary_category in configured_categories` and reject papers whose primary category is outside the configured set.

### 18. Always Provide Full Abstracts as Summary Fallback
**Problem**: When summaries are unavailable (`--no-summary`), the "Most Relevant" tier showed "*Summary not available.*" and "Somewhat Relevant" truncated abstracts to 400 chars. The arXiv API returns full abstracts — truncation was purely in the formatter.

**Solution**: Use `paper.abstract` as fallback in both tiers when no LLM summary exists.

### 19. Default Should Minimize Token Usage
For LLM-based pipelines:
- **Default mode should minimize API calls** to save tokens/cost
- Make expensive operations opt-in via flags (`--use-llm-scoring`)
- Provide fast local alternatives as default (embedding-based filtering)
- Only use LLM for final output generation by default

## Fetching

### 20. RSS Feeds Use Announcement Date; API Uses Submission Date
**Problem**: The arXiv Atom API (`export.arxiv.org/api/query`) sorts by **submission date**, not **announcement date**. Papers submitted Friday may not be announced until Monday (weekend batching), creating a 1-3 day gap. The API's date filter misses these papers because it looks at submission timestamps, not when the paper appeared on "new listings". Observed: papers 2602.07114, 2602.07159, 2602.08312 all appeared on the website but were missed by the API fetcher.

**Solution**: Use arXiv RSS feeds (`rss.arxiv.org/rss/{categories}`) as the default source. RSS lists papers by announcement date, exactly matching the website. A single GET request with `+`-joined categories returns all papers with full abstracts and `announce_type` metadata (`new`, `cross`, `replace`, `replace-cross`).

**Tradeoffs**:
- RSS only has today's announcements — no historical lookback. Keep the API as fallback for `--days N`.
- RSS includes `replace` and `replace-cross` entries (paper revisions) that must be filtered out.
- The `announce_type` and abstract are embedded in the `<description>` field as a formatted string, not structured XML — requires regex parsing.
- Auto-switch: when `--days` is specified with RSS source, automatically switch to API.

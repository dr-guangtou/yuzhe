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

### 2. Prefilter Scoring Must Reflect Priorities
**Problem**: Flat scoring (all primary = 6.0) doesn't distinguish between "paper in field" vs "paper in field about a tracked project".

**Solution**: Score should combine signals multiplicatively:
- Project match + primary category → highest score (8.5)
- Project match only → high score (7.5)
- Primary category only → medium score (6.0)
- Secondary category only → lower score (4.0)

### 3. Tier Assignment: Explicit Priority Rules
**Problem**: Complex threshold logic can miss obvious cases (project + primary = should always be top tier).

**Solution**: Add explicit priority checks before threshold checks:
```python
if project_match and primary_category_match:
    return Tier.MOST_RELEVANT  # Always, regardless of score
```

## API Integration

### 4. arXiv Uses HTTPS
The arXiv API silently redirects HTTP to HTTPS. Use HTTPS directly:
```python
ARXIV_API_BASE = "https://export.arxiv.org/api/query"
```

### 5. Python String Format vs JSON
Prompt templates containing JSON examples need escaped braces:
- Use `{{` and `}}` in templates that will be processed with `.format()`
- Or use a different templating approach (Jinja2, etc.)

### 6. LLM Rate Limits Need Graceful Fallback
When LLM scoring fails (rate limits, timeouts), fall back to prefilter scoring with neutral values rather than failing the entire pipeline.

## Architecture

### 7. Config-Driven Over Hardcoded
**Problem**: Adding a new LLM provider required editing Python code (adding to a dict in `llm_client.py`).

**Solution**: Move provider definitions to `config.yaml`. The Python code reads `(api_key_env, base_url, default_model, client_type)` from config and constructs the appropriate client. Nearly all providers use the OpenAI-compatible format, so `client_type` defaults to "openai" and only needs explicit override for Gemini.

**Result**: 8 providers defined in YAML, zero hardcoded provider knowledge in Python.

## Testing

### 8. Test with Real Papers from User
Ask users to provide examples of papers they consider relevant. Use these as ground truth for scoring validation.

### 9. Skip-LLM Mode is Not Just for Testing
The `--skip-llm` mode should produce reasonable results on its own. It's a valid production mode when:
- LLM API is unavailable
- Processing large batches quickly
- Cost-sensitive scenarios

### 10. OpenAI-Compatible base_url Must Include /v1
**Problem**: Setting base_url to `https://api.moonshot.cn` (without `/v1`) results in 404 because our code appends `/chat/completions`, making the final URL `https://api.moonshot.cn/chat/completions`.

**Solution**: Always include the version path in base_url: `https://api.moonshot.cn/v1`. The convention is `{base_url}/chat/completions`.

### 11. Moonshot kimi-k2.5 Has Temperature Restrictions
**Problem**: `kimi-k2.5` model only accepts `temperature=1` and returns empty responses for short prompts.

**Solution**: Use `kimi-latest` instead - it supports `temperature=0.3`, handles system prompts, and produces reliable responses.

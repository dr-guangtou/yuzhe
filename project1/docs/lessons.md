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

## Testing

### 7. Test with Real Papers from User
Ask users to provide examples of papers they consider relevant. Use these as ground truth for scoring validation.

### 8. Skip-LLM Mode is Not Just for Testing
The `--skip-llm` mode should produce reasonable results on its own. It's a valid production mode when:
- LLM API is unavailable
- Processing large batches quickly
- Cost-sensitive scenarios

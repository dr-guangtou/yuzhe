"""HTTP fetcher with rate limiting and User-Agent header.

Used for both RSS feed fetching and HTML page fetching (enrichment).
"""

import time
import urllib.request
import urllib.error
from typing import Optional

# Module-level timestamp for rate limiting across calls
_last_fetch_time: float = 0.0


def fetch_url(
    url: str,
    timeout: float = 30.0,
    user_agent: str = "yuzhe-journal-summary/1.0",
    delay: float = 2.0,
) -> bytes:
    """Fetch URL content with rate limiting and proper headers.

    Args:
        url: The URL to fetch.
        timeout: Request timeout in seconds.
        user_agent: User-Agent header value.
        delay: Minimum seconds between requests.

    Returns:
        Raw bytes of the response body.

    Raises:
        urllib.error.URLError: On network errors.
        urllib.error.HTTPError: On HTTP errors (4xx, 5xx).
    """
    global _last_fetch_time

    # Rate limit: wait if needed
    elapsed = time.time() - _last_fetch_time
    if elapsed < delay:
        time.sleep(delay - elapsed)

    req = urllib.request.Request(url, headers={"User-Agent": user_agent})
    with urllib.request.urlopen(req, timeout=timeout) as response:
        data = response.read()

    _last_fetch_time = time.time()
    return data

"""NASA/ADS API client for enriching publication metadata."""

import json
import logging
import os
import time
import urllib.request
import urllib.parse

from orcid_fetcher import Publication

logger = logging.getLogger(__name__)

ADS_SEARCH_URL = "https://api.adsabs.harvard.edu/v1/search/query"
ADS_FIELDS = (
    "abstract,author,bibcode,citation_count,doi,pub,pubdate,"
    "title,year,identifier,page,volume,esources"
)


class ADSFetcher:
    """Enriches publications with metadata from NASA/ADS."""

    def __init__(
        self,
        api_token: str,
        timeout: int = 30,
        rate_limit: float = 1.0,
    ):
        self.api_token = api_token
        self.timeout = timeout
        self.rate_limit = rate_limit
        self._last_request_time = 0.0

    def _throttle(self):
        """Respect rate limit between requests."""
        elapsed = time.time() - self._last_request_time
        if elapsed < self.rate_limit:
            time.sleep(self.rate_limit - elapsed)
        self._last_request_time = time.time()

    def _query(self, query_str: str) -> dict | None:
        """Execute a single ADS search query."""
        self._throttle()
        params = urllib.parse.urlencode({
            "q": query_str,
            "fl": ADS_FIELDS,
            "rows": 1,
        })
        url = f"{ADS_SEARCH_URL}?{params}"
        req = urllib.request.Request(
            url,
            headers={
                "Authorization": f"Bearer {self.api_token}",
                "Accept": "application/json",
            },
        )

        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as response:
                data = json.loads(response.read().decode("utf-8"))
            docs = data.get("response", {}).get("docs", [])
            if docs:
                return docs[0]
        except Exception as e:
            logger.warning("ADS query failed for '%s': %s", query_str, e)

        return None

    def search_by_doi(self, doi: str) -> dict | None:
        """Search ADS by DOI."""
        return self._query(f'doi:"{doi}"')

    def search_by_arxiv(self, arxiv_id: str) -> dict | None:
        """Search ADS by arXiv ID."""
        return self._query(f'arxiv:"{arxiv_id}"')

    def enrich_publication(self, pub: Publication) -> Publication:
        """Enrich a single publication with ADS metadata."""
        doc = None

        if pub.doi:
            doc = self.search_by_doi(pub.doi)
        if doc is None and pub.arxiv_id:
            doc = self.search_by_arxiv(pub.arxiv_id)

        if doc is None:
            logger.info("No ADS match for: %s", pub.title[:60])
            return pub

        pub.abstract = doc.get("abstract", pub.abstract) or ""
        pub.authors = doc.get("author", pub.authors) or []
        pub.bibcode = doc.get("bibcode", pub.bibcode) or ""
        pub.citation_count = doc.get("citation_count", pub.citation_count) or 0

        if pub.bibcode:
            pub.ads_url = (
                f"https://ui.adsabs.harvard.edu/abs/{pub.bibcode}/abstract"
            )
            # Use eprint PDF if available
            esources = doc.get("esources", [])
            if "EPRINT_PDF" in esources:
                pub.pdf_url = (
                    f"https://ui.adsabs.harvard.edu/link_gateway/"
                    f"{pub.bibcode}/EPRINT_PDF"
                )
            elif "PUB_PDF" in esources:
                pub.pdf_url = (
                    f"https://ui.adsabs.harvard.edu/link_gateway/"
                    f"{pub.bibcode}/PUB_PDF"
                )

        # Fill DOI from ADS if missing
        if not pub.doi:
            ads_dois = doc.get("doi", [])
            if ads_dois:
                pub.doi = ads_dois[0]

        # Fill journal from ADS if missing
        if not pub.journal:
            pub.journal = doc.get("pub", "") or ""

        logger.info("Enriched: %s (citations: %d)", pub.title[:60], pub.citation_count)
        return pub

    def enrich_all(self, pubs: list[Publication]) -> list[Publication]:
        """Enrich all publications with ADS metadata."""
        total = len(pubs)
        for i, pub in enumerate(pubs, 1):
            logger.info("Enriching %d/%d: %s", i, total, pub.title[:50])
            self.enrich_publication(pub)
        return pubs

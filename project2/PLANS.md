# Project 2: Personal Publication Record - Implementation Plan

## Pipeline

1. ORCID API -> publication list (title, year, journal, DOI, arXiv ID)
2. ADS API -> enrich each publication (abstract, authors, bibcode, citations, links)
3. YAML database -> merge new publications, detect updates
4. LLM summaries -> short + long, English + Chinese (4 calls per paper)
5. Portfolio builder -> per-paper Markdown documents

## Phases

- [x] Phase 0: Project setup (branch, directory structure, config, llm_client)
- [x] Phase 1: Publication fetching (ORCID + ADS)
- [x] Phase 2: Publication database (YAML store with merge)
- [x] Phase 3: Summary generation + prompt templates
- [x] Phase 4: Portfolio builder
- [x] Phase 5: Main pipeline (CLI + state tracking)
- [x] Phase 6: Verification (full pipeline tested with 37 publications, 36 summaries)

## Key Design Decisions

- ORCID as primary publication list source (reliable with Chinese names)
- ADS API for metadata enrichment (abstracts, authors, bibcodes, citations)
- Separate English summary and Chinese translation LLM calls
- Figure extraction deferred to v2 (placeholder directories created)
- llm_client.py copied from project1 (shared interface via ProviderConfig/LLMConfig/Config)

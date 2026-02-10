# Project 3: Periodic Journal Summary

## Core Workflow and Functionality:

The idea is very simple: organize a collection of online feeds for the major journals in astrophysics and cosmology, periodically fetch the most recent updates, filter them through the corpus and topics prepared in Project 1. This project should develop scripts that collect information from different journals and combine them into a collection with an uniform format and clear definitions of metadata. From this collection, the project will also generate a concise summary file with just the titles of the relevant papers as a "reminder" to the user.

### Important Notes

- When fetching information from the RSS feed or the journal's website, be patient and polite. Respect any potential rules.
- Note that different journals could have different format for the RSS feed. It may be necessary to create separated scripts for each journal.
- In addition to the RSS feed, each journal also has a "most recent issue" or similar webpage. It could be used as a backup to grab the new articles in case the RSS feed is not working.
- If the HTML path can work, it may also provide an opportunity to organize and summarize the relevant publications from the past volumes/issues, which could be a nice (but not high priority) feature to have.

## User Input / Configurations

- User configuration is summarized in `project3/config.yaml` file. It includes:
  - List of journals to follow and their RSS feed URLs.
  - Location to the `song_db` corpus data (currently in `project1` folder).
  - List of topics or projects to follow (currently in the `project1/config.yaml`).

### List of Journals:

- The Astrophysical Journal (ApJ): https://iopscience.iop.org/journal/0004-637X
  - The RSS feed: https://iopscience.iop.org/journal/rss/0004-637X
  - HTML example: Table of contents for issue 2, volume 997 
    - https://iopscience.iop.org/issue/0004-637X/997/2?alternativeContent=true 

- The Astronomical Journal (AJ): https://iopscience.iop.org/journal/1538-3881
  - The RSS feed: https://iopscience.iop.org/journal/rss/1538-3881
  - HTML example: Table of contents for issue 2, volume 171
    - https://iopscience.iop.org/issue/1538-3881/171/2?alternativeContent=true

- The Astrophysical Journal Letters (ApJL): https://iopscience.iop.org/journal/2041-8205
  - https://iopscience.iop.org/journal/rss/2041-8205
  - HTML example: Table of contents for issue 2, volume 997
    - https://iopscience.iop.org/issue/2041-8205/997/2?alternativeContent=true

- The Astrophysical Journal Supplement Series (ApJS): https://iopscience.iop.org/journal/0067-0049
  - https://iopscience.iop.org/journal/rss/0067-0049
  - HTML example: Table of contents for issue 2, volume 282
    - https://iopscience.iop.org/issue/0067-0049/282/2?alternativeContent=true

- The Monthly Notices of the Royal Astronomical Society (MNRAS): https://academic.oup.com/mnras?login=false
  - RSS Feed (Latest Issue): https://academic.oup.com/rss/site_5326/3192.xml
  - RSS Feed (Advanced Articles): https://academic.oup.com/rss/site_5326/advanceAccess_3192.xml
  - HTML Example: Table of contents for issue 3, volume 546: 
    - https://academic.oup.com/mnras/issue/546/3

- The Monthly Notices of the Royal Astronomical Society Letters (MNRASL): https://academic.oup.com/mnrasl?login=false
  - RSS Feed (Latest Issue): https://academic.oup.com/rss/site_5327/3193.xml
  - RSS Feed (Advanced Articles): https://academic.oup.com/rss/site_5327/advanceAccess_3193.xml
  - HTML Example: Table of contents for issue 1, volume 544:
    - https://academic.oup.com/mnrasl/issue/544/1

- Astronomy & Astrophysics (A&A): https://www.aanda.org/
  - RSS Feed (Recent Articles): https://feeds.feedburner.com/edp_aa?format=xml
  - RSS Feed (Press Release): https://feeds.feedburner.com/aa_pressreleases?format=xml
  - HTML Example: Table of Contents for volume 705: 
    - https://www.aanda.org/articles/aa/abs/2026/01/contents/contents.html

- Nature Astronomy (NatAstro): https://www.nature.com/natastron/
  - RSS Feed: https://www.nature.com/natastron.rss
  - HTML Example: Table of contents for issue 1, volume 10:
    - https://www.nature.com/natastron/volumes/10/issues/1

- Publications of the Astronomical Society of the Pacific (PASP): https://iopscience.iop.org/journal/1538-3873
  - RSS Feed: https://iopscience.iop.org/journal/rss/1538-3873
  - HTML Example: Table of contents for issue 1, volume 138: 
    - https://iopscience.iop.org/issue/1538-3873/138/1

- Publication of the Astronomical Society of Japan (PASJ): https://academic.oup.com/pasj
  - RSS Feed (Latest Issue): https://academic.oup.com/rss/site_5345/3211.xml
  - HTML Example: Table of contents for issue 1, volume 78:
    - https://academic.oup.com/pasj/issue/78/1

- Publications of the Astronomical Society of Australia (PASA): https://www.cambridge.org/core/journals/publications-of-the-astronomical-society-of-australia
  - RSS Feed: Unavailable
  - RSS Feed (Press Release): https://www.cambridge.org/core/blog/tag/pasa/feed/
  - HTML Example: https://www.cambridge.org/core/journals/publications-of-the-astronomical-society-of-australia/all-issues

- Journal of Cosmology and Astroparticle Physics (JCAP): https://iopscience.iop.org/journal/1475-7516
  - RSS Feed: https://iopscience.iop.org/journal/rss/1475-7516
  - HTML Example: Table of contents for issue 01, volume 2026
    - https://iopscience.iop.org/issue/1475-7516/2026/01

- The Open Journal of Astrophysics (OJA): https://astro.theoj.org/
  - RSS Feed: https://astro.theoj.org/feed
  - HTML Example: The contents of volume 8: 
    - https://astro.theoj.org/issue/11229-vol-8-2025
  
## Proposed Directory Structure:

``` Markdown
- project3
    - README.md (Basic feature, user guide)
    - PLANS.md (Core files to store the plans for agents)
    - config.yaml (keep a list of interested topics, questions, and projects to match)
    - docs (other documentation)
      - journal
        - 2026-02-04_1.md 
        - 2026-02-04_2.md
        ...
      - SUMMARY.md (summary of the status of the current development)
    - src (for the Python scripts or code in other languages)
    - summary (to keep the output results):
        - reminder (simple list of recently published and relevant paper)
            - 2026-02.md (Markdown list of relevant papers from all journals.) 
            ....
        - apj (using the acronym of the journal)
            - issue_xxx.jsonl (Summary of the relevant papers in a given issue; other formats are Ok; this should aim to support a local database with the complete abstract.)
            ...
        - apjl 
            - issue_yyy.jsonl
            ...
        ...
    - temp (Sandbox for experiment and to save temporary files downloaded from the internet. Make sure the folder is in .gitignore)
```

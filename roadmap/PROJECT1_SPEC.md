# Project 1: Daily arXiv Summary

## Core Workflow and Functionality:

- Based on a list of topics and questions provided by the user, the code should find the relevant preprints on arXiv using a score system and group them into "Most Relevant", "Somewhat Relevant", and "Could be Interesting".
- For each category, the code will provide different levels of summaries of the preprints.
  - For the most relevant one, it will generate a detailed summary.
  - For the "somewhat relevant" ones, it will provide a 3-5 sentence summary.
  - For the "could be interesting" ones, just the title is enough.
- This tool can be automatically run every day when the updates on arXiv become available.
- After getting the results, this tool should send the user the list of the daily preprint recommendations using WhatsApp or Slack.
- The summary files, including the detailed summary for the most relevant papers, will also be automatically generated and saved in multiple places, including the Obsidian Vault.

### Important Features:

- This tool should allow users to choose between different LLMs using their APIs.
- In the future, this tool could send the prompt to multiple LLMs and synthesize the summary into a more comprehensive report.
- This tool should have an `update` mode and a `debug` mode:
  - In the `update` mode: the code will check whether arXiv has been updated since the last summary was generated. If there is no new update, the code will stop.
  - In the `debug` mode: the code will try to find the relevant papers again, generate the summaries, and organize the files regardless of the arXiv update status.

## arXiv API:

- With the development of this tool, you should also develop a reusable skill to use the arXiv API for future agentic coding (vibe coding).
  - The format of this skill should follow the standard of the SKILLS.md file that could be used for Claude Code, Codex, Cursor, or other tools.
  - Similar skill has been developed, for example, the `arxiv-search` skill: https://skills.sh/yorkeccak/scientific-skills/arxiv-search
    - You should try to avoid reinventing the wheel: grab the skill and use it, and then improve or modify it.
   
- Reference for arXiv API:
  - arXiv API Access: https://info.arxiv.org/help/api/index.html
  - arXiv API Basics: https://info.arxiv.org/help/api/basics.html
  - arXiv API User's Manual: https://info.arxiv.org/help/api/user-manual.html

- It is absolutely essential to respect the rules of the arXiv API. For example, do not query too frequently, add a few seconds' stop between searches.

## Reference:

- You should try to learn from the references. Use their methods or available tools whenever possible.

- `aparture` by Josh Speagle: https://github.com/joshspeagle/aparture
  - Detailed document can be found here: https://joshspeagle.com/aparture/
  - This is a WebUI-based interactive tool with more sophisticated functionalities.
  - The score system to identify the relevant papers and the prompt systems to generate different summaries are worth studying.
  - The reasons I don't want to just use `aparture` are that:
    1. I don't want to fully rely on a WebUI-based system. Generating the summary and putting it directly into files is more to my taste.
    2. I need the option to test more LLMs, especially domestic (Chinese) open-source models.
    3. I have a few special needs for the summary.

## Proposed Directory Structure:

``` Markdown
- project1
    - README.md (Basic feature, user guide)
    - PLANS.md (Core files to store the plans for agents)
    - docs (other documentation)
      - journal
        - 2026-02-04_1.md 
        - 2026-02-04_2.md
        ...
      - SUMMARY.md (summary of the status of the current development)
    - src (for the Python scripts or code in other languages)
    - prompts (I want to maintain the key prompts separately)
        - match_preprint.md 
        - summary.md
        ...
    - arxiv_digest (folder that stores the main results)
        - input.yaml (keep a list of interested topics, questions, and projects to match)
        - archive
          - 2026
            - 2026-02-04.md 
            - 2026-02-05.md
        ...
    - temp (in case you need to download large PDF file or the LaTeX source files from the arXiv, put it in the temp folder. Make sure folder is in .gitignore)
```

## Draft of the Prompt to Get the Summary: 

```markdown
## SUMMARY FORMAT (respond in this structure):

### 1. ONE-SENTENCE SUMMARY
What is the single most important finding? (1 sentence, plain language)

### 2. KEY FINDINGS (3-4 bullet points)
- Main result with specific numbers/values if given
- Methodology highlight (data used, sample size, redshift range)
- Unexpected or surprising result
- Connection to broader field (how does this change what we know?)

### 3. KEY TECHNICAL DETAILS

- If this paper focuses on the observational side of the research:
    - Provide a brief summary of the key methodologies: What are the targets of the observation? What observables (physical properties) of these targets were measured/inferred/estimated? Did the paper do any modeling of these observations?
    - Provide a quick reference of the sample using the following format:

| Property | Value |
|----------|-------|
| Sample size | N = ? |
| Redshift range | z = ? |
| Key instruments | JWST / HST / ground-based? |
| Data products | Spectra/imaging? What type? |
| Citation to note | Most relevant reference |

- If this paper relies on numerical simulations:
    - Provide a brief summary of the key methodologies: What simulations does the paper use? What physical properties does the paper focus on? Does the paper try to reproduce any observations? If so, using what methods?
    - Provide a quick reference of the simulation used in this paper: N-body simulation or hydrodynamic simulation (or MHD simulation)? What are the key references of the simulation?

- If this paper is a pure theoretical paper:
    - Provide a brief summary of the core logic and main methods: Does the paper try to derive an analytic expression of a certain physical property? Does the paper try to build a theoretical model/picture of a certain phenomenon?

### 4. Datasets and Tools
- The authors may include the URLs to the critical databases or datasets in the preprint, as well as the links (e.g., GitHub repo link) of the important software used in this work. Parse the text, identify these links as best you can, and organize them in a list.

### 5. Knowledge Base from the Introduction:

- Summarize the main topics of the Introduction or Background sections of the preprint into a few (1-3) bullet points. Be concise. These will be used as indices for future queries.

---

**Style guidelines:**
- Use professional but accessible language
- Include specific numbers (redshifts, masses, timescales) when given
- Flag any claims that seem overstated or need verification
- Highlight connections to galaxy-halo connection work
```

# Project 2: Personal Publication Record

## Core Workflow and Functionality:

- The user, me, will provide some basic information, such as my name, ORCID, NASA/ADS publication library, and Google Scholar account.
- This tool will based on this information to organize my publication list. For each publication, this tool will gather the basic metadata into a database (e.g., a YAML or JSON file is probably fine).
- For each publication, this tool will generate two versions of summaries using a LLM API, with a positive and affirmative tone, to highlight the new results and the creative aspect of the publication. These summarizes will be used to support future grant application and personal website.
    1. Version 1: a really short version. The "Punch-line" style, with no more than 2 sentences.
    2. Version 2: a longer version that summarizes the motivation, basic technical details, and all the key findings of the publication.
- For each summary, this tool will call a LLM through API to translate the summary into Chinese.
- For each publication, this tools should try to use the public (e.g., arXiv) or open access (through the journal) version of the publication to keep a record of the key figures. The candidates of the key figures should satisfy at least one of the following criteria:
    1. The figure that has been referenced the most in the publication.
    2. The figure that has been mentioned multiple times in the "Results" or "Conclusion" section.
    3. The figure with a caption that is closely related to the main conclusion of this publication.
    4. The figure that was labelled as the key figure in the text.
- For each publication, form a Markdown document that includes:
    1. The title, main authors, and citation information (journal, issue, page).
    2. Link to the online version of the paper (e.g., the ADS link).
    3. A "punch-line" style summary with no more than 2 sentences.
    4. A longer version summary of the publication with more technical details and a complete summary of the key findings.
    5. A key figure from the publication.

### Important Features:

- This tool should allow users to choose between different LLMs using their APIs.
- In the future, this tool could send the prompt to multiple LLMs and synthesize the summary into a more comprehensive report.
- This tool should have an `update` mode and a `debug` mode:
  - In the `update` mode: the code will check whether arXiv has been updated since the last summary was generated. If there is no new update, the code will stop.
  - In the `debug` mode: the code will try to find the relevant papers again, generate the summaries, and organize the files regardless of the arXiv update status.

## Proposed Directory Structure:

``` Markdown
- project2
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
        - summary_short.md 
        - summary_long.md
        ...
    - publication_record (folder that stores the main results)
        - user_info.yaml
        - publication_list.md
        - publication_list.yaml (or any other format)
        - summary
            - model_1 (e.g., GPT, Claude Opus, Kimi 2.5)
                - summary_short_eng.md
                - summary_long_eng.md
                - summary_short_chn.md
                - summary_long_chn.md
            - model_2
            ...
        - portfolio 
        - paper_1.md (with figure imbedded; file should have a more informative name)
        - paper_1_figures
            - paper_1_fig1.png/jpg/pdf/eps
            - paper_2_fig2.png/jpg/pdf/eps 
            ...
        - paper_2.md
        - paper_2_figures
        ...
    - temp (in case you need to download large PDF file or the LaTeX source files from the arXiv, put it in the temp folder. Make sure folder is in .gitignore)
```

## Reference:

- The `summarize` tool could be interesting here: https://summarize.sh/
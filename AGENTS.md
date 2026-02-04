# Guidelines for the Yuzhe Development

- Follow the `~/.claude/CLAUDE.md` closely if this file exists.
- Start with the `roadmap/ROADMAP.md` file, which contains the overall development guideline for each project. This file contains:
    - Index (e.g., `INDEX = 1, 2...`), and title of the project.
    - The main motivation of the project.
    - The status of the project: under planning, just beginning, being tested, or finished. 
    - The detailed specification file for the project, typically named as `PROJECT[INDEX]_SPEC.md` in the same `roadmap` folder. 
    - The name of the folder for the actual development is typically `project[INDEX]` under the `yuzhe` folder.
- Important to remember that the `roadmap` folder is off-limits and only for humans.
- If the project status is "under planning" or "just beginning": study the project specification file carefully, draft a development plan in the project folder, named `PLAN.md`.
- If the project status is "being tested" or "finished", ignore the specification file in the `roadmap` folder, but rely on the `README.md` and `PLAN.md` in the project folder.
- After the user approves the plan, begin the development by creating the basic file structure laid out in the specification file (in the "Proposed Directory Structure" section).
- Always create the `README.md` file first to rephrase the content of the specification file and prepare to write the user's guide.
- Start the development and keep a "journal" in the `docs/journal` folder: summarize the model output in a Markdown file named using the current date after each iteration.
  - When you encounter a bug, summarize what happened in the journal to avoid repeating the same mistake.
- Keep the user's feedback in the journal as well. Always update `PLAN.md` and other relevant documents before addressing problems. If it is necessary to update the specification file in the `roadmap` folder, remind the human, but do not make any edits there directly.
- For each of the projects, if a prompt is required to finish a task using an LLM API, keep the text of the prompt in a separate Markdown file with a concise and clear name.

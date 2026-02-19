# Project 4: A Light-weight, Local-only, Tag-based Figure Management System

- Proposed name of this software: LaMian (as 拉面, the pulled noodle from China)


## Core Workflow and Functionality:

### Visual-Knowledge-Based (VKB)

- At the core, I want LaMian to become a local-storage only, meta-data rich database for figures and informative screenshots for scientific research. It should help a researcher to gradually accumulate a visual-based knowledge base.
  - `Vault`: I want to borrow the "Vault" concept from Obsidian for figures, which focuses on organized folders and metadata.
  - `Metadata`: For each figure saved, LaMian should have a comprehensive list of metadata stored and organized. These metadata should include: 
    - 1. Basic information from the file itself: name, type, date saved, date last updated, etc. 
    - 2. Information about the source: If it is from a publication, then it can inherit a lot of metadata from the publisher or the online database like NASA/ADS (or SciExplorer). The DOI or other main reference information should be the key. If the figure is from the internet, then the URL of the original website could be the key information. 
    - 3. Tags, caption, and user note: 
      - Tags: well-defined tag system could help organize and search the saved figures much more efficiently. A "hierachical tag" system should be supported. For example: `galaxy`, `galaxy:massive`, `galaxy:elliptical`. A saved figure could have multiple tags.
      - Caption: by default, the saved figure should have some basic description. If this figure is saved from a publication, the original caption could be sufficient. The user could provide the caption. And, if possible, LaMian can provide an API-interface to ask an LLM to generate the caption.
      - User note: this is simple, user could provide notes in Markdown format.
      - All these metadata should be searchable; User could use the tags to efficiently form a group of figures about a specific topic to study. 
  - `Link`: I want to find a way to link a figure to another using something similar to the wiki-link format used in the Markdown notes. In its simplest form, I envision that there would be a unique name or ID for each figure. And the user can use that to link one figure to others, either using a separate "tag"-like systems or in the user note.

### CLI

- There should be a build-in CLI system to support the most fundamental functionalities, such as inject a saved picture to the database, automatic or manual enrichment of the metadata, modify the tags (add, delete, rename), LLM-based caption generation, etc. 
- If possible, the CLI system should also enable database-level functions, such as searching, filtering, and basic statistics. 
- Regardless of what language was used to achieve these functionalities, a shell-based command-line interface should be available. For example: `lamian inject 2602.00001_1.png --metadata 2602.00001_1.yaml`.

### GUI 

- LaMian should also have a light-weight, simple GUI system to support the interactive management of the VKB. 
- In the most basic form, the user should be able to: 
  - Examine the vault and individual figure in different ways (e.g., list, snapshots, or slide-mode), along with all the saved information such as metadata and links. 
  - Since each figure is a real file on the harddrive, the GUI should allow the users to perform the basic file management, such as delete or copy. For example, the user can copy the figure as a file from the VKB and paste it to a different location on the harddrive.
  - The GUI should enable all the CLI functionalities to interact and manage the VKB and all the items inside. 
- Given the lack of experience in GUI design on my part, recommendations and suggestions are highly welcomed.

### High-level Features:

- Inject individual figure is not a very efficient, so there could be a few high-level features that support a more automated workflow. Here are some of my visions (regardless of the feasibility): 
  - `Screenshot Mode`: The GUI could have a screenshot button that allows the user to take a screenshot of any figure on the screen and directly inject it into the database, then ask the user to input necessary metadata and note. If the screenshot process can be window, app, or even file-sensitive, that would be even better. In my imagination, if I take a screenshot from an opened PDF file with necessary metadata (e.g., DOI, URL), LaMian could automatically grab the metadata for me. 
  - `arXiv Mode`: Eventually, I hope I can just ask LaMian to grab a certain figure from an arXiv preprint for me, and it will automatically find, save, and inject the figure with the necessary information for me. 
  - `Publication Mode`: Similar to the arXiv mode, but for other online publications. 

### AI Feature: 

- In the short term, I just hope that the design and development of LaMian is "agent-ready", meaning that the design is aware of the recent progress in agentic development and co-work and intentionally make LaMian more friendly for agent as much as possible. But AI feature should not be built-in from the beginning. I want to follow the example of Obsidian: having a solid software, then make everything available for an agent through CLI. That is precisely why I want to emphasize the importance of the CLI from the beginning.

## Reference:

- `tagespaces`: https://www.tagspaces.org/
  - The idea is similar and obviously more advanced. But it is not exactly what I want to use to drive daily research. I want a simpler and cleaner version focusing on what we actually need in scientific research. 
- `DEVONthink`: https://www.devontechnologies.com/apps/devonthink 
  - A very sophisticated software that supports most of the functionalities I need, with some AI support. But, again, it is too heavy, too complicated, and too expensive to me.

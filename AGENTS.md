# AGENTS.md (Codex and other coding agents)

This repository is GRISTORA (brewing software). Two agents work on it: Claude and ChatGPT/Codex. Read these files before you change anything:

1. `gelistirme/dongu/README.md`: the development loop v2 rules (markers, round limit, CI fixes, merge rules).
2. `marketing-site/CHATGPT-SCOPE.md`: your permission classes A, B and C and the work rules.
3. The `CLAUDE.md` file of the area that you change (`sunucu/`, `web/`, `ios/`). It has the build and test commands.

Rules for each Codex task:

- The task source is GitHub, queried live (`gelistirme/dongu/README.md` › 11). Do not use a chat history or `gelistirme/ISLER.md` as the list of tasks. Read all pages of the issues with `assigned-chatgpt`.
- Work only on the issue or PR that started the task. Push to the branch of that PR, or open a new branch `codex/<issue>-<name>`.
- Start each PR body and each comment with the loop marker (`gelistirme/dongu/README.md` › 4) with `actor=codex`.
- Do not merge. Do not change `.github/workflows/`, `adr/`, `gelistirme/ISLER.md`, `gelistirme/DURUM.md` or any `CLAUDE.md` file.
- Do not put calculation code in `web/` or `ios/`. The calculation engine runs only on the server.
- Do not add a dependency without a Claude review.
- Run the checks of the changed area. Report the real result.
- Write notes, `TODO` text and PR text in ASD-STE100 (root `CLAUDE.md` › Yazım standardı).

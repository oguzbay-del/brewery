# ChatGPT and Codex loop prompts

The owner pastes these prompts into ChatGPT once (README › 9). The rules are in `gelistirme/dongu/README.md` and `marketing-site/CHATGPT-SCOPE.md`. A rule change needs no prompt change.

## Scheduled task (every hour)

```
Repository: oguzbay-del/brewery (GitHub app), branch main. Role: ChatGPT in GRISTORA loop v2.
Do not use a chat history, a remembered task list or ISLER.md as the task source. Query GitHub live in each run.
1. Read on main: marketing-site/CHATGPT-SCOPE.md, gelistirme/dongu/README.md (section 11 "Live task source") and AGENTS.md.
2. If the issue LOOP-CONTROL has the label loop:pause, report "paused" and stop.
3. Do section 11 of README.md: list ALL open issues with assigned-chatgpt (all pages), list all open PRs and their checks, then sort the work in the order of section 11 step 3.
4. First fix your PRs that have loop:fix-requested, a Claude fix-request or ci-fix marker for the current head SHA, or a red CI. Then do your in-progress issues that have no PR. Then start ready issues (maximum 3 in this run, different files only).
5. Do not stop after one issue. Repeat steps 3 and 4 until no ready issue is left or you have no capacity.
6. Write the loop marker of README section 4 on each automatic comment and PR body (actor=chatgpt). Obey README section 5.
7. Report in max 8 lines: issues started, PRs opened, blocked items with the reason, errors of a query. If nothing changed, report "no change".
```

## PR event task (GitHub pull request activity)

Trigger: pull request activity in `oguzbay-del/brewery`: reviews, comments and commit updates.

```
Repository: oguzbay-del/brewery. Role: ChatGPT in GRISTORA loop v2, event run.
Read gelistirme/dongu/README.md on main. Look only at the PR of this event.
If the PR branch starts with chatgpt/ or codex/ and a Claude marker with kind=fix-request or kind=ci-fix exists for the current head SHA, fix the points and push to the same branch. Write a fix-done marker comment.
If the PR branch starts with claude/, do nothing (Codex reviews it).
Obey README › 5. If nothing to do, report "no change".
```

## Codex (from `@codex` comments)

Codex reads `AGENTS.md` at the root of the repository. `AGENTS.md` points to the same rules. The Codex task path through `@codex` comments is off (README section 6.3); Codex reviews Claude PRs. A Codex PR or push follows the same permission classes as ChatGPT. Codex does not merge.

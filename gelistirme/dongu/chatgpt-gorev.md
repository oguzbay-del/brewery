# ChatGPT and Codex loop prompts

The owner pastes these prompts into ChatGPT once (README › 9). The rules are in `gelistirme/dongu/README.md` and `marketing-site/CHATGPT-SCOPE.md`. A rule change needs no prompt change.

## Scheduled task (every hour)

```
Repo oguzbay-del/brewery, branch main. Role: ChatGPT in GRISTORA loop v2.
Read once, in full, and do not copy into the chat: marketing-site/CHATGPT-SCOPE.md, gelistirme/dongu/README.md (section 11), AGENTS.md.
Task source = GitHub, live. No chat history, no remembered list, no ISLER.md.
1. If LOOP-CONTROL (#344) has loop:pause: report "paused", stop. Read its 3 newest comments (checkpoint).
2. Query (README 11.1 and 11.1a): find the numbers with search_issues (label filters ready, in-progress, review) and search_prs; read the real labels with fetch_issue (max 8). Use the tools that your connector has; the labels of a search result are never the state. Check runs only for the PRs you act on.
3. Work in the order of README 11.2: fix-requested/red PRs, drafts and in-progress issues, merge ready class A/B PRs with green checks, then start up to 3 ready issues (different files only). Max 8 tool calls for each package. No subagents: do the packages one after the other (max 3 for each run, README 11.5). Run budget 40 calls: no new package after 30; the last 3 calls are for the checkpoint (README 11.3).
4. Finish rule (README 11.4): the run must change something (commit, PR, merge, label, marker). Mark a PR ready as soon as the checks of its head are green; a draft has status:in-progress. Never merge class C. Never stop at a status report while ready work exists.
5. Errors (README 11.3): limit error = stop new work, checkpoint on #344, end; transient = retry once; access error = retry once with one search_issues call, then report the exact error and stop. Never guess from a failed query.
6. Marker on each automatic comment and PR body: <!-- loop:v2 actor=chatgpt kind=<kind> ref=<sha7> round=<n> -->.
7. Report max 6 lines: changed items (issue/PR numbers), blocked items with reason, error class. If nothing changed and nothing is ready: "no change".
```

## PR event task (GitHub pull request activity)

Trigger: pull request activity in `oguzbay-del/brewery`: reviews, comments, check results and commit updates.

```
Repo oguzbay-del/brewery. Role: ChatGPT in GRISTORA loop v2, event run. Handle only the PR of this event (max 8 tool calls).
Read gelistirme/dongu/README.md sections 4, 5 and 11 once; do not copy them into the chat.
1. Branch not chatgpt/ or codex/: do nothing.
2. A Claude fix-request or ci-fix marker (or a red check) for the current head SHA: fix the points, push to the same branch, write a fix-done marker. Do not repeat an action that already has your marker for this head.
3. Checks of the current head are green and the PR is a draft of yours: mark it ready (mark_pull_request_ready_for_review); set status:review on its issue.
4. Class A or B PR, checks green, no conflict, no loop:fix-requested: merge it now. Class C: never merge.
5. Limit, transient or access error: README 11.3 (retry once, checkpoint on #344, no guessing).
6. Nothing to do: "no change".
```

## Codex (from `@codex` comments)

Codex reads `AGENTS.md` at the root of the repository. `AGENTS.md` points to the same rules. The Codex task path through `@codex` comments is off (README section 6.3); Codex reviews Claude PRs. A Codex PR or push follows the same permission classes as ChatGPT. Codex does not merge.

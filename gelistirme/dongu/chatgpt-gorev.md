# ChatGPT and Codex loop prompts

The owner pastes these prompts into ChatGPT once (README › 9). The rules are in `gelistirme/dongu/README.md` and `marketing-site/CHATGPT-SCOPE.md`. A rule change needs no prompt change.

## Scheduled task (every hour)

```
Repository: oguzbay-del/brewery (GitHub app). Role: ChatGPT in GRISTORA loop v2.
1. Read on main: marketing-site/CHATGPT-SCOPE.md and gelistirme/dongu/README.md.
2. If the issue LOOP-CONTROL has the label loop:pause, report "paused" and stop.
3. Do one pass of CHATGPT-SCOPE.md › Work loop. Write the loop marker of README › 4 on each automatic comment and PR body (actor=chatgpt).
4. First fix your PRs that have a Claude fix-request or ci-fix marker for the current head SHA, or a red CI.
5. Obey README › 5 (one owner, one action for each head SHA, max 3 fix rounds, quiet time).
6. Report in max 5 lines. If nothing changed, report "no change".
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

Codex reads `AGENTS.md` at the root of the repository. `AGENTS.md` points to the same rules. A Codex PR or push follows the same permission classes as ChatGPT. Codex does not merge.

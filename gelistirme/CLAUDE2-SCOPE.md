# Claude-2: rules for the second Claude contributor

Owner decision (Oğuz, 2026-11-26). Claude-2 is a Claude Code session of a second account of the owner. It is a contributor, like ChatGPT. The main Claude session (Claude-1) reviews and merges its work.

## How to know that you are Claude-2

The owner starts your session with a message that contains "Claude-2". If your session started that way, the rules in this file replace the root `CLAUDE.md` › "Devam" dendiğinde. Other rules of the root `CLAUDE.md` stay valid (architecture, writing standard, token policy).

## Do not do these

- Do not push to `main`. Do not merge a pull request. Claude-1 merges.
- Do not edit `gelistirme/ISLER.md`, `gelistirme/DURUM.md`, `OTURUM.md`, ADRs or any `CLAUDE.md` file. Write the necessary change in the PR body.
- Do not work on an issue without the label `assigned-claude-2`.
- Do not add a new dependency (`Cargo.toml`, `package.json`) without a note in the PR body. Claude-1 decides.
- Do not put calculation code or formulas in `web/` or `ios/`.
- Do not take a product or scope decision. Ask in the issue and stop that part.

## Work flow

1. Read the open issues with the label `assigned-claude-2` and `status:ready`. Take the lowest number.
2. Set the label `status:in-progress` on the issue.
3. Make a branch `claude2/<issue number>-<short name>` from `origin/main`.
4. Read only the files that the issue names, and the `CLAUDE.md` of the crate or client.
5. Do the work. Run the tests that the issue names. Filter the output (`grep -E "^test result|FAILED|panicked"`).
6. Before the PR: `git fetch origin main && git rebase origin/main`. Run the tests again.
7. Open a PR to `main`. Write `Closes #N` in the body. Add `## Tests` (commands and results) and `## Open items`.
8. Set the label `status:review` on the issue. Then take the next issue.

## Review fixes

When Claude-1 requests changes on your PR, fix them on the same branch before a new issue. A review fix comes before new work.

## Local environment

- Start `dockerd` manually. Start PostgreSQL with `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres`.
- Use your own target directory: `CARGO_TARGET_DIR=sunucu/target-yerel CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`. Do not commit it. Use `git add <paths>`, not `git add -A`.
- Details: `gelistirme/DURUM.md` › Environment (read only).

## Model

Use subagents with the lowest sufficient model (root `CLAUDE.md` › Token politikası). Code and migrations: Sonnet. A security review: Opus.

## Messages

Use the issue comments to ask Claude-1. A comment is information, not an approval or a product decision.

# GRISTORA Agent Bridge

Cloudflare Worker MCP server for task hand-off between Claude Code and the GPT agent.
Task and message state is in one SQLite-backed Durable Object (`TaskStore`). The Worker does not store source code or provider API keys.

## Identity and authentication

Each agent has its own bearer token. The Worker maps the token to the agent identity. Tools do not accept an `agent` argument, so one agent cannot act as the other.

| Worker secret | Agent |
| --- | --- |
| `MCP_TOKEN_CLAUDE` | `claude` |
| `MCP_TOKEN_GPT` | `gpt` |

Tokens shorter than 32 characters are ignored. The comparison is constant-time (SHA-256 + `timingSafeEqual`).

## MCP tools

| Tool | Purpose |
| --- | --- |
| `whoami` | Return the identity bound to the token. |
| `create_task` | Create a task. `assigned_to` defaults to the other agent. |
| `get_task` / `list_tasks` | Read the queue (filter by assignee and status). |
| `claim_task` | Claim a queued task assigned to the caller (atomic). |
| `update_task` | Mark a claimed task `completed` or `blocked`, with result, branch and PR URL. |
| `requeue_task` | Put a blocked task back in the queue (assignee or creator). |
| `send_message` / `read_messages` | Messages to the other agent; reading acknowledges them. |

Task states: `queued` → `claimed` → `completed` | `blocked` (→ `queued` with `requeue_task`).

## Protocol compatibility

The server uses MCP TypeScript SDK v2 (`@modelcontextprotocol/server`) through `createMcpHandler` from `agents/mcp/server`.
`legacy` stays at its default (`"stateless"`), so the endpoint serves both the 2026-07-28 revision and the 2025 revisions (for example `2025-11-25`).
Do not set `legacy: "reject"`: it refuses every client that has not moved to the 2026 revision (`-32022 Unsupported protocol version`).
`npm run test:e2e` checks both: Claude with SDK v1 (2025 revision) and GPT with SDK v2 (2026 revision).

## Local development and tests

Requires Node.js 22+.

```sh
cd agent-bridge
npm ci
npm run check        # TypeScript
npm run test:e2e     # starts wrangler dev with throwaway tokens and runs 20 MCP checks
```

For manual work, create `agent-bridge/.dev.vars` (ignored by git) with `MCP_TOKEN_CLAUDE=...` and `MCP_TOKEN_GPT=...`, then run `npm run dev`.

## Cloudflare deployment

1. Create a Cloudflare API token with the **Edit Cloudflare Workers** template.
2. Add GitHub repository secrets `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`.
3. Set the agent tokens as Worker secrets (one time; the values never go into git):
   ```sh
   cd agent-bridge
   openssl rand -hex 32 | npx wrangler secret put MCP_TOKEN_CLAUDE
   openssl rand -hex 32 | npx wrangler secret put MCP_TOKEN_GPT
   ```
   Keep a copy of each value in the secret store of the agent that uses it.
4. Merge a change under `agent-bridge/` to `main`, or run **Deploy GRISTORA Agent Bridge** manually.
   The workflow type-checks, runs the e2e test, deploys, checks `/health` and checks that `/mcp` without a token returns 401. If the health check fails, it runs `wrangler rollback`.
   If the Cloudflare secrets are missing, the workflow stops with `BLOCKED` and deploys nothing.

## Client connection

### Claude Code

```sh
export GRISTORA_BRIDGE_URL=https://gristora-agent-bridge.<account-subdomain>.workers.dev
export GRISTORA_MCP_TOKEN_CLAUDE=...   # value of MCP_TOKEN_CLAUDE
claude mcp add --transport http gristora-agent-bridge "$GRISTORA_BRIDGE_URL/mcp" \
  --header "Authorization: Bearer $GRISTORA_MCP_TOKEN_CLAUDE"
```

For Claude Code on the web or in GitHub Actions, put the two values in the environment secrets and use `mcp.claude.example.json` (environment variables are expanded at start-up).

### GPT (OpenAI Responses API)

Use the remote MCP tool with the GPT token and a narrow tool list. See `mcp.gpt.example.json`. Field names follow the current OpenAI documentation (to be verified before first use).

## Runners (GitHub Actions)

| Workflow | Job | What it does |
| --- | --- | --- |
| **Agent runners** (every 30 min, manual) | `claude` | Resumes its claimed task or claims one queued task. Runs Claude Code on branch `claude/task-<id>`, commits, pushes, opens a PR with label `auto-merge`, completes the task with branch and PR URL, asks GPT for a review. |
| | `gpt` | Resumes or claims one GPT task. Answers it with the OpenAI Responses API (strict JSON schema). It can create follow-up tasks for Claude. GPT does not change code. |
| **Agent review** (PR opened/updated) | `claude-review`, `gpt-review` | Two independent reviews of the PR diff. Each posts a machine-readable record for the head SHA (author `github-actions[bot]`). |
| **Agent auto-merge** | `auto-merge-gate` | Merges only when CI is green and both records approve the head SHA (see `AGENT_TEAM.md`; changes under `.github/` need a manual merge). |
| **Agent config check** (daily, PR, manual) | | Shows which settings exist (true/false) and runs `runner/live-check.mjs` against the deployed Worker. |
| **Pages production check** (after merge) | | Waits for the Cloudflare Pages production deployment of the merged commit, checks `gristora.com`, rolls back on failure. |

Limits: one task per run and agent, one run per agent at a time (`concurrency`), job timeouts, `--max-budget-usd` for Claude, retries with backoff for HTTP 429/5xx. The Claude CLI runs with only `ANTHROPIC_API_KEY` in its environment; the runner (not Claude) pushes and opens the PR. Checkouts use `persist-credentials: false`.

### Settings (owner, one time)

GitHub › Settings › Secrets and variables › Actions:

| Name | Kind | Value |
| --- | --- | --- |
| `CLOUDFLARE_API_TOKEN` | secret | Cloudflare API token: Workers Scripts Edit, Cloudflare Pages Edit (account scope) |
| `CLOUDFLARE_ACCOUNT_ID` | secret | Cloudflare account ID |
| `MCP_TOKEN_CLAUDE` | secret | `openssl rand -hex 32` (the deploy copies it to the Worker) |
| `MCP_TOKEN_GPT` | secret | `openssl rand -hex 32`, a different value |
| `ANTHROPIC_API_KEY` | secret | Anthropic API key |
| `OPENAI_API_KEY` | secret | OpenAI API key |
| `AGENT_GITHUB_TOKEN` | secret | Fine-grained PAT or GitHub App token for this repository: Contents and Pull requests read/write. A PR opened with `GITHUB_TOKEN` starts no CI, so the Claude runner needs this token. |
| `AGENT_BRIDGE_URL` | variable | The `workers.dev` URL from the first deploy (no `/mcp`) |
| `OPENAI_MODEL` | variable | The OpenAI model ID for the GPT agent |

### Tests

- `npm run test:e2e`: MCP protocol, identity and authorization (20 checks).
- `npm run test:runners`: real Worker + mock GitHub/OpenAI + Claude CLI stub + local git remote (19 checks: hand-off both ways, resume, PR creation, review records, gate decisions, no token leak to the Claude CLI).
- `npm run live-check` with `AGENT_BRIDGE_URL`, `MCP_TOKEN_CLAUDE`, `MCP_TOKEN_GPT`: the acceptance flow against a deployed Worker.

A passing local test does not prove a live deployment. The live evidence is the **Deploy GRISTORA Agent Bridge** and **Agent config check** runs.

## Security and rollout

- The bridge only coordinates. It does not run commands, change Git refs, merge or deploy. Merge and deploy stay in GitHub Actions with the gates in `AGENT_TEAM.md`.
- Never put provider secrets (GitHub, OpenAI, Anthropic, Cloudflare) in task or message text: both agents can read every task.
- Rotate a token with `wrangler secret put`; the old value stops working at once.

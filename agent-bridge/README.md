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

## Security and rollout

- The bridge only coordinates. It does not run commands, change Git refs, merge or deploy. Merge and deploy stay in GitHub Actions with the gates in `AGENT_TEAM.md`.
- Never put provider secrets (GitHub, OpenAI, Anthropic, Cloudflare) in task or message text: both agents can read every task.
- Rotate a token with `wrangler secret put`; the old value stops working at once.

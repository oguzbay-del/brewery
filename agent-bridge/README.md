# GRISTORA Agent Bridge

Cloudflare Worker MCP server for shared task hand-off between Claude Code and the GPT agent.

## MCP tools

- `create_task`: assign work to `claude` or `gpt`.
- `list_tasks`: inspect the shared queue.
- `claim_task`: claim a queued task assigned to the caller.
- `update_task`: report completion or a blocker, with branch and PR metadata.
- `send_message`: send a review request or hand-off message to the other agent.
- `read_messages`: read and acknowledge messages for an agent.

Task and message state is stored in a SQLite-backed Durable Object. The Worker does not store source code or provider API keys.

## Local development

Requires Node.js 22+.

```sh
npm install
npx wrangler secret put MCP_SHARED_TOKEN
npm run dev
```

Set a long random value for `MCP_SHARED_TOKEN`. Do not commit it.

## Cloudflare deployment

1. Create a Cloudflare API token with Workers Scripts:Edit and account-level Workers access.
2. Add GitHub repository secrets:
   - `CLOUDFLARE_API_TOKEN`
   - `CLOUDFLARE_ACCOUNT_ID`
   - `MCP_SHARED_TOKEN` (set separately as a Worker secret with Wrangler)
3. Run the **Deploy GRISTORA Agent Bridge** workflow or merge a change under `agent-bridge/` to `main`.
4. Set the Worker secret:
   `npx wrangler secret put MCP_SHARED_TOKEN --config agent-bridge/wrangler.jsonc`
5. Verify `https://gristora-agent-bridge.<account-subdomain>.workers.dev/health` and the authenticated `/mcp` endpoint.

## Client connection

Use the deployed URL ending in `/mcp` and HTTP Bearer authentication.

Claude Code example (keep the token in the local environment, not in the repository):

```sh
claude mcp add --transport http gristora-agent-bridge \
  https://gristora-agent-bridge.<account-subdomain>.workers.dev/mcp \
  --header "Authorization: Bearer $GRISTORA_MCP_TOKEN"
```

For the GPT agent, configure the same remote MCP URL and Bearer token in the OpenAI Responses API MCP tool. Use a narrow allowed-tools list.

## Security and rollout

The shared token is a bootstrap credential. Before production use, replace it with per-agent credentials and role-based authorization. Never expose GitHub, OpenAI, Anthropic, or Cloudflare provider secrets through MCP tool results.

The bridge only coordinates tasks. It does not run shell commands, modify Git refs, merge pull requests, or deploy production. Those actions remain in isolated GitHub Actions workflows with explicit policy gates.

// Bindings for the Worker. `cloudflare:workers` exposes `env` typed as `Cloudflare.Env`,
// so the bindings are declared on that interface, not on a local type.
declare namespace Cloudflare {
  interface Env {
    TASK_STORE: DurableObjectNamespace<import("./index").TaskStore>;
    /** Bearer token for the Claude agent. Set with `wrangler secret put MCP_TOKEN_CLAUDE`. */
    MCP_TOKEN_CLAUDE?: string;
    /** Bearer token for the GPT agent. Set with `wrangler secret put MCP_TOKEN_GPT`. */
    MCP_TOKEN_GPT?: string;
  }
}

interface Env extends Cloudflare.Env {}

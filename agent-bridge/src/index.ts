import { DurableObject, env as workerEnv } from "cloudflare:workers";
import { McpServer } from "@modelcontextprotocol/server";
import type { ServerContext } from "@modelcontextprotocol/server";
import { createMcpHandler } from "agents/mcp/server";
import { z } from "zod";

export const AGENTS = ["claude", "gpt"] as const;
export type AgentName = (typeof AGENTS)[number];
export const TASK_STATUSES = ["queued", "claimed", "completed", "blocked"] as const;
export type TaskStatus = (typeof TASK_STATUSES)[number];

export interface Task {
  id: string;
  created_by: AgentName;
  assigned_to: AgentName;
  title: string;
  description: string;
  priority: number;
  status: TaskStatus;
  created_at: string;
  updated_at: string;
  result: string | null;
  branch: string | null;
  pull_request_url: string | null;
}

export interface Message {
  id: string;
  task_id: string | null;
  from_agent: AgentName;
  to_agent: AgentName;
  body: string;
  created_at: string;
}

export class TaskStore extends DurableObject<Env> {
  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    this.ctx.storage.sql.exec(`
      CREATE TABLE IF NOT EXISTS tasks (
        id TEXT PRIMARY KEY,
        created_by TEXT NOT NULL CHECK (created_by IN ('claude', 'gpt')),
        assigned_to TEXT NOT NULL CHECK (assigned_to IN ('claude', 'gpt')),
        title TEXT NOT NULL,
        description TEXT NOT NULL,
        priority INTEGER NOT NULL DEFAULT 3,
        status TEXT NOT NULL DEFAULT 'queued'
          CHECK (status IN ('queued', 'claimed', 'completed', 'blocked')),
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        result TEXT,
        branch TEXT,
        pull_request_url TEXT
      );
      CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        task_id TEXT,
        from_agent TEXT NOT NULL CHECK (from_agent IN ('claude', 'gpt')),
        to_agent TEXT NOT NULL CHECK (to_agent IN ('claude', 'gpt')),
        body TEXT NOT NULL,
        created_at TEXT NOT NULL,
        read_at TEXT
      );
      CREATE INDEX IF NOT EXISTS idx_tasks_agent_status
        ON tasks(assigned_to, status, priority, created_at);
      CREATE INDEX IF NOT EXISTS idx_messages_recipient
        ON messages(to_agent, read_at, created_at);
    `);
  }

  createTask(input: {
    created_by: AgentName;
    assigned_to: AgentName;
    title: string;
    description: string;
    priority: number;
  }): Task {
    const now = new Date().toISOString();
    const task: Task = {
      id: crypto.randomUUID(),
      created_by: input.created_by,
      assigned_to: input.assigned_to,
      title: input.title,
      description: input.description,
      priority: input.priority,
      status: "queued",
      created_at: now,
      updated_at: now,
      result: null,
      branch: null,
      pull_request_url: null
    };
    this.ctx.storage.sql.exec(
      `INSERT INTO tasks
       (id, created_by, assigned_to, title, description, priority, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      task.id, task.created_by, task.assigned_to, task.title, task.description,
      task.priority, task.status, task.created_at, task.updated_at
    );
    return task;
  }

  getTask(id: string): Task | null {
    const rows = this.ctx.storage.sql
      .exec("SELECT * FROM tasks WHERE id = ?", id)
      .toArray() as unknown as Task[];
    return rows[0] ?? null;
  }

  listTasks(agent?: AgentName, status?: TaskStatus): Task[] {
    let query = "SELECT * FROM tasks WHERE 1 = 1";
    const values: string[] = [];
    if (agent) {
      query += " AND assigned_to = ?";
      values.push(agent);
    }
    if (status) {
      query += " AND status = ?";
      values.push(status);
    }
    query += " ORDER BY priority ASC, created_at ASC LIMIT 100";
    return this.ctx.storage.sql.exec(query, ...values).toArray() as unknown as Task[];
  }

  /** Atomic: a Durable Object runs one request at a time and SQL calls are synchronous. */
  claimTask(id: string, agent: AgentName): Task | null {
    const cursor = this.ctx.storage.sql.exec(
      `UPDATE tasks SET status = 'claimed', updated_at = ?
       WHERE id = ? AND assigned_to = ? AND status = 'queued'`,
      new Date().toISOString(), id, agent
    );
    if (cursor.rowsWritten === 0) return null;
    return this.getTask(id);
  }

  updateTask(input: {
    id: string;
    agent: AgentName;
    status: "completed" | "blocked";
    result: string;
    branch?: string;
    pull_request_url?: string;
  }): Task | null {
    const cursor = this.ctx.storage.sql.exec(
      `UPDATE tasks
       SET status = ?, result = ?, branch = COALESCE(?, branch),
           pull_request_url = COALESCE(?, pull_request_url), updated_at = ?
       WHERE id = ? AND assigned_to = ? AND status = 'claimed'`,
      input.status, input.result, input.branch ?? null,
      input.pull_request_url ?? null, new Date().toISOString(), input.id, input.agent
    );
    if (cursor.rowsWritten === 0) return null;
    return this.getTask(input.id);
  }

  /** A blocked task goes back to the queue so that its assignee can claim it again. */
  requeueTask(id: string, agent: AgentName): Task | null {
    const cursor = this.ctx.storage.sql.exec(
      `UPDATE tasks SET status = 'queued', updated_at = ?
       WHERE id = ? AND status = 'blocked' AND (assigned_to = ? OR created_by = ?)`,
      new Date().toISOString(), id, agent, agent
    );
    if (cursor.rowsWritten === 0) return null;
    return this.getTask(id);
  }

  sendMessage(input: {
    task_id?: string;
    from_agent: AgentName;
    to_agent: AgentName;
    body: string;
  }): string {
    const id = crypto.randomUUID();
    this.ctx.storage.sql.exec(
      `INSERT INTO messages (id, task_id, from_agent, to_agent, body, created_at)
       VALUES (?, ?, ?, ?, ?, ?)`,
      id, input.task_id ?? null, input.from_agent, input.to_agent,
      input.body, new Date().toISOString()
    );
    return id;
  }

  readMessages(agent: AgentName): Message[] {
    const rows = this.ctx.storage.sql
      .exec(
        `SELECT id, task_id, from_agent, to_agent, body, created_at FROM messages
         WHERE to_agent = ? AND read_at IS NULL ORDER BY created_at ASC LIMIT 100`,
        agent
      )
      .toArray() as unknown as Message[];
    const now = new Date().toISOString();
    this.ctx.storage.transactionSync(() => {
      for (const row of rows) {
        this.ctx.storage.sql.exec("UPDATE messages SET read_at = ? WHERE id = ?", now, row.id);
      }
    });
    return rows;
  }
}

const agentSchema = z.enum(AGENTS);

function text(value: unknown, isError = false) {
  return {
    content: [{ type: "text" as const, text: typeof value === "string" ? value : JSON.stringify(value) }],
    ...(isError ? { isError: true } : {})
  };
}

/** The calling agent comes from the verified bearer token, never from a tool argument. */
function callerOf(ctx: ServerContext): AgentName {
  const id = ctx.http?.authInfo?.clientId;
  if (id === "claude" || id === "gpt") return id;
  throw new Error("Unauthenticated MCP request.");
}

function otherAgent(agent: AgentName): AgentName {
  return agent === "claude" ? "gpt" : "claude";
}

export function createServer(): McpServer {
  const server = new McpServer({ name: "gristora-agent-bridge", version: "0.2.0" });
  const store = () => workerEnv.TASK_STORE.get(workerEnv.TASK_STORE.idFromName("gristora-shared"));

  server.registerTool(
    "whoami",
    {
      description: "Return the agent identity bound to the bearer token of this connection.",
      inputSchema: z.object({}),
      annotations: { readOnlyHint: true }
    },
    async (_args, ctx) => text({ agent: callerOf(ctx) })
  );

  server.registerTool(
    "create_task",
    {
      description:
        "Create a task in the shared GRISTORA queue. assigned_to defaults to the other agent, so a caller hands work over.",
      inputSchema: z.object({
        assigned_to: agentSchema.optional(),
        title: z.string().min(3).max(160),
        description: z.string().min(10).max(12000),
        priority: z.number().int().min(1).max(5).default(3)
      })
    },
    async (input, ctx) => {
      const caller = callerOf(ctx);
      const task = await store().createTask({
        created_by: caller,
        assigned_to: input.assigned_to ?? otherAgent(caller),
        title: input.title,
        description: input.description,
        priority: input.priority
      });
      return text(task);
    }
  );

  server.registerTool(
    "get_task",
    {
      description: "Read one task by id, including its result, branch and pull request URL.",
      inputSchema: z.object({ id: z.string().uuid() }),
      annotations: { readOnlyHint: true }
    },
    async ({ id }, ctx) => {
      callerOf(ctx);
      const task = await store().getTask(id);
      return task ? text(task) : text("Task not found.", true);
    }
  );

  server.registerTool(
    "list_tasks",
    {
      description: "List shared GRISTORA tasks, optionally filtered by assignee and status.",
      inputSchema: z.object({
        assigned_to: agentSchema.optional(),
        status: z.enum(TASK_STATUSES).optional()
      }),
      annotations: { readOnlyHint: true }
    },
    async (input, ctx) => {
      callerOf(ctx);
      return text(await store().listTasks(input.assigned_to, input.status));
    }
  );

  server.registerTool(
    "claim_task",
    {
      description: "Claim a queued task that is assigned to the calling agent.",
      inputSchema: z.object({ id: z.string().uuid() })
    },
    async ({ id }, ctx) => {
      const task = await store().claimTask(id, callerOf(ctx));
      return task ? text(task) : text("Task is not queued for this agent.", true);
    }
  );

  server.registerTool(
    "update_task",
    {
      description:
        "Mark a task that the caller claimed as completed or blocked, with the result, branch and pull request URL.",
      inputSchema: z.object({
        id: z.string().uuid(),
        status: z.enum(["completed", "blocked"]),
        result: z.string().min(1).max(12000),
        branch: z.string().max(200).optional(),
        pull_request_url: z.string().url().optional()
      })
    },
    async (input, ctx) => {
      const task = await store().updateTask({ ...input, agent: callerOf(ctx) });
      return task ? text(task) : text("Task is not claimed by this agent.", true);
    }
  );

  server.registerTool(
    "requeue_task",
    {
      description: "Put a blocked task back in the queue. The assignee or the creator can do this.",
      inputSchema: z.object({ id: z.string().uuid() })
    },
    async ({ id }, ctx) => {
      const task = await store().requeueTask(id, callerOf(ctx));
      return task ? text(task) : text("Task is not blocked or the caller is not its assignee or creator.", true);
    }
  );

  server.registerTool(
    "send_message",
    {
      description: "Send a message or a review request to the other GRISTORA agent.",
      inputSchema: z.object({
        task_id: z.string().uuid().optional(),
        body: z.string().min(1).max(12000)
      })
    },
    async (input, ctx) => {
      const caller = callerOf(ctx);
      const id = await store().sendMessage({
        task_id: input.task_id,
        from_agent: caller,
        to_agent: otherAgent(caller),
        body: input.body
      });
      return text({ message_id: id, to_agent: otherAgent(caller) });
    }
  );

  server.registerTool(
    "read_messages",
    {
      description: "Read and acknowledge the unread messages addressed to the calling agent.",
      inputSchema: z.object({})
    },
    async (_args, ctx) => text(await store().readMessages(callerOf(ctx)))
  );

  return server;
}

// `legacy` stays at its default ("stateless"): clients that still speak the 2025 MCP
// revisions must keep working. `legacy: "reject"` would refuse them.
const mcpHandler = createMcpHandler(() => createServer(), { route: "/mcp" });

async function sha256(value: string): Promise<ArrayBuffer> {
  return crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
}

/** Constant-time match of the bearer token against each configured agent token. */
export async function authenticate(request: Request, env: Env): Promise<AgentName | null> {
  const header = request.headers.get("authorization") ?? "";
  const match = /^Bearer\s+(\S+)$/i.exec(header);
  if (!match) return null;
  // Two equal tokens make the identity ambiguous: refuse both agents (fail closed).
  if (env.MCP_TOKEN_CLAUDE && env.MCP_TOKEN_CLAUDE === env.MCP_TOKEN_GPT) return null;
  const given = await sha256(match[1]);
  const tokens: Array<[AgentName, string | undefined]> = [
    ["claude", env.MCP_TOKEN_CLAUDE],
    ["gpt", env.MCP_TOKEN_GPT]
  ];
  let found: AgentName | null = null;
  for (const [agent, token] of tokens) {
    if (!token || token.length < 32) continue;
    if (crypto.subtle.timingSafeEqual(given, await sha256(token))) found = agent;
  }
  return found;
}

export default {
  async fetch(request: Request, env: Env, _ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === "/health") {
      return Response.json({ service: "gristora-agent-bridge", status: "ok" });
    }
    if (url.pathname !== "/mcp") {
      return new Response("Not found", { status: 404 });
    }
    const agent = await authenticate(request, env);
    if (!agent) {
      return new Response("Unauthorized", {
        status: 401,
        headers: { "WWW-Authenticate": 'Bearer realm="gristora-agent-bridge"' }
      });
    }
    return mcpHandler.fetch(request, {
      authInfo: { token: "redacted", clientId: agent, scopes: ["tasks", "messages"] }
    });
  }
} satisfies ExportedHandler<Env>;

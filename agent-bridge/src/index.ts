import { DurableObject } from "cloudflare:workers";
import { McpServer } from "@modelcontextprotocol/server";
import { createMcpHandler } from "agents/mcp/server";
import { z } from "zod";

interface Env {
  TASK_STORE: DurableObjectNamespace<TaskStore>;
  MCP_SHARED_TOKEN: string;
}

type AgentName = "claude" | "gpt";
type TaskStatus = "queued" | "claimed" | "completed" | "blocked";

interface TaskInput {
  id: string;
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

export class TaskStore extends DurableObject<Env> {
  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    this.ctx.storage.sql.exec(`
      CREATE TABLE IF NOT EXISTS tasks (
        id TEXT PRIMARY KEY,
        assigned_to TEXT NOT NULL CHECK (assigned_to IN ('claude', 'gpt')),
        title TEXT NOT NULL,
        description TEXT NOT NULL,
        priority INTEGER NOT NULL DEFAULT 3,
        status TEXT NOT NULL DEFAULT 'queued',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        result TEXT,
        branch TEXT,
        pull_request_url TEXT
      );
      CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        task_id TEXT,
        from_agent TEXT NOT NULL,
        to_agent TEXT NOT NULL,
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

  async createTask(input: {
    assigned_to: AgentName;
    title: string;
    description: string;
    priority: number;
  }): Promise<TaskInput> {
    const now = new Date().toISOString();
    const task: TaskInput = {
      id: crypto.randomUUID(),
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
       (id, assigned_to, title, description, priority, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
      task.id, task.assigned_to, task.title, task.description, task.priority,
      task.status, task.created_at, task.updated_at
    );
    return task;
  }

  async listTasks(agent?: AgentName, status?: TaskStatus): Promise<TaskInput[]> {
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
    return this.ctx.storage.sql.exec(query, ...values).toArray() as TaskInput[];
  }

  async claimTask(id: string, agent: AgentName): Promise<TaskInput | null> {
    const now = new Date().toISOString();
    this.ctx.storage.sql.exec(
      "UPDATE tasks SET status = 'claimed', updated_at = ? WHERE id = ? AND assigned_to = ? AND status = 'queued'",
      now, id, agent
    );
    const rows = this.ctx.storage.sql.exec(
      "SELECT * FROM tasks WHERE id = ? AND assigned_to = ?",
      id, agent
    ).toArray() as TaskInput[];
    return rows[0] ?? null;
  }

  async updateTask(input: {
    id: string;
    agent: AgentName;
    status: "completed" | "blocked";
    result: string;
    branch?: string;
    pull_request_url?: string;
  }): Promise<boolean> {
    const now = new Date().toISOString();
    const result = this.ctx.storage.sql.exec(
      `UPDATE tasks
       SET status = ?, result = ?, branch = COALESCE(?, branch),
           pull_request_url = COALESCE(?, pull_request_url), updated_at = ?
       WHERE id = ? AND assigned_to = ? AND status = 'claimed'`,
      input.status, input.result, input.branch ?? null,
      input.pull_request_url ?? null, now, input.id, input.agent
    );
    return result.rowsWritten > 0;
  }

  async sendMessage(input: {
    task_id?: string;
    from_agent: AgentName;
    to_agent: AgentName;
    body: string;
  }): Promise<string> {
    const id = crypto.randomUUID();
    this.ctx.storage.sql.exec(
      "INSERT INTO messages (id, task_id, from_agent, to_agent, body, created_at) VALUES (?, ?, ?, ?, ?, ?)",
      id, input.task_id ?? null, input.from_agent, input.to_agent,
      input.body, new Date().toISOString()
    );
    return id;
  }

  async readMessages(agent: AgentName): Promise<Array<Record<string, unknown>>> {
    const rows = this.ctx.storage.sql.exec(
      "SELECT id, task_id, from_agent, to_agent, body, created_at FROM messages WHERE to_agent = ? AND read_at IS NULL ORDER BY created_at ASC LIMIT 100",
      agent
    ).toArray() as Array<Record<string, unknown>>;
    const now = new Date().toISOString();
    for (const row of rows) {
      this.ctx.storage.sql.exec("UPDATE messages SET read_at = ? WHERE id = ?", now, row.id);
    }
    return rows;
  }
}

function createServer(env: Env) {
  const server = new McpServer({
    name: "gristora-agent-bridge",
    version: "0.1.0"
  });
  const store = env.TASK_STORE.get(env.TASK_STORE.idFromName("gristora-shared"));

  server.registerTool(
    "create_task",
    {
      description: "Create a task for Claude or GPT in the shared GRISTORA development queue.",
      inputSchema: {
        assigned_to: z.enum(["claude", "gpt"]),
        title: z.string().min(3).max(160),
        description: z.string().min(10).max(12000),
        priority: z.number().int().min(1).max(5).default(3)
      }
    },
    async (input) => {
      const task = await store.createTask(input);
      return { content: [{ type: "text", text: JSON.stringify(task) }] };
    }
  );

  server.registerTool(
    "list_tasks",
    {
      description: "List shared GRISTORA tasks, optionally filtered by agent and status.",
      inputSchema: {
        agent: z.enum(["claude", "gpt"]).optional(),
        status: z.enum(["queued", "claimed", "completed", "blocked"]).optional()
      }
    },
    async (input) => {
      const tasks = await store.listTasks(input.agent, input.status);
      return { content: [{ type: "text", text: JSON.stringify(tasks) }] };
    }
  );

  server.registerTool(
    "claim_task",
    {
      description: "Claim a queued task assigned to the calling agent.",
      inputSchema: {
        id: z.string().uuid(),
        agent: z.enum(["claude", "gpt"])
      }
    },
    async (input) => {
      const task = await store.claimTask(input.id, input.agent);
      return { content: [{ type: "text", text: task ? JSON.stringify(task) : "Task not available for this agent." }] };
    }
  );

  server.registerTool(
    "update_task",
    {
      description: "Mark a claimed task completed or blocked and attach the result, branch, and pull request URL.",
      inputSchema: {
        id: z.string().uuid(),
        agent: z.enum(["claude", "gpt"]),
        status: z.enum(["completed", "blocked"]),
        result: z.string().min(1).max(12000),
        branch: z.string().max(200).optional(),
        pull_request_url: z.string().url().optional()
      }
    },
    async (input) => {
      const updated = await store.updateTask(input);
      return { content: [{ type: "text", text: updated ? "Task updated." : "Task was not claimed by this agent." }] };
    }
  );

  server.registerTool(
    "send_message",
    {
      description: "Send a message or review request to the other GRISTORA agent.",
      inputSchema: {
        task_id: z.string().uuid().optional(),
        from_agent: z.enum(["claude", "gpt"]),
        to_agent: z.enum(["claude", "gpt"]),
        body: z.string().min(1).max(12000)
      }
    },
    async (input) => {
      if (input.from_agent === input.to_agent) {
        return { content: [{ type: "text", text: "Sender and recipient must be different agents." }], isError: true };
      }
      const id = await store.sendMessage(input);
      return { content: [{ type: "text", text: JSON.stringify({ message_id: id }) }] };
    }
  );

  server.registerTool(
    "read_messages",
    {
      description: "Read unread messages addressed to Claude or GPT.",
      inputSchema: { agent: z.enum(["claude", "gpt"]) }
    },
    async ({ agent }) => {
      const messages = await store.readMessages(agent);
      return { content: [{ type: "text", text: JSON.stringify(messages) }] };
    }
  );

  return server;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === "/health") {
      return Response.json({ service: "gristora-agent-bridge", status: "ok" });
    }
    if (url.pathname !== "/mcp") {
      return new Response("Not found", { status: 404 });
    }
    const expected = `Bearer ${env.MCP_SHARED_TOKEN}`;
    if (!env.MCP_SHARED_TOKEN || request.headers.get("authorization") !== expected) {
      return new Response("Unauthorized", { status: 401 });
    }
    const handler = createMcpHandler(() => createServer(env), {
      route: "/mcp",
      legacy: "reject"
    });
    return handler(request, env, ctx);
  }
} satisfies ExportedHandler<Env>;

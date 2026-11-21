// End-to-end test: starts `wrangler dev` with throwaway tokens, then drives the bridge
// with a 2025-revision client (MCP SDK v1) as Claude and a 2026-revision client (SDK v2) as GPT.
import { spawn } from "node:child_process";
import { randomBytes } from "node:crypto";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Client as LegacyClient } from "@modelcontextprotocol/sdk/client/index.js";
import { StreamableHTTPClientTransport as LegacyTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";
import { Client, StreamableHTTPClientTransport } from "@modelcontextprotocol/client";

const port = Number(process.env.E2E_PORT ?? 8787);
const base = `http://127.0.0.1:${port}`;
const tokens = { claude: randomBytes(24).toString("hex"), gpt: randomBytes(24).toString("hex") };

const dev = spawn(
  "npx",
  ["wrangler", "dev", "--ip", "127.0.0.1", "--port", String(port),
    "--persist-to", mkdtempSync(join(tmpdir(), "bridge-state-")),
    "--var", `MCP_TOKEN_CLAUDE:${tokens.claude}`, "--var", `MCP_TOKEN_GPT:${tokens.gpt}`],
  { stdio: ["ignore", "pipe", "pipe"], detached: true }
);
let devLog = "";
dev.stdout.on("data", (d) => (devLog += d));
dev.stderr.on("data", (d) => (devLog += d));
const stop = () => { try { process.kill(-dev.pid, "SIGTERM"); } catch {} };

let failures = 0;
const check = (condition, name) => {
  console.log(`${condition ? "PASS" : "FAIL"} ${name}`);
  if (!condition) failures++;
};
const json = (result) => {
  try { return JSON.parse(result.content[0].text); } catch { return result.content[0].text; }
};
const auth = (token) => ({ requestInit: { headers: { Authorization: `Bearer ${token}` } } });

try {
  let ready = false;
  for (let i = 0; i < 60 && !ready; i++) {
    try { ready = (await fetch(`${base}/health`)).ok; } catch {}
    if (!ready) await new Promise((r) => setTimeout(r, 1000));
  }
  if (!ready) throw new Error(`wrangler dev did not start:\n${devLog.slice(-2000)}`);

  const endpoint = new URL(`${base}/mcp`);
  const anonymous = await fetch(endpoint, { method: "POST", body: "{}", headers: { "content-type": "application/json" } });
  check(anonymous.status === 401, "request without a token returns 401");
  const wrong = await fetch(endpoint, { method: "POST", body: "{}", headers: { "content-type": "application/json", authorization: "Bearer wrong" } });
  check(wrong.status === 401, "request with a wrong token returns 401");

  const claude = new LegacyClient({ name: "e2e-claude", version: "1.0.0" });
  await claude.connect(new LegacyTransport(endpoint, auth(tokens.claude)));
  check(true, "2025-revision client (SDK v1) connects");
  const tools = (await claude.listTools()).tools.map((t) => t.name);
  check(tools.length === 9, `9 tools listed (${tools.sort().join(", ")})`);
  check(json(await claude.callTool({ name: "whoami", arguments: {} })).agent === "claude", "whoami returns claude");

  const task = json(await claude.callTool({ name: "create_task", arguments: { title: "Review PR", description: "Review the bridge for security issues." } }));
  check(task.assigned_to === "gpt" && task.created_by === "claude", "claude creates a task for gpt (default assignee)");
  check((await claude.callTool({ name: "claim_task", arguments: { id: task.id } })).isError === true, "claude cannot claim a gpt task");
  check((await claude.callTool({ name: "claim_task", arguments: { id: task.id, agent: "gpt" } })).isError === true, "an agent argument cannot change the identity");

  const gpt = new Client({ name: "e2e-gpt", version: "1.0.0" });
  await gpt.connect(new StreamableHTTPClientTransport(endpoint, auth(tokens.gpt)));
  check(true, "2026-revision client (SDK v2) connects");
  check(json(await gpt.callTool({ name: "whoami", arguments: {} })).agent === "gpt", "whoami returns gpt");
  const queued = json(await gpt.callTool({ name: "list_tasks", arguments: { assigned_to: "gpt", status: "queued" } }));
  check(queued.some((t) => t.id === task.id), "gpt sees the queued task");
  check(json(await gpt.callTool({ name: "claim_task", arguments: { id: task.id } })).status === "claimed", "gpt claims the task");
  check((await gpt.callTool({ name: "claim_task", arguments: { id: task.id } })).isError === true, "a second claim is refused");
  const blocked = json(await gpt.callTool({ name: "update_task", arguments: { id: task.id, status: "blocked", result: "Needs a token." } }));
  check(blocked.status === "blocked", "gpt marks the task blocked");
  check(json(await claude.callTool({ name: "requeue_task", arguments: { id: task.id } })).status === "queued", "the creator requeues the blocked task");
  await gpt.callTool({ name: "claim_task", arguments: { id: task.id } });
  const done = json(await gpt.callTool({ name: "update_task", arguments: { id: task.id, status: "completed", result: "No blockers.", branch: "fix/bridge", pull_request_url: "https://github.com/oguzbay-del/brewery/pull/1" } }));
  check(done.status === "completed" && done.branch === "fix/bridge", "gpt completes the task with branch and PR");
  await gpt.callTool({ name: "send_message", arguments: { task_id: task.id, body: "Review done." } });
  const reverse = json(await gpt.callTool({ name: "create_task", arguments: { title: "Fix nit", description: "Rename the variable in index.ts." } }));
  check(reverse.assigned_to === "claude", "gpt creates a task for claude");
  const inbox = json(await claude.callTool({ name: "read_messages", arguments: {} }));
  check(inbox.length === 1 && inbox[0].from_agent === "gpt", "claude reads the gpt message");
  check(json(await claude.callTool({ name: "read_messages", arguments: {} })).length === 0, "read messages are acknowledged");
  const invalid = await claude.callTool({ name: "create_task", arguments: { title: "x", description: "short" } }).catch(() => ({ isError: true }));
  check(invalid.isError === true, "schema validation rejects invalid input");

  await claude.close();
  await gpt.close();
} catch (error) {
  failures++;
  console.error(error);
} finally {
  stop();
}
console.log(failures === 0 ? "ALL PASSED" : `${failures} FAILED`);
process.exit(failures === 0 ? 0 : 1);

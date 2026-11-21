// Live acceptance check against a deployed bridge (no model calls).
//   AGENT_BRIDGE_URL, MCP_TOKEN_CLAUDE, MCP_TOKEN_GPT  -> node runner/live-check.mjs
// It proves: health, 401 without a token, per-agent identity, authorization between agents,
// Claude -> GPT hand-off, GPT claim/answer, GPT -> Claude hand-off, Claude resume and completion.
// The tasks it creates are marked "[acceptance]" and end in the completed state.
import { bridge, fetchRetry, requireEnv } from "./lib.mjs";

const env = requireEnv("AGENT_BRIDGE_URL", "MCP_TOKEN_CLAUDE", "MCP_TOKEN_GPT");
const base = env.AGENT_BRIDGE_URL.replace(/\/+$/, "");
let failures = 0;
const check = (ok, name, evidence = "") => {
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${evidence ? ` — ${evidence}` : ""}`);
  if (!ok) failures++;
};
const expectError = async (p) => p.then(() => false, () => true);

const health = await fetchRetry(`${base}/health`);
const body = await health.json().catch(() => ({}));
check(health.ok && body.status === "ok", "GET /health", `${health.status} ${JSON.stringify(body)}`);
const anon = await fetch(`${base}/mcp`, { method: "POST", headers: { "content-type": "application/json" }, body: "{}" });
check(anon.status === 401, "POST /mcp without a token is refused", String(anon.status));

const claude = await bridge(base, env.MCP_TOKEN_CLAUDE, "live-check-claude");
const gpt = await bridge(base, env.MCP_TOKEN_GPT, "live-check-gpt");
try {
  check((await claude.call("whoami")).agent === "claude", "Claude token maps to claude");
  check((await gpt.call("whoami")).agent === "gpt", "GPT token maps to gpt");
  await gpt.call("read_messages");
  await claude.call("read_messages");

  const stamp = new Date().toISOString();
  const t1 = await claude.call("create_task", {
    title: `[acceptance] review request ${stamp}`,
    description: "Acceptance check: GPT claims this task, answers it and hands a task back to Claude."
  });
  check(t1.assigned_to === "gpt" && t1.created_by === "claude", "1. Claude creates a task for GPT", t1.id);
  check(await expectError(claude.call("claim_task", { id: t1.id })), "Claude cannot claim a GPT task");

  const c1 = await gpt.call("claim_task", { id: t1.id });
  check(c1.status === "claimed", "2a. GPT claims the task");
  check(await expectError(gpt.call("claim_task", { id: t1.id })), "a second claim is refused");
  await gpt.call("send_message", { task_id: t1.id, body: "Acceptance: review done, no blocking findings." });
  const d1 = await gpt.call("update_task", { id: t1.id, status: "completed", result: "Acceptance review: approve." });
  check(d1.status === "completed", "2b. GPT reviews and responds (task completed, message sent)");

  const t2 = await gpt.call("create_task", {
    title: `[acceptance] follow-up ${stamp}`,
    description: "Acceptance check: Claude claims this task, stops, resumes it and completes it."
  });
  check(t2.assigned_to === "claude" && t2.created_by === "gpt", "3. GPT creates a task for Claude", t2.id);
  check(await expectError(gpt.call("update_task", { id: t2.id, status: "completed", result: "x" })), "GPT cannot complete a Claude task");

  const inbox = await claude.call("read_messages");
  check(inbox.some((m) => m.task_id === t1.id && m.from_agent === "gpt"), "Claude receives the GPT message");
  await claude.call("claim_task", { id: t2.id });
  // Simulate an interrupted run: a new session finds the claimed task and resumes it.
  const resumed = await claude.call("list_tasks", { assigned_to: "claude", status: "claimed" });
  check(resumed.some((t) => t.id === t2.id), "4a. Claude finds its claimed task after an interruption");
  const d2 = await claude.call("update_task", { id: t2.id, status: "completed", result: "Acceptance: resumed and completed." });
  check(d2.status === "completed", "4b. Claude resumes and completes the task");
  const final = await gpt.call("get_task", { id: t2.id });
  check(final.status === "completed" && final.result.includes("resumed"), "GPT sees the completed result");
} finally {
  await claude.close();
  await gpt.close();
}
console.log(failures === 0 ? "LIVE ACCEPTANCE: ALL PASSED" : `LIVE ACCEPTANCE: ${failures} FAILED`);
process.exit(failures === 0 ? 0 : 1);

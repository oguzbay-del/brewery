// Offline integration test of the runners: real Worker (wrangler dev), mock GitHub and
// OpenAI APIs, a stub for the Claude CLI and a local bare git remote. No secrets needed.
import { spawn, spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";
import http from "node:http";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { evaluate } from "../../.github/scripts/agent-auto-merge.mjs";

const here = resolve(import.meta.dirname);
const port = 8790;
const bridgeUrl = `http://127.0.0.1:${port}`;
const tokens = { claude: randomBytes(24).toString("hex"), gpt: randomBytes(24).toString("hex") };
const SHA = "c".repeat(40);
let failures = 0;
const check = (ok, name) => { console.log(`${ok ? "PASS" : "FAIL"} ${name}`); if (!ok) failures++; };

// ---- mock GitHub + OpenAI ----
const state = { comments: [], labels: new Set(), prs: [], openaiCalls: [] };
const pr7 = { number: 7, title: "agent(claude): demo", body: "demo", head: { ref: "claude/demo", sha: SHA, repo: { full_name: "o/r" } }, base: { ref: "main", repo: { full_name: "o/r", default_branch: "main" } }, state: "open", draft: false, mergeable: true, mergeable_state: "clean", labels: [] };
const mock = http.createServer(async (req, res) => {
  let body = "";
  for await (const chunk of req) body += chunk;
  const json = (code, value) => { res.writeHead(code, { "content-type": "application/json" }); res.end(JSON.stringify(value)); };
  const url = new URL(req.url, "http://x");
  if (req.method === "POST" && url.pathname === "/v1/responses") {
    const input = JSON.parse(body);
    state.openaiCalls.push({ auth: req.headers.authorization, model: input.model, schema: input.text.format.name, strict: input.text.format.strict });
    const out = input.text.format.name === "review"
      ? { verdict: "approve", summary: "mock gpt review", findings: [{ severity: "nit", file: "a.ts", title: "naming", detail: "minor" }] }
      : { status: "completed", result: "mock gpt answer", message_to_other_agent: "GPT answer ready.", follow_up_tasks: [{ title: "Implement mock follow-up", description: "Change agent-output.txt and run the tests." }] };
    return json(200, { output: [{ type: "message", content: [{ type: "output_text", text: JSON.stringify(out) }] }] });
  }
  if (url.pathname === "/repos/o/r/pulls/7") {
    if (req.headers.accept === "application/vnd.github.diff") { res.writeHead(200); return res.end("diff --git a/x b/x\n+demo\n"); }
    return json(200, pr7);
  }
  if (req.method === "POST" && url.pathname === "/repos/o/r/issues/7/comments") {
    state.comments.push({ id: state.comments.length + 1, user: { login: "github-actions[bot]" }, created_at: new Date().toISOString(), body: JSON.parse(body).body });
    return json(201, {});
  }
  if (/\/repos\/o\/r\/issues\/\d+\/labels/.test(url.pathname)) {
    if (req.method === "POST") JSON.parse(body).labels.forEach((l) => state.labels.add(l));
    else state.labels.delete(decodeURIComponent(url.pathname.split("/").pop()));
    return json(200, []);
  }
  if (req.method === "GET" && url.pathname === "/repos/o/r/pulls") return json(200, state.prs.filter((p) => `o:${p.head}` === url.searchParams.get("head")));
  if (req.method === "POST" && url.pathname === "/repos/o/r/pulls") {
    const p = { ...JSON.parse(body), number: 100 + state.prs.length, html_url: `https://github.com/o/r/pull/${100 + state.prs.length}` };
    state.prs.push(p);
    return json(201, p);
  }
  json(404, { message: `mock: ${req.method} ${url.pathname}` });
});
await new Promise((r) => mock.listen(8791, "127.0.0.1", r));

// ---- real Worker ----
const dev = spawn("npx", ["wrangler", "dev", "--ip", "127.0.0.1", "--port", String(port),
  "--persist-to", mkdtempSync(join(tmpdir(), "bridge-state-")),
    "--var", `MCP_TOKEN_CLAUDE:${tokens.claude}`, "--var", `MCP_TOKEN_GPT:${tokens.gpt}`],
  { cwd: resolve(here, ".."), stdio: "ignore", detached: true });
const stop = () => { try { process.kill(-dev.pid, "SIGTERM"); } catch {} mock.close(); };

const baseEnv = {
  PATH: process.env.PATH, HOME: process.env.HOME,
  GITHUB_REPOSITORY: "o/r", GITHUB_API_URL: "http://127.0.0.1:8791", GITHUB_TOKEN: "gh-test",
  OPENAI_BASE_URL: "http://127.0.0.1:8791/v1", OPENAI_API_KEY: "sk-test", OPENAI_MODEL: "test-model",
  ANTHROPIC_API_KEY: "ant-test", CLAUDE_BIN: join(here, "claude-stub.mjs"),
  AGENT_BRIDGE_URL: bridgeUrl, MCP_TOKEN_CLAUDE: tokens.claude, MCP_TOKEN_GPT: tokens.gpt
};
// Async: the mock server runs in this process, so a synchronous spawn would deadlock.
const run = (script, args, extra = {}, cwd = resolve(here, "..")) => new Promise((done) => {
  const child = spawn("node", [resolve(here, "../runner", script), ...args], { cwd, env: { ...baseEnv, ...extra } });
  let out = "";
  child.stdout.on("data", (d) => (out += d));
  child.stderr.on("data", (d) => (out += d));
  child.on("close", (code) => done({ code, out: out.trim() }));
});

try {
  for (let i = 0; i < 60; i++) { try { if ((await fetch(`${bridgeUrl}/health`)).ok) break; } catch {} await new Promise((r) => setTimeout(r, 1000)); }

  // Live acceptance flow (MCP mechanics) against the local Worker.
  const live = await run("live-check.mjs", []);
  check(live.code === 0 && live.out.includes("ALL PASSED"), "live-check.mjs passes against the Worker");
  if (live.code !== 0) console.log(live.out);

  // Independent reviews -> machine-readable records -> gate.
  const g = await run("gpt.mjs", ["review", "7"], { POST_REVIEW: "true" });
  check(g.code === 0 && state.comments.some((c) => c.body.includes('"agent":"gpt"')), "GPT review is posted as a record");
  check(state.openaiCalls[0]?.auth === "Bearer sk-test" && state.openaiCalls[0]?.strict === true && state.openaiCalls[0]?.model === "test-model", "GPT uses the Responses API with a strict JSON schema");
  const c = await run("claude.mjs", ["review", "7"], { POST_REVIEW: "true" });
  const claudeComment = state.comments.find((x) => x.body.includes('"agent":"claude"'));
  check(c.code === 0 && claudeComment, "Claude review is posted as a record");
  check(claudeComment?.body.includes("leaked=none"), "the Claude CLI gets no GitHub/OpenAI/GPT token");
  check(state.labels.has("review:gpt-ok") && state.labels.has("review:claude-ok"), "review labels are synced");

  const green = { pr: { ...pr7, labels: [{ name: "auto-merge" }] }, files: ["agent-bridge/src/index.ts"],
    checkRuns: [{ name: "typecheck, bundle, MCP e2e", status: "completed", conclusion: "success" }], statuses: [],
    workflowRuns: [{ name: "Agent bridge checks", head_sha: SHA, run_number: 1, status: "completed", conclusion: "success" }],
    comments: state.comments };
  check(evaluate(green).merge === true, "gate: both reviews + green CI -> merge");
  check(evaluate({ ...green, checkRuns: [{ name: "typecheck, bundle, MCP e2e", status: "completed", conclusion: "failure" }] }).merge === false, "gate: a failing test blocks the merge");
  check(evaluate({ ...green, comments: state.comments.filter((x) => !x.body.includes('"agent":"gpt"')) }).merge === false, "gate: a missing review blocks the merge");

  // GPT answers a task and hands work to Claude.
  const { bridge } = await import("../runner/lib.mjs");
  const cl = await bridge(bridgeUrl, tokens.claude, "test-claude");
  const t = await cl.call("create_task", { title: "Design tests", description: "Design tests for the demo module." });
  const gt = await run("gpt.mjs", ["tasks"]);
  const t1 = await cl.call("get_task", { id: t.id });
  check(gt.code === 0 && t1.status === "completed" && t1.result.includes("mock gpt answer"), "GPT runner claims and completes a task");
  const followUps = await cl.call("list_tasks", { assigned_to: "claude", status: "queued" });
  check(followUps.some((x) => x.title === "Implement mock follow-up" && x.created_by === "gpt"), "GPT runner creates a follow-up task for Claude");

  // Claude implements on its own branch and opens a PR (local bare remote, mock GitHub).
  const work = mkdtempSync(join(tmpdir(), "runner-"));
  const bare = join(work, "remote.git");
  const repo = join(work, "repo");
  const sh = (cmd, cwd) => spawnSync("sh", ["-c", cmd], { cwd, encoding: "utf8" });
  sh(`git init -q --bare ${bare} && git init -q -b main ${repo}`, work);
  writeFileSync(join(repo, "README.md"), "demo\n");
  sh(`git add . && git -c user.name=t -c user.email=t@t commit -qm init && git remote add origin ${bare} && git push -q origin main && git fetch -q origin`, repo);
  const ct = await run("claude.mjs", ["tasks"], { AGENT_GITHUB_TOKEN: "pat-test", AGENT_GIT_REMOTE: bare }, repo);
  const branches = sh(`git ls-remote --heads ${bare}`, repo).stdout;
  const task = (await cl.call("list_tasks", { assigned_to: "claude" })).find((x) => x.title === "Implement mock follow-up");
  check(ct.code === 0 && /refs\/heads\/claude\/task-[0-9a-f]{8}/.test(branches), "Claude runner pushes an isolated claude/task-* branch");
  check(state.prs.length === 1 && state.labels.has("auto-merge"), "Claude runner opens a PR with the auto-merge label");
  check(task?.status === "completed" && task.pull_request_url === state.prs[0]?.html_url && task.branch?.startsWith("claude/task-"), "Claude runner completes the task with branch and PR URL");
  check(task?.result.includes("leaked=none") && task.result.includes("mcp=true"), "the Claude CLI gets the bridge MCP config and no other token");
  const gp = await bridge(bridgeUrl, tokens.gpt, "test-gpt");
  const inbox = await gp.call("read_messages");
  check(inbox.some((m) => m.body.includes(state.prs[0]?.html_url)), "Claude asks GPT to review the PR");
  check((await gp.call("list_tasks", { assigned_to: "gpt", status: "queued" })).some((x) => x.title === "Review stub change"), "Claude runner creates a follow-up task for GPT");

  // Resume: a claimed task is picked up before queued tasks.
  const r = await gp.call("create_task", { title: "Resume me", description: "This task is claimed, then resumed." });
  await cl.call("claim_task", { id: r.id });
  const ct2 = await run("claude.mjs", ["tasks"], { AGENT_GITHUB_TOKEN: "pat-test", AGENT_GIT_REMOTE: bare }, repo);
  check(ct2.out.includes(`Resuming task ${r.id}`) && (await cl.call("get_task", { id: r.id })).status === "completed", "Claude runner resumes a claimed task");

  // Missing configuration stops the runner with BLOCKED.
  const blocked = await run("gpt.mjs", ["tasks"], { OPENAI_API_KEY: "" });
  check(blocked.code === 1 && blocked.out.includes("BLOCKED"), "a missing secret stops the runner with BLOCKED");
  await cl.close(); await gp.close();
} catch (error) {
  failures++;
  console.error(error);
} finally {
  stop();
}
console.log(failures === 0 ? "ALL PASSED" : `${failures} FAILED`);
process.exit(failures === 0 ? 0 : 1);

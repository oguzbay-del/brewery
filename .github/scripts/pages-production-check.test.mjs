import { test } from "node:test";
import assert from "node:assert/strict";
import http from "node:http";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const script = fileURLToPath(new URL("./pages-production-check.mjs", import.meta.url));
const SHA = "d".repeat(40);
const OLD = "e".repeat(40);

function server(handler) {
  return new Promise((resolve) => {
    const s = http.createServer(handler);
    s.listen(0, "127.0.0.1", () => resolve(s));
  });
}
function run(envExtra) {
  return new Promise((resolve) => {
    const child = spawn("node", [script], { env: { PATH: process.env.PATH, ...envExtra } });
    let out = "";
    child.stdout.on("data", (d) => (out += d));
    child.stderr.on("data", (d) => (out += d));
    child.on("close", (code) => resolve({ code, out }));
  });
}

async function scenario({ siteHealthy, deployStatus = "success" }) {
  const calls = [];
  let rolledBack = false;
  const site = await server((req, res) => {
    const ok = siteHealthy || rolledBack;
    res.writeHead(ok ? 200 : 500, { "content-type": "text/html" });
    res.end(ok ? '<meta name="robots" content="index, follow"><title>GRISTORA</title>' : "error");
  });
  const cf = await server((req, res) => {
    calls.push(`${req.method} ${req.url}`);
    res.setHeader("content-type", "application/json");
    if (req.method === "POST" && req.url.endsWith("/deployments/old-1/rollback")) {
      rolledBack = true;
      return res.end(JSON.stringify({ success: true, result: {} }));
    }
    res.end(JSON.stringify({ success: true, result: [
      { id: "new-1", url: "https://new.example", deployment_trigger: { metadata: { commit_hash: SHA } }, latest_stage: { name: "deploy", status: deployStatus } },
      { id: "old-1", deployment_trigger: { metadata: { commit_hash: OLD } }, latest_stage: { name: "deploy", status: "success" } }
    ] }));
  });
  const r = await run({
    CLOUDFLARE_API_TOKEN: "t", CLOUDFLARE_ACCOUNT_ID: "a",
    CLOUDFLARE_API_BASE: `http://127.0.0.1:${cf.address().port}`,
    PRODUCTION_URL: `http://127.0.0.1:${site.address().port}`,
    EXPECTED_SHA: SHA, HEALTH_ROUTES: "/,/pricing/", HEALTH_RETRY_SECONDS: "0", DEPLOY_POLL_SECONDS: "0", DEPLOY_WAIT_SECONDS: "5"
  });
  site.close(); cf.close();
  return { ...r, calls, rolledBack };
}

test("healthy deployment: passes, no rollback", async () => {
  const r = await scenario({ siteHealthy: true });
  assert.equal(r.code, 0, r.out);
  assert.equal(r.rolledBack, false);
  assert.match(r.out, /Production deployment new-1 .* success/);
});

test("failed health check: rolls back to the previous successful deployment and fails", async () => {
  const r = await scenario({ siteHealthy: false });
  assert.equal(r.code, 1);
  assert.equal(r.rolledBack, true);
  assert.ok(r.calls.includes("POST /accounts/a/pages/projects/gristora/deployments/old-1/rollback"));
  assert.match(r.out, /Rollback done; health check passes/);
});

test("failed Cloudflare build: fails without touching production", async () => {
  const r = await scenario({ siteHealthy: true, deployStatus: "failure" });
  assert.equal(r.code, 1);
  assert.equal(r.rolledBack, false);
  assert.match(r.out, /ended with failure/);
});

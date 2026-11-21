// Cloudflare Pages production check after a merge to main. No dependencies; Node 22+.
// 1. Find the production deployment for EXPECTED_SHA and wait until it finishes.
// 2. Check the production URL (status, brand, robots, key routes).
// 3. If the health check fails, roll back to the previous successful production deployment.
// Without CLOUDFLARE_API_TOKEN/CLOUDFLARE_ACCOUNT_ID it runs only step 2 and says so.

const env = process.env;
const API = env.CLOUDFLARE_API_BASE || "https://api.cloudflare.com/client/v4";
const PROJECT = env.PAGES_PROJECT || "gristora";
const PROD = (env.PRODUCTION_URL || "https://gristora.com").replace(/\/+$/, "");
const SHA = env.EXPECTED_SHA || env.GITHUB_SHA || "";
const WAIT_MS = Number(env.DEPLOY_WAIT_SECONDS || 1200) * 1000;
const POLL_MS = Number(env.DEPLOY_POLL_SECONDS || 20) * 1000;
const ROUTES = (env.HEALTH_ROUTES || "/,/breweries/,/homebrewers/,/pricing/,/how-it-works/,/tr/").split(",");
const hasCf = Boolean(env.CLOUDFLARE_API_TOKEN && env.CLOUDFLARE_ACCOUNT_ID);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const lines = [];
const log = (s) => { console.log(s); lines.push(s); };

async function cf(path, init = {}) {
  const res = await fetch(`${API}/accounts/${env.CLOUDFLARE_ACCOUNT_ID}/pages/projects/${PROJECT}${path}`, {
    ...init,
    headers: { authorization: `Bearer ${env.CLOUDFLARE_API_TOKEN}`, "content-type": "application/json" },
    signal: AbortSignal.timeout(30_000)
  });
  const body = await res.json().catch(() => ({}));
  if (!res.ok || body.success === false) throw new Error(`Cloudflare ${init.method ?? "GET"} ${path} -> ${res.status} ${JSON.stringify(body.errors ?? [])}`);
  return body.result;
}

export async function health(base = PROD) {
  const problems = [];
  for (const route of ROUTES) {
    try {
      const res = await fetch(base + route, { redirect: "follow", signal: AbortSignal.timeout(20_000), headers: { "cache-control": "no-cache" } });
      const html = await res.text();
      if (res.status !== 200) problems.push(`${route} -> ${res.status}`);
      else if (!/GRISTORA/.test(html)) problems.push(`${route} has no GRISTORA brand`);
      if (route === "/" && base === PROD && !/<meta name="robots" content="index, follow">/.test(html)) problems.push("/ is not index, follow");
    } catch (error) {
      problems.push(`${route} -> ${error.message}`);
    }
  }
  return problems;
}

const isSuccess = (d) => d?.latest_stage?.name === "deploy" && d.latest_stage.status === "success";
const isFailed = (d) => ["failure", "canceled"].includes(d?.latest_stage?.status);
const shaOf = (d) => d?.deployment_trigger?.metadata?.commit_hash ?? "";

async function main() {
  let deployment = null;
  let previous = null;
  if (!hasCf) {
    log("::warning::CLOUDFLARE_API_TOKEN/CLOUDFLARE_ACCOUNT_ID are not set: deployment correlation and rollback are not available. Only the health check runs.");
  } else if (SHA) {
    const deadline = Date.now() + WAIT_MS;
    while (Date.now() < deadline) {
      const list = await cf(`/deployments?env=production&per_page=25`);
      deployment = list.find((d) => shaOf(d) === SHA) ?? null;
      previous = list.find((d) => shaOf(d) !== SHA && isSuccess(d)) ?? null;
      if (deployment && (isSuccess(deployment) || isFailed(deployment))) break;
      log(`Waiting for the production deployment of ${SHA.slice(0, 7)} (${deployment?.latest_stage?.name ?? "not started"}/${deployment?.latest_stage?.status ?? "-"})`);
      await sleep(POLL_MS);
    }
    if (!deployment) throw new Error(`No Cloudflare Pages production deployment for ${SHA} within ${WAIT_MS / 1000}s.`);
    if (!isSuccess(deployment)) throw new Error(`Production deployment ${deployment.id} ended with ${deployment.latest_stage?.status}. Cloudflare keeps the previous deployment live.`);
    log(`Production deployment ${deployment.id} for ${SHA.slice(0, 7)}: success (${deployment.url})`);
  }

  // Cloudflare's edge can serve the previous version for a short time; retry the check.
  let problems = [];
  for (let i = 0; i < 6; i++) {
    problems = await health();
    if (problems.length === 0) break;
    await sleep(Number(env.HEALTH_RETRY_SECONDS || 15) * 1000);
  }
  if (problems.length === 0) {
    log(`Health check passed: ${PROD} (${ROUTES.length} routes)`);
    return 0;
  }
  log(`::error::Health check failed: ${problems.join("; ")}`);

  if (hasCf && deployment && previous) {
    log(`Rolling back production to ${previous.id} (${shaOf(previous).slice(0, 7)})`);
    await cf(`/deployments/${previous.id}/rollback`, { method: "POST" });
    const after = await health();
    log(after.length === 0 ? "Rollback done; health check passes on the previous deployment." : `::error::Rollback done, but the health check still fails: ${after.join("; ")}`);
  } else {
    log("::error::No rollback: Cloudflare credentials or a previous successful deployment are missing.");
  }
  return 1;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main()
    .then(async (code) => {
      if (env.GITHUB_STEP_SUMMARY) (await import("node:fs")).appendFileSync(env.GITHUB_STEP_SUMMARY, `## Pages production\n\n${lines.join("\n\n")}\n`);
      process.exit(code);
    })
    .catch((error) => {
      console.log(`::error::${error.message}`);
      process.exit(1);
    });
}

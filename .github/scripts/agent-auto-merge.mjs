// Guarded auto-merge for agent pull requests (Claude + GPT). No dependencies; Node 22+.
// The decision logic (`evaluate`) is pure and is covered by agent-auto-merge.test.mjs.
// The gate fails closed: any missing, pending or failed signal means "do not merge".

export const LABELS = {
  optIn: "auto-merge",
  claudeReview: "review:claude-ok",
  gptReview: "review:gpt-ok",
  block: "do-not-merge"
};

/** Changed paths -> workflows (by `name:`) that must have a successful run on the head SHA. */
export const REQUIRED_WORKFLOWS = [
  { pattern: /^marketing-site\//, workflow: "Marketing site checks" },
  { pattern: /^\.github\/workflows\/marketing-site\.yml$/, workflow: "Marketing site checks" },
  { pattern: /^(agent-bridge\/|AGENT_TEAM\.md$|\.github\/workflows\/agent-bridge-checks\.yml$)/, workflow: "Agent bridge checks" },
  { pattern: /^\.github\/(workflows|scripts)\//, workflow: "Actionlint" },
  { pattern: /^(sunucu\/|formuller\/testler\/vektorler\/)/, workflow: "sunucu" },
  { pattern: /^(web\/|sozlesme\/istemci\/typescript\/src\/)/, workflow: "web" },
  { pattern: /^ios\//, workflow: "ios" },
  { pattern: /^(sozlesme\/|sunucu\/)/, workflow: "sozlesme" },
  { pattern: /^(altyapi\/|sunucu\/|Makefile$)/, workflow: "altyapi" },
  { pattern: /^(plan|arayuz|formuller|araclar|tasarim-sistemi|erisilebilirlik)\//, workflow: "belgeler" }
];

/** GPT branches may change only the public marketing website (marketing-site/CHATGPT-SCOPE.md). */
export const GPT_ALLOWED = [/^marketing-site\//, /^web-stratejisi\//];

/** Check runs that belong to this gate. They are not evidence for or against a merge. */
export const SELF_CHECKS = new Set(["auto-merge-gate", "auto-merge-clear-reviews"]);

const OK_CONCLUSIONS = new Set(["success", "neutral", "skipped"]);

export function evaluate({ pr, files, checkRuns, statuses, workflowRuns }) {
  const reasons = [];
  const labels = new Set(pr.labels.map((l) => l.name));
  const branch = pr.head.ref;
  const agent = branch.startsWith("claude/") ? "claude" : branch.startsWith("gpt/") ? "gpt" : null;

  if (pr.state !== "open") reasons.push("PR is not open");
  if (pr.draft) reasons.push("PR is a draft");
  if (pr.head.repo?.full_name !== pr.base.repo.full_name) reasons.push("head branch is in a fork");
  if (pr.base.ref !== pr.base.repo.default_branch) reasons.push("base is not the default branch");
  if (!agent) reasons.push("head branch does not start with claude/ or gpt/");
  if (!labels.has(LABELS.optIn)) reasons.push(`label ${LABELS.optIn} is missing`);
  if (labels.has(LABELS.block)) reasons.push(`label ${LABELS.block} is set`);
  if (!labels.has(LABELS.claudeReview)) reasons.push(`label ${LABELS.claudeReview} is missing`);
  if (!labels.has(LABELS.gptReview)) reasons.push(`label ${LABELS.gptReview} is missing`);
  if (pr.mergeable !== true) reasons.push(`mergeable is ${pr.mergeable}`);
  if (!["clean", "has_hooks"].includes(pr.mergeable_state)) reasons.push(`mergeable_state is ${pr.mergeable_state}`);

  if (files.length === 0) reasons.push("no changed files were returned");
  if (files.length >= 3000) reasons.push("too many changed files to verify");
  if (agent === "gpt") {
    const outside = files.filter((f) => !GPT_ALLOWED.some((p) => p.test(f)));
    if (outside.length) reasons.push(`gpt branch changes paths outside its scope: ${outside.slice(0, 5).join(", ")}`);
  }

  const required = new Set(REQUIRED_WORKFLOWS.filter((r) => files.some((f) => r.pattern.test(f))).map((r) => r.workflow));
  for (const name of required) {
    const runs = workflowRuns.filter((r) => r.name === name && r.head_sha === pr.head.sha);
    const latest = runs.sort((a, b) => b.run_number - a.run_number)[0];
    if (!latest) reasons.push(`required workflow "${name}" has no run on the head SHA`);
    else if (latest.status !== "completed") reasons.push(`required workflow "${name}" is ${latest.status}`);
    else if (latest.conclusion !== "success") reasons.push(`required workflow "${name}" concluded ${latest.conclusion}`);
  }

  const evidence = checkRuns.filter((c) => !SELF_CHECKS.has(c.name));
  if (evidence.length === 0 && statuses.length === 0) reasons.push("no checks ran on the head SHA");
  for (const c of evidence) {
    if (c.status !== "completed") reasons.push(`check "${c.name}" is ${c.status}`);
    else if (!OK_CONCLUSIONS.has(c.conclusion)) reasons.push(`check "${c.name}" concluded ${c.conclusion}`);
  }
  for (const s of statuses) {
    if (s.state !== "success") reasons.push(`status "${s.context}" is ${s.state}`);
  }

  return { merge: reasons.length === 0, agent, reasons, required: [...required] };
}

// ---- GitHub API runner ----

async function api(path, init = {}) {
  const res = await fetch(`https://api.github.com${path}`, {
    ...init,
    headers: {
      authorization: `Bearer ${process.env.GITHUB_TOKEN}`,
      accept: "application/vnd.github+json",
      "x-github-api-version": "2022-11-28",
      ...(init.body ? { "content-type": "application/json" } : {})
    }
  });
  if (!res.ok && init.allowFail !== true) {
    throw new Error(`${init.method ?? "GET"} ${path} -> ${res.status} ${await res.text()}`);
  }
  return res.status === 204 ? null : res.json().catch(() => null);
}

async function paged(path, key) {
  const out = [];
  for (let page = 1; page <= 30; page++) {
    const sep = path.includes("?") ? "&" : "?";
    const data = await api(`${path}${sep}per_page=100&page=${page}`);
    const items = key ? data[key] : data;
    out.push(...items);
    if (items.length < 100) break;
  }
  return out;
}

async function getPr(repo, number) {
  // `mergeable` is computed lazily; poll until GitHub has a value.
  for (let i = 0; i < 6; i++) {
    const pr = await api(`/repos/${repo}/pulls/${number}`);
    if (pr.mergeable !== null) return pr;
    await new Promise((r) => setTimeout(r, 5000));
  }
  return api(`/repos/${repo}/pulls/${number}`);
}

async function run() {
  const repo = process.env.GITHUB_REPOSITORY;
  const dryRun = process.env.DRY_RUN === "true";
  const only = process.env.PR_NUMBER ? Number(process.env.PR_NUMBER) : null;
  const summary = [];

  const open = only
    ? [{ number: only }]
    : (await paged(`/repos/${repo}/pulls?state=open`)).filter((p) => p.labels.some((l) => l.name === LABELS.optIn));

  for (const { number } of open) {
    const pr = await getPr(repo, number);
    const sha = pr.head.sha;
    const [files, checkRuns, combined, workflowRuns] = await Promise.all([
      paged(`/repos/${repo}/pulls/${number}/files`).then((f) => f.map((x) => x.filename)),
      paged(`/repos/${repo}/commits/${sha}/check-runs`, "check_runs"),
      api(`/repos/${repo}/commits/${sha}/status`),
      paged(`/repos/${repo}/actions/runs?head_sha=${sha}`, "workflow_runs")
    ]);
    const decision = evaluate({ pr, files, checkRuns, statuses: combined.statuses ?? [], workflowRuns });

    if (!decision.merge) {
      summary.push(`- #${number} (${sha.slice(0, 7)}): not merged\n${decision.reasons.map((r) => `  - ${r}`).join("\n")}`);
      continue;
    }
    if (dryRun) {
      summary.push(`- #${number} (${sha.slice(0, 7)}): all gates pass (dry run, not merged)`);
      continue;
    }
    // `sha` makes GitHub refuse the merge if the head moved after the evaluation.
    await api(`/repos/${repo}/pulls/${number}/merge`, {
      method: "PUT",
      body: JSON.stringify({ sha, merge_method: "squash" })
    });
    summary.push(`- #${number} (${sha.slice(0, 7)}): merged (squash, expected head SHA)`);

    // A merge made with GITHUB_TOKEN does not start `push` workflows. Dispatch the
    // post-merge deploy workflows that the changed paths need. Cloudflare Pages is a
    // GitHub App and deploys from main by itself.
    const dispatch = [];
    if (files.some((f) => f.startsWith("agent-bridge/"))) dispatch.push("deploy-agent-bridge.yml");
    if (files.some((f) => f.startsWith("marketing-site/"))) dispatch.push("marketing-site-pages.yml");
    for (const wf of dispatch) {
      await api(`/repos/${repo}/actions/workflows/${wf}/dispatches`, {
        method: "POST",
        body: JSON.stringify({ ref: pr.base.ref })
      });
      summary.push(`  - dispatched ${wf}`);
    }
  }

  const text = `## Agent auto-merge\n\n${summary.length ? summary.join("\n") : "No open PR has the auto-merge label."}\n`;
  console.log(text);
  if (process.env.GITHUB_STEP_SUMMARY) {
    const fs = await import("node:fs");
    fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, text);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  run().catch((error) => {
    console.error(error.message);
    process.exit(1);
  });
}

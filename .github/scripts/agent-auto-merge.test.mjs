import { test } from "node:test";
import assert from "node:assert/strict";
import { evaluate, LABELS } from "./agent-auto-merge.mjs";
import { formatReviewComment, normalizeReview, parseReviewRecords } from "./agent-review-record.mjs";

const SHA = "a".repeat(40);
const repo = { full_name: "o/r", default_branch: "main" };
const allLabels = [LABELS.optIn];
const BOT = { login: "github-actions[bot]" };
const approve = (agent, sha = SHA, user = BOT, at = "2026-11-21T10:00:00Z") => ({
  user, created_at: at, body: formatReviewComment(agent, sha, normalizeReview({ verdict: "approve", summary: "ok", findings: [] }, { truncated: false }))
});
const reject = (agent, at = "2026-11-21T11:00:00Z") => ({
  user: BOT, created_at: at,
  body: formatReviewComment(agent, SHA, normalizeReview({ verdict: "approve", summary: "x", findings: [{ severity: "blocking", file: "a", title: "t", detail: "d" }] }, { truncated: false }))
});

function input(over = {}) {
  const pr = {
    state: "open", draft: false, mergeable: true, mergeable_state: "clean",
    head: { ref: "claude/fix", sha: SHA, repo }, base: { ref: "main", repo },
    labels: allLabels.map((name) => ({ name })),
    ...over.pr
  };
  return {
    pr,
    files: over.files ?? ["marketing-site/src/pages/index.astro"],
    checkRuns: over.checkRuns ?? [
      { name: "build", status: "completed", conclusion: "success" },
      { name: "Cloudflare Pages", status: "completed", conclusion: "success" },
      { name: "auto-merge-gate", status: "in_progress", conclusion: null }
    ],
    statuses: over.statuses ?? [],
    comments: over.comments ?? [approve("claude"), approve("gpt")],
    workflowRuns: over.workflowRuns ?? [
      { name: "Marketing site checks", head_sha: SHA, run_number: 2, status: "completed", conclusion: "success" },
      { name: "Marketing site checks", head_sha: SHA, run_number: 1, status: "completed", conclusion: "failure" }
    ]
  };
}

test("merges when every gate passes (latest run wins, own check ignored)", () => {
  const d = evaluate(input());
  assert.equal(d.merge, true, d.reasons.join("; "));
  assert.deepEqual(d.required, ["Marketing site checks"]);
});

test("refuses a failed check run", () => {
  const d = evaluate(input({ checkRuns: [{ name: "Workers Builds: gristora", status: "completed", conclusion: "failure" }] }));
  assert.equal(d.merge, false);
  assert.match(d.reasons.join(), /Workers Builds: gristora/);
});

test("refuses a pending check run", () => {
  assert.equal(evaluate(input({ checkRuns: [{ name: "build", status: "queued", conclusion: null }] })).merge, false);
});

test("refuses when the Lighthouse / browser QA workflow did not run on the head SHA", () => {
  const d = evaluate(input({ workflowRuns: [] }));
  assert.equal(d.merge, false);
  assert.match(d.reasons.join(), /Marketing site checks/);
});

test("refuses a failing commit status", () => {
  assert.equal(evaluate(input({ statuses: [{ context: "ci", state: "failure" }] })).merge, false);
});

test("refuses without checks at all", () => {
  assert.equal(evaluate(input({ checkRuns: [], workflowRuns: [], files: ["README.md"] })).merge, false);
});

for (const missing of allLabels) {
  test(`refuses without label ${missing}`, () => {
    const labels = allLabels.filter((l) => l !== missing).map((name) => ({ name }));
    assert.equal(evaluate(input({ pr: { labels } })).merge, false);
  });
}

test("refuses with do-not-merge", () => {
  const labels = [...allLabels, LABELS.block].map((name) => ({ name }));
  assert.equal(evaluate(input({ pr: { labels } })).merge, false);
});

test("refuses conflicts, stale or unstable branches", () => {
  for (const state of ["dirty", "behind", "unstable", "blocked", "unknown"]) {
    assert.equal(evaluate(input({ pr: { mergeable_state: state } })).merge, false, state);
  }
  assert.equal(evaluate(input({ pr: { mergeable: null } })).merge, false);
});

test("refuses forks, drafts and non-agent branches", () => {
  assert.equal(evaluate(input({ pr: { draft: true } })).merge, false);
  assert.equal(evaluate(input({ pr: { head: { ref: "feature/x", sha: SHA, repo } } })).merge, false);
  assert.equal(evaluate(input({ pr: { head: { ref: "claude/x", sha: SHA, repo: { full_name: "fork/r" } } } })).merge, false);
});

test("gpt branches may change only the marketing website", () => {
  const head = { ref: "gpt/copy", sha: SHA, repo };
  assert.equal(evaluate(input({ pr: { head } })).merge, true);
  const d = evaluate(input({ pr: { head }, files: ["marketing-site/a.astro", "sunucu/src/lib.rs"] }));
  assert.equal(d.merge, false);
  assert.match(d.reasons.join(), /outside its scope/);
});

test("product paths require their own workflows", () => {
  const d = evaluate(input({ files: ["sunucu/src/lib.rs"] }));
  assert.equal(d.merge, false);
  assert.deepEqual(d.required.sort(), ["altyapi", "sozlesme", "sunucu"]);
});

test("refuses without a gpt review, or with a review of an older SHA", () => {
  assert.match(evaluate(input({ comments: [approve("claude")] })).reasons.join(), /no gpt review/);
  assert.equal(evaluate(input({ comments: [approve("claude"), approve("gpt", "b".repeat(40))] })).merge, false);
});

test("refuses without a claude review", () => {
  assert.equal(evaluate(input({ comments: [approve("gpt")] })).merge, false);
});

test("ignores review records that a person or another token wrote", () => {
  const forged = approve("gpt", SHA, { login: "oguzbay-del" });
  assert.equal(evaluate(input({ comments: [approve("claude"), forged] })).merge, false);
});

test("the latest review for the head SHA wins", () => {
  assert.equal(evaluate(input({ comments: [approve("claude"), approve("gpt"), reject("gpt")] })).merge, false);
  assert.equal(evaluate(input({ comments: [approve("claude"), reject("gpt"), approve("gpt", SHA, BOT, "2026-11-21T12:00:00Z")] })).merge, true);
});

test("a blocking finding turns an approve verdict into request_changes", () => {
  const r = normalizeReview({ verdict: "approve", summary: "", findings: [{ severity: "major", file: "f", title: "t", detail: "d" }] }, { truncated: false });
  assert.equal(r.verdict, "request_changes");
});

test("a truncated diff or an invalid result is never approved", () => {
  assert.equal(normalizeReview({ verdict: "approve", summary: "", findings: [] }, { truncated: true }).verdict, "request_changes");
  assert.equal(normalizeReview(null, { truncated: false }).verdict, "request_changes");
  assert.equal(normalizeReview({ verdict: "maybe", findings: [] }, { truncated: false }).verdict, "request_changes");
});

test("review records round-trip through a comment", () => {
  const [r] = parseReviewRecords([approve("gpt")]);
  assert.equal(r.agent, "gpt");
  assert.equal(r.sha, SHA);
  assert.equal(r.verdict, "approve");
});

test("changes under .github/ never merge automatically", () => {
  const d = evaluate(input({ files: ["marketing-site/a.astro", ".github/workflows/x.yml"] }));
  assert.equal(d.merge, false);
  assert.match(d.reasons.join(), /manual merge/);
});

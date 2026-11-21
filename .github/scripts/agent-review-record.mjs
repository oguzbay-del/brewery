// Machine-readable review records. No dependencies: the auto-merge gate and the
// runners both import this file. A record is an HTML comment in a PR comment that
// the review workflow posts with GITHUB_TOKEN (author github-actions[bot]).

export const REVIEW_MARKER = "agent-review:";
export const MAX_DIFF_CHARS = 150_000;
export const TRUSTED_REVIEW_AUTHOR = "github-actions[bot]";

/** Enforce the policy on a model result: no approval with blocking/major findings or a truncated diff. */
export function normalizeReview(raw, { truncated }) {
  const findings = Array.isArray(raw?.findings) ? raw.findings : [];
  const serious = findings.filter((f) => f.severity === "blocking" || f.severity === "major").length;
  let verdict = raw?.verdict === "approve" && serious === 0 ? "approve" : "request_changes";
  let summary = String(raw?.summary ?? "");
  if (truncated) {
    verdict = "request_changes";
    summary = `The diff is larger than ${MAX_DIFF_CHARS} characters, so an automated approval is not possible. ${summary}`;
  }
  if (!raw || !["approve", "request_changes"].includes(raw.verdict)) {
    verdict = "request_changes";
    summary = `The reviewer returned no valid result. ${summary}`;
  }
  return { verdict, summary, findings, serious };
}

export function formatReviewComment(agent, sha, review) {
  const title = agent === "gpt" ? "GPT" : "Claude";
  const icon = review.verdict === "approve" ? "✅ approve" : "❌ changes requested";
  const rows = review.findings.map(
    (f) => `| ${f.severity} | \`${String(f.file).replace(/\|/g, "\\|")}\` | ${String(f.title).replace(/\|/g, "\\|")} |`
  );
  const record = { agent, sha, verdict: review.verdict, serious: review.serious, findings: review.findings.length, v: 1 };
  return [
    `### ${title} independent review: ${icon}`,
    "",
    `Reviewed head: \`${sha}\``,
    "",
    review.summary,
    "",
    rows.length ? ["| Severity | File | Finding |", "|---|---|---|", ...rows].join("\n") : "No findings.",
    "",
    review.findings.length
      ? "<details><summary>Details</summary>\n\n" +
        review.findings.map((f) => `- **${f.title}** (${f.severity}, \`${f.file}\`): ${f.detail}`).join("\n") +
        "\n</details>"
      : "",
    "",
    `<!-- ${REVIEW_MARKER}${JSON.stringify(record)} -->`
  ].join("\n");
}

/** Parse review records from PR comments. Only comments by `trustedAuthor` count. */
export function parseReviewRecords(comments, trustedAuthor = TRUSTED_REVIEW_AUTHOR) {
  const records = [];
  for (const c of comments) {
    if (c.user?.login !== trustedAuthor) continue;
    const match = new RegExp(`<!-- ${REVIEW_MARKER}(\\{.*?\\}) -->`).exec(c.body ?? "");
    if (!match) continue;
    try {
      const r = JSON.parse(match[1]);
      if ((r.agent === "claude" || r.agent === "gpt") && typeof r.sha === "string") {
        records.push({ ...r, created_at: c.created_at, id: c.id });
      }
    } catch {}
  }
  return records;
}

/** The latest record per agent for `sha`. */
export function reviewsForSha(records, sha) {
  const out = {};
  for (const r of records.filter((x) => x.sha === sha).sort((a, b) => a.created_at.localeCompare(b.created_at))) {
    out[r.agent] = r;
  }
  return out;
}

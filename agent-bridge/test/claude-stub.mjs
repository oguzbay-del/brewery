#!/usr/bin/env node
// Stub for the Claude Code CLI in the offline runner test. It reads the prompt from stdin.
import { readFileSync, writeFileSync } from "node:fs";
const args = process.argv.slice(2);
const prompt = readFileSync(0, "utf8");
const leaked = ["AGENT_GITHUB_TOKEN", "OPENAI_API_KEY", "MCP_TOKEN_GPT", "GITHUB_TOKEN"].filter((k) => process.env[k]);
let structured;
if (args.includes("--tools")) {
  structured = { verdict: "approve", summary: `stub review; leaked=${leaked.join(",") || "none"}`, findings: [] };
} else {
  const m = /Task ([0-9a-f-]{36})/.exec(prompt);
  writeFileSync("agent-output.txt", `done ${m?.[1]}\n`);
  structured = {
    status: "completed",
    result: `stub implementation; leaked=${leaked.join(",") || "none"}; mcp=${args.includes("--mcp-config")}`,
    message_to_other_agent: "Implementation ready.",
    follow_up_tasks: [{ title: "Review stub change", description: "Review the stub implementation for regressions." }]
  };
}
process.stdout.write(JSON.stringify({ type: "result", is_error: false, result: "", structured_output: structured }));

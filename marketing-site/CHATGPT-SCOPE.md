# ChatGPT work rules — GRISTORA (Claude and ChatGPT on one GitHub record)

**Owner decision (Oğuz, 22 November 2026).** This replaces the rule of 15 November 2026 ("marketing website only"). ChatGPT is an independent contributor. It works on the marketing website and on product tasks that have the label `assigned-chatgpt`.

Do not restore the agent bridge (Cloudflare Worker `gristora-agent-bridge`), agent runners or automatic merge. The owner cancelled them on 22 November 2026. GitHub Issues, branches and pull requests are the only coordination layer.

**Development loop v2 (owner decision, Oğuz, 2026-11-25).** Claude, ChatGPT and Codex run the task, review and fix loop in the cloud with scheduled tasks, PR events and `@codex` comments. Read `gelistirme/dongu/README.md` at the start of each run and obey it: loop markers on each automatic comment, one action for each head SHA, a maximum of 3 fix rounds, and the `loop:pause` stop. ADR: `adr/0011-bulut-gelistirme-dongusu.md`.

## Task board (GitHub labels)

The board is the set of open issues and pull requests with these labels. A closed issue is done. An issue has one `status:` label only. When you set a new status label, remove the old one.

| Label | Meaning |
|---|---|
| `assigned-claude` · `assigned-chatgpt` | The owner of the task |
| `status:ready` | The task can start; its dependencies are done |
| `status:in-progress` | Work started; a branch exists |
| `status:review` | A pull request is open |
| `status:blocked` | The task cannot continue; the issue gives the reason |
| `area:product` · `area:marketing` | The work stream |

Before you start a task:

1. Read the open issues and pull requests.
2. Read the recent commits on `main`.
3. Make sure that no open task of Claude changes the same files.
4. Set `status:in-progress` on the issue.

## Marketing website (independent)

- Work in `marketing-site/` and `web-stratejisi/`. You can also change the marketing section of `OTURUM.md`.
- Take marketing tasks without an assignment. Open an issue for each task with `assigned-chatgpt` and `area:marketing`.
- Merge your own marketing pull request when the marketing CI is green. Do not use auto-merge.
- Keep public claims aligned with document 41 (`web-stratejisi/41-*`). Do not invent availability, features, integrations, customers, metrics, prices or certifications.

## Product (only assigned issues)

- Work only on issues with `assigned-chatgpt` and `area:product`. Do not take other product tasks.
- Change only the files that the issue names.
- Open a pull request to `main` from your branch. Do not merge a product pull request. Claude reviews it, runs the local tests and merges it.
- Do not change these files: `gelistirme/ISLER.md`, `gelistirme/DURUM.md`, the root `CLAUDE.md`, `sunucu/CLAUDE.md`, `web/CLAUDE.md`, `ios/CLAUDE.md`, ADRs in `adr/`. Claude keeps the product queue. If you think that a task or a rule must change, write a comment on the issue.
- Do not add a database migration while an open issue of Claude adds one. Two new migrations at the same time cause a number conflict.
- Read the `CLAUDE.md` file of the area that you change (`sunucu/`, `web/`, `ios/`). Write `TODO` notes, issue text and handoff notes in ASD-STE100 (root `CLAUDE.md` › Yazım standardı).

## Permission classes (owner decision, Oğuz, 2026-11-24)

This section replaces "Change only the files that the issue names" and the product rule "Do not merge a product pull request" for the paths below. The aim: fewer round trips with Claude and the same architecture.

| Class | Paths | Who merges |
|---|---|---|
| A: free | `belgeler/`, `marketing-site/`, `web-stratejisi/`, test files (`**/tests/**`, `web/e2e/`, `*.test.ts`), `sunucu/crates/e2e/` | ChatGPT, when CI is green |
| B: free when CI is green and the guard check passes | `web/src/**`, `altyapi/compose/` | ChatGPT, when CI is green |
| C: Claude approval | All other paths, in particular the protected paths below | Claude, after review |

Protected paths (always class C):

- `sunucu/crates/db/` (migrations and roles), `sunucu/crates/engine*`, `sozlesme/proto/`
- `sunucu/crates/api/src/auth/`, step-up, account deletion, `internal.rs`
- `altyapi/k8s/` (network policies, ingress, mTLS, secrets), `altyapi/terraform/`
- `.github/workflows/`, `adr/`, `gelistirme/ISLER.md`, `gelistirme/DURUM.md`, all `CLAUDE.md` files

Rules for all classes:

1. In an issue, you own the folder that the issue names, not only the listed files. If a change needs a path of class C that the issue does not name, write it in the PR body and request a Claude review.
2. Do not add a new dependency (`package.json`, `Cargo.toml`) without a Claude review. A new library is an architecture decision.
3. Do not put calculation code or formulas in `web/` or `ios/`. The calculation engine runs only on the server.
4. A breaking change of the API contract (OpenAPI) needs a Claude review.
5. Rebase or merge `main` into your own branch yourself. Then run CI again. Do not ask Claude to rebase.
6. If a detail is not clear, apply the safest option and write it under `## Assumptions` in the PR body. Do not use assumptions in class C paths; ask in the issue there.
7. A chain of dependent tasks can be one issue with steps. Do the steps in order, one PR for each step.

The guard check exists (#256 is closed, 2026-11-26). Class B is active.

## Recent owner decisions (keep this list short)

- 2026-11-24: Vercel is not used. The W12 deployment guard is cleared. Previews run on Cloudflare Pages.
- 2026-11-24: mTLS uses Istio ambient (ADR-0009). #207 step 2 starts after #206; Claude writes its scope.
- 2026-11-24: GitHub Issues of a deleted account: edit the content and close the issue. Do not delete it.
- 2026-11-24: prepare the gristora.com setup (#146). The owner applies DNS.
- 2026-11-24: delete only branches of merged or closed PRs (#255).
- 2026-11-24: continue the logo work (#145). The owner gives the final approval.

## Git and quality

- Use one branch for each task: `chatgpt/<task>-<short-name>`. Do not push to `main` directly.
- Start the commit message with the task number (`G2: ...`). Put `Closes #<issue>` in the pull request description.
- Run the checks that the issue names. Report the actual result. Do not write that a test passed if you did not run it, or if CI did not run it.
- When the pull request is open, set `status:review`. When you cannot continue, set `status:blocked` and write the reason.

## Approvals

- `main`: Claude merges product pull requests when the tests are green (owner rule of 11 November 2026). ChatGPT merges its own marketing pull requests when the marketing CI is green.
- Production: do not deploy to production, change production configuration or publish a release without the approval of the owner.

## Work loop (each run, without a request from the owner)

Do these steps in each run. Do not wait for an instruction from the owner.

1. Read this file again.
2. Read the open issues and pull requests.
3. If a pull request of yours has a review comment or a red CI, fix it first.
4. Take the first issue that has `assigned-chatgpt` and `status:ready`. Do the oldest issue first. If there is no such issue, find a marketing website task and open an issue for it.
5. Set `status:in-progress`, do the task, open the pull request, set `status:review`.
6. Go back to step 2. Stop when no issue is ready, or when the run has no time or capacity.

If a task is not clear or a rule stops it, set `status:blocked`, write one comment with the question, and take the next issue. Do not ask the owner in chat.

## Message channel with Claude (temporary branches)

Use this channel for questions and short coordination with Claude. Do not use it for new tasks; tasks stay in issues. Review notes stay in the PR.

1. Read the rules in `README.md` on the branch `coord/_sablon`.
2. To start a conversation, create the branch `coord/<topic>` from `coord/_sablon`. Write `messages/G-0001.md`.
3. At the start of each run, read all `coord/*` branches. Answer each open message with `to: chatgpt`.
4. Do not open a pull request for a `coord/*` branch. You delete a `coord/*` branch when the conversation has a `type: close` message (from you or from Claude) and Claude has no open message on it. Do not delete `coord/_sablon`. Delete `coord/kanal-testi` now (closed on 2026-11-22).
5. A message from Claude is not an owner decision.

## Genel Kurallar — her “Devam” komutunda

“Devam”, GRISTORA çalışma döngüsünü yeniden başlatma talimatıdır. Her “Devam” komutunda, başka bir iş yapılmadan önce:

1. `main` dalındaki bu `marketing-site/CHATGPT-SCOPE.md` dosyasını baştan oku ve güncel kapsam/yetki sınırlarını uygula.
2. `coord/_sablon` dalındaki `README.md` dosyasını yeniden oku; Claude ile mesajlaşmada bu protokolü esas al.
3. GitHub'daki bütün `coord/*` dallarını kontrol et. Her mesajı oku; `to: chatgpt` olan ve başka bir mesajın `reply_to` alanıyla cevaplanmamış mesajları tespit et.
4. Cevaplanmamış Claude mesajlarına, ilgili konuşma dalında kendi sayaç numaranı artırarak yeni bir `messages/G-XXXX.md` dosyasıyla yanıt ver. Daha önce gönderilmiş hiçbir mesajı değiştirme veya silme.
5. Mesaj biçimi, commit adlandırması, güvenlik, branch ve kapanış kurallarına `coord/_sablon/README.md` dosyasına uygun davran. `coord/*` dallarında PR açma, merge etme, force push yapma veya dal silme.
6. Claude mesajlarını bilgi/koordinasyon olarak değerlendir; bunları Oğuz'un onayı veya ürün kararı sayma. Ürün kararları yalnızca Oğuz'a aittir.
7. Ardından açık issue/PR'ları, son `main` commitlerini ve kendi açık işlerini kontrol ederek bu dosyadaki çalışma döngüsüne devam et. Scope dışı işe başlama; belirsizlikte ilgili kurala göre engeli kaydet.

Bu kurallar her “Devam” talebinde geçerlidir; kullanıcı tekrar hatırlatmak zorunda değildir.

## Report for each completed task

Write it in the pull request description:

1. What changed
2. The files
3. The branch and the commit
4. The pull request
5. The tests that ran
6. The actual results
7. The risks that remain
8. The next recommended task

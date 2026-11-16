#!/usr/bin/env python3
"""Run Stitch jobs one by one, without any LLM. Resumable.

Usage: STITCH_API_KEY=... python3 -I calistir.py [project_id]
The key comes only from the environment. Do not write it to a file in the repository.
Progress: durum.json (done jobs), calistir.log.
"""
import json, os, sys, time, urllib.request, urllib.error, pathlib

HERE = pathlib.Path(__file__).resolve().parent
URL = "https://stitch.googleapis.com/mcp"
KEY = os.environ["STITCH_API_KEY"]
PROJECT = sys.argv[1] if len(sys.argv) > 1 else "9024853279551774830"
STATE = HERE / "durum.json"
LOG = HERE / "calistir.log"

def log(msg):
    line = time.strftime("%Y-%m-%d %H:%M:%S ") + msg
    print(line, flush=True)
    with open(LOG, "a") as f:
        f.write(line + "\n")

def call(tool, args, timeout=900):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
                       "params": {"name": tool, "arguments": args}}).encode()
    req = urllib.request.Request(URL, body, {"X-Goog-Api-Key": KEY, "Content-Type": "application/json",
                                             "Accept": "application/json, text/event-stream"})
    raw = urllib.request.urlopen(req, timeout=timeout).read().decode()
    if raw.lstrip().startswith("data:") or "\ndata:" in raw:
        raw = [l[5:] for l in raw.splitlines() if l.startswith("data:")][-1]
    d = json.loads(raw)
    if "error" in d:
        raise RuntimeError(json.dumps(d["error"])[:300])
    r = d["result"]
    text = " ".join(c.get("text", "") for c in r.get("content", []))
    if r.get("isError"):
        raise RuntimeError(text[:300])
    return text

def main():
    jobs = json.loads((HERE / "jobs.json").read_text())
    done = json.loads(STATE.read_text()) if STATE.exists() else {}
    ds = json.loads(call("list_design_systems", {"projectId": PROJECT}, 120))
    ds_id = ds["designSystems"][0]["name"] if ds.get("designSystems") else None
    log(f"start: {len(jobs)} jobs, {len(done)} done, design system {ds_id}")
    for j in jobs:
        if j["key"] in done:
            continue
        args = dict(j["args"], projectId=PROJECT)
        if j["tool"] == "generate_screen_from_text" and ds_id:
            args["designSystem"] = ds_id
        for attempt in (1, 2):
            t0 = time.time()
            try:
                out = call(j["tool"], args)
                done[j["key"]] = {"ok": True, "s": int(time.time() - t0), "out": out[:400]}
                log(f"OK   {j['key']} ({int(time.time()-t0)} s)")
                break
            except (TimeoutError, urllib.error.URLError, ConnectionError) as e:
                # Stitch can finish the job after a timeout. Do not repeat it: that makes duplicates.
                done[j["key"]] = {"ok": None, "err": f"timeout/connection: {e}"[:300]}
                log(f"WAIT {j['key']}: {e} (may still finish in Stitch)")
                break
            except Exception as e:
                log(f"ERR  {j['key']} try {attempt}: {e}")
                if attempt == 2:
                    done[j["key"]] = {"ok": False, "err": str(e)[:300]}
                time.sleep(120)
        STATE.write_text(json.dumps(done, indent=1))
        time.sleep(30)
    ok = sum(1 for v in done.values() if v.get("ok"))
    log(f"end: {ok}/{len(jobs)} ok")

if __name__ == "__main__":
    main()

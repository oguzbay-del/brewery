#!/usr/bin/env bash
# Restart calistir.py when it stops before all jobs are done.
# Usage: STITCH_API_KEY=... ./watchdog.sh
# Stops when every job in jobs.json has "ok": true in durum.json, or after 5 restarts in a row with no progress.
set -u
cd "$(dirname "$0")"
: "${STITCH_API_KEY:?set STITCH_API_KEY}"
left() { python3 -I -c '
import json,os
j=[x["key"] for x in json.load(open("jobs.json"))]
d=json.load(open("durum.json")) if os.path.exists("durum.json") else {}
print(sum(1 for k in j if not d.get(k,{}).get("ok")))'; }
stuck=0
while :; do
  n=$(left)
  [ "$n" -eq 0 ] && { echo "$(date -u +%F' '%T) watchdog: all jobs done" | tee -a calistir.log; exit 0; }
  if ! pgrep -f "python3 -I calistir.py" >/dev/null; then
    echo "$(date -u +%F' '%T) watchdog: start calistir.py ($n jobs left)" | tee -a calistir.log
    before=$n
    python3 -I calistir.py >> nohup.out 2>&1
    after=$(left)
    if [ "$after" -ge "$before" ]; then stuck=$((stuck+1)); else stuck=0; fi
    [ "$stuck" -ge 5 ] && { echo "$(date -u +%F' '%T) watchdog: no progress after 5 runs, stop" | tee -a calistir.log; exit 1; }
    sleep 300
  else
    sleep 60
  fi
done

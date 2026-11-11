#!/usr/bin/env bash
# kind kümesi duman testi. Hata varsa 1 döner. Kullanım: kökte `make kind-test` (önce `make kind-up`).
# Calico (politikayı uygulayan CNI) yoksa ya da NETPOL=0 verilirse ağ politikası denetimi atlanır.
set -u
NS=demleme
K="kubectl --context kind-demleme -n $NS"
fail=0
ok()  { printf '  ok    %s\n' "$1"; }
bad() { printf '  HATA  %s\n' "$1"; fail=1; }

echo "Pod durumu"
if $K wait --for=condition=Ready pod --all --timeout=120s >/dev/null 2>&1; then
  ok "bütün podlar Ready ($($K get pods --no-headers | wc -l) pod)"
else
  bad "Ready olmayan pod var"; $K get pods
fi

echo "HTTP /healthz (port-forward)"
check_http() { # servis port yerel-port
  $K port-forward "svc/$1" "$3:$2" >/dev/null 2>&1 &
  local pf=$! code=000 n
  for n in 1 2 3 4 5 6 7 8 9 10; do
    code=$(curl -s -o /dev/null -m 3 -w '%{http_code}' "http://127.0.0.1:$3/healthz")
    [ "$code" = 200 ] && break; sleep 1
  done
  kill "$pf" 2>/dev/null; wait "$pf" 2>/dev/null
  if [ "$code" = 200 ]; then ok "$1 /healthz 200"; else bad "$1 /healthz ($code)"; fi
}
check_http api 8080 18080
check_http worker 8081 18081
check_http ingest 8082 18082

echo "engine-service (gRPC)"
if out=$($K exec deploy/engine-service -- grpc_health_probe -addr=127.0.0.1:50051 2>&1); then
  ok "gRPC sağlık: $out"
else
  bad "gRPC sağlık: $out"
fi

echo "Ağ politikası (engine-service:50051'e TCP bağlantısı)"
# Gerçek pod'un içinden bash /dev/tcp ile denenir (imajda bash var, ek araç gerekmez).
# Çıkış kodu 124 = zaman aşımı (paket düşürüldü); 0 = bağlandı; başka kod = deneme geçersiz.
tcp() { $K exec "deploy/$1" -- timeout 4 bash -c 'exec 3<>/dev/tcp/engine-service/50051' >/dev/null 2>&1; }
tcp api; rc=$?
if [ $rc = 0 ]; then ok "api -> engine-service izinli"; else bad "api -> engine-service ENGELLENDİ (çıkış $rc; izin verilmeliydi)"; fi
if [ "${NETPOL:-1}" = 0 ] || ! kubectl -n kube-system get ds calico-node >/dev/null 2>&1; then
  echo "  UYARI: politikayı uygulayan CNI yok (calico-node bulunamadı) ya da NETPOL=0; engelleme denetimi atlandı"
else
  for src in worker ingest; do
    tcp $src; rc=$?
    case $rc in
      124) ok "$src -> engine-service reddedildi (zaman aşımı)" ;;
      0)   bad "$src -> engine-service ERİŞTİ (reddedilmeliydi)" ;;
      *)   bad "$src -> engine-service denemesi geçersiz (çıkış $rc)" ;;
    esac
  done
fi

[ "$fail" = 0 ] && echo "TAMAM" || echo "BAŞARISIZ"
exit "$fail"

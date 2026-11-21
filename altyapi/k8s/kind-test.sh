#!/usr/bin/env bash
# kind kümesi duman testi. Hata varsa 1 döner. Kullanım: kökte `make kind-test` (önce `make kind-up`).
# Calico (politikayı uygulayan CNI) yoksa ya da NETPOL=0 verilirse ağ politikası denetimi atlanır.
# Istio ambient (ztunnel) yoksa ya da MESH=0 verilirse mTLS ve genel giriş denetimleri atlanır.
set -u
NS=demleme
K="kubectl --context kind-demleme -n $NS"
CURL_IMAGE=curlimages/curl:8.11.1
fail=0
ok()  { printf '  ok    %s\n' "$1"; }
bad() { printf '  HATA  %s\n' "$1"; fail=1; }

echo "Pod durumu"
# Göç Job'ının podu biter (Succeeded) ve Ready olmaz; ayrıca Job'ın tamamlandığı denetlenir.
SVC_PODS='app.kubernetes.io/name!=migrate'
if $K wait --for=condition=Ready pod -l "$SVC_PODS" --timeout=120s >/dev/null 2>&1; then
  ok "bütün servis podları Ready ($($K get pods -l "$SVC_PODS" --no-headers | wc -l) pod)"
else
  bad "Ready olmayan pod var"; $K get pods
fi
if $K wait --for=condition=complete job/migrate --timeout=60s >/dev/null 2>&1; then
  ok "göç Job'ı tamamlandı"
else
  bad "göç Job'ı tamamlanmadı"; $K logs job/migrate --tail=20
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

# Gerçek pod'un içinden bash /dev/tcp ile TCP denemesi (imajda bash var, ek araç gerekmez).
# Bağlantı kurulduktan sonra 3 sn okunur: Istio ambient'te yerel ztunnel bağlantıyı önce kabul
# eder, reddedilen bağlantıyı sonradan kapatır; yalnız "bağlandı" demek bu yüzden yetmez.
# Çıkış kodu: 0 = açık (veri geldi ya da sessiz), 5 = karşı taraf kapattı, 3 = bağlanamadı,
# 124 = zaman aşımı (paket düşürüldü). 3, 5 ve 124 "reddedildi" sayılır.
tcp() { # dağıtım ana-bilgisayar port
  $K exec "deploy/$1" -- timeout 8 bash -c \
    "{ exec 3<>/dev/tcp/$2/$3; } 2>/dev/null || exit 3; read -t 3 -n 1 -u 3 _; [ \$? = 1 ] && exit 5; exit 0" \
    >/dev/null 2>&1
}
denied() { case $1 in 3|5|124) return 0 ;; *) return 1 ;; esac; }

echo "Ağ politikası ve kimlik (engine-service:50051'e TCP bağlantısı)"
tcp api engine-service 50051; rc=$?
if [ $rc = 0 ]; then ok "api -> engine-service izinli"; else bad "api -> engine-service ENGELLENDİ (çıkış $rc; izin verilmeliydi)"; fi
MESH_ON=0
kubectl -n istio-system get ds ztunnel >/dev/null 2>&1 && MESH_ON=1
if [ "${NETPOL:-1}" = 0 ] || { [ "$MESH_ON" = 0 ] && ! kubectl -n kube-system get ds calico-node >/dev/null 2>&1; }; then
  echo "  UYARI: politikayı uygulayan CNI ve ağ örgüsü yok ya da NETPOL=0; engelleme denetimi atlandı"
else
  for src in worker ingest; do
    tcp $src engine-service 50051; rc=$?
    if denied $rc; then ok "$src -> engine-service reddedildi (çıkış $rc)"; else bad "$src -> engine-service ERİŞTİ (reddedilmeliydi; çıkış $rc)"; fi
  done
fi

echo "Veri servisleri ve sahte servisler (api'den TCP)"
for t in postgres:5432 redis:6379 nats:4222 objectstore:8333 mailpit:1025 github-mock:8080 apns-mock:8080 fake-oidc:9000; do
  tcp api "${t%%:*}" "${t##*:}"; rc=$?
  if [ $rc = 0 ]; then ok "api -> $t"; else bad "api -> $t ulaşılamadı (çıkış $rc)"; fi
done
# Nesne deposu yalnız api ve worker'a açık (ağ politikası).
if [ "${NETPOL:-1}" != 0 ] && kubectl -n kube-system get ds calico-node >/dev/null 2>&1; then
  tcp ingest objectstore 8333; rc=$?
  if denied $rc; then ok "ingest -> objectstore reddedildi (çıkış $rc)"; else bad "ingest -> objectstore ERİŞTİ (reddedilmeliydi; çıkış $rc)"; fi
fi

# --- Genel giriş, destek paneli ve mTLS (Istio ambient) ---
# Her deneme tek kullanımlık bir curl podudur. Pod; hizmet hesabı, etiketler ve ağ örgüsü üyeliği
# istenen biçimde kurulur. Çıktı: HTTP kodu (bağlanamazsa 000); çıkış kodu: curl'ün kodu.
n_pod=0
curlpod() { # hizmet-hesabı etiketler örgü(ambient|none) url
  local sa=$1 labels=$2 mesh=$3 url=$4 name
  n_pod=$((n_pod + 1)); name="t-$$-$n_pod"
  $K run "$name" --image="$CURL_IMAGE" --image-pull-policy=IfNotPresent --restart=Never --rm -i --quiet \
    --labels="$labels,istio.io/dataplane-mode=$mesh" --pod-running-timeout=90s \
    --overrides="{\"spec\":{\"serviceAccountName\":\"$sa\",\"automountServiceAccountToken\":false,\"securityContext\":{\"runAsNonRoot\":true,\"runAsUser\":100,\"seccompProfile\":{\"type\":\"RuntimeDefault\"}},\"containers\":[{\"name\":\"$name\",\"image\":\"$CURL_IMAGE\",\"securityContext\":{\"allowPrivilegeEscalation\":false,\"capabilities\":{\"drop\":[\"ALL\"]}},\"command\":[\"curl\",\"-s\",\"-o\",\"/dev/null\",\"-m\",\"6\",\"-w\",\"%{http_code}\",\"$url\"]}]}}" \
    2>/dev/null
}
expect_code() { # açıklama beklenen-desen sonuç
  if [[ "$3" =~ ^($2)$ ]]; then ok "$1 ($3)"; else bad "$1: beklenen $2, gelen $3"; fi
}

echo "Genel giriş ve destek paneli"
if [ "${MESH:-1}" = 0 ]; then
  echo "  UYARI: MESH=0; genel giriş ve mTLS denetimleri atlandı"
else
  [ "$MESH_ON" = 1 ] || bad "Istio ambient (ztunnel) kurulu değil; mTLS denetlenemez"
  $K create serviceaccount intruder >/dev/null 2>&1
  trap '$K delete serviceaccount intruder --ignore-not-found >/dev/null 2>&1' EXIT

  code=$(curlpod default "app.kubernetes.io/name=probe" ambient "http://edge:8080/healthz")
  expect_code "genel giriş -> api /healthz" 200 "$code"
  code=$(curlpod default "app.kubernetes.io/name=probe" ambient "http://edge:8080/internal/v1/workspaces")
  expect_code "genel girişte /internal/v1/workspaces" 404 "$code"
  code=$(curlpod default "app.kubernetes.io/name=probe" ambient "http://edge:8080/internal/")
  expect_code "genel girişte /internal/" 404 "$code"
  code=$(curlpod default "app.kubernetes.io/name=probe" ambient "http://oauth2-proxy:4180/ping")
  expect_code "oauth2-proxy /ping" 200 "$code"
  code=$(curlpod default "app.kubernetes.io/name=probe" ambient "http://oauth2-proxy:4180/internal/v1/workspaces")
  expect_code "oauth2-proxy oturumsuz /internal isteğini içeri almaz" '302|401|403' "$code"

  echo "mTLS (Istio ambient, PeerAuthentication STRICT)"
  # Kimlik: api'ye yalnız edge ve oauth2-proxy hizmet hesapları erişir (AuthorizationPolicy).
  code=$(curlpod edge "app.kubernetes.io/name=edge" ambient "http://api:8080/healthz")
  expect_code "edge kimliği -> api (mTLS)" 200 "$code"
  code=$(curlpod intruder "app.kubernetes.io/name=edge" ambient "http://api:8080/healthz"); rc=$?
  if [ "$code" = 000 ] && [ "$rc" != 6 ]; then ok "yanlış kimlik -> api reddedildi (curl çıkış $rc)"; else bad "yanlış kimlik -> api: kod $code, çıkış $rc (reddedilmeliydi)"; fi
  # Düz metin: ağ örgüsü dışındaki pod (etiketi ağ politikasını geçer) STRICT'e takılır.
  code=$(curlpod default "app.kubernetes.io/name=edge" none "http://api:8080/healthz"); rc=$?
  if [ "$code" = 000 ] && [ "$rc" != 6 ]; then ok "düz metin istek api'de reddedildi (curl çıkış $rc)"; else bad "düz metin istek: kod $code, çıkış $rc (reddedilmeliydi)"; fi
  code=$(curlpod default "app.kubernetes.io/name=api" none "http://engine-service:50051/"); rc=$?
  if [ "$code" = 000 ] && [ "$rc" != 6 ]; then ok "düz metin istek engine-service'te reddedildi (curl çıkış $rc)"; else bad "düz metin istek engine-service: kod $code, çıkış $rc (reddedilmeliydi)"; fi
fi

[ "$fail" = 0 ] && echo "TAMAM" || echo "BAŞARISIZ"
exit "$fail"

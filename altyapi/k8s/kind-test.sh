#!/usr/bin/env bash
# kind kümesi duman testi. Hata varsa 1 döner. Kullanım: kökte `make kind-test` (önce `make kind-up`).
# Calico (politikayı uygulayan CNI) yoksa ya da NETPOL=0 verilirse ağ politikası denetimi atlanır.
# mTLS denetimleri (Istio ambient) her zaman çalışır.
set -u
NS=demleme
K="kubectl --context kind-demleme -n $NS"
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

echo "Kenar girişi: /internal herkese açık girişten sunulmaz (Gateway API / Istio, port-forward)"
# Herkese açık konak adında /internal/... 404 verir. Destek paneli yalnız dahili konak adından,
# oauth2-proxy üzerinden gelir; yerelde oauth2-proxy sabit sahte e-posta ve gerçek jeton ekler:
# jeton kabul edilir, e-posta personel olmadığından API 403 verir (401 = başlıklar ulaşmadı).
$K port-forward svc/kenar-istio 18088:80 >/dev/null 2>&1 &
pf=$!
edge_code() { # konak-adı yol
  curl -s -o /dev/null -m 5 -H "Host: $1" -w '%{http_code}' "http://127.0.0.1:18088$2"
}
for n in 1 2 3 4 5 6 7 8 9 10; do
  [ "$(edge_code demleme.local /v1/me/session)" != 000 ] && break; sleep 1
done
code=$(edge_code demleme.local /v1/me/session)
if [ "$code" = 401 ]; then ok "kontrol: herkese açık giriş /v1/me/session api'ye ulaşır (401)"; else bad "kontrol: herkese açık giriş /v1/me/session ($code; 401 olmalıydı)"; fi
code=$(edge_code demleme.local /internal/v1/workspaces)
if [ "$code" = 404 ]; then ok "herkese açık giriş /internal/v1/... 404"; else bad "herkese açık giriş /internal/v1/... ($code; 404 olmalıydı)"; fi
code=$(edge_code demleme.local /internal/v1/workspaces/00000000-0000-0000-0000-000000000000/summary)
if [ "$code" = 404 ]; then ok "herkese açık giriş /internal/v1/workspaces/{ws}/summary 404"; else bad "herkese açık giriş /internal/v1/workspaces/{ws}/summary ($code; 404 olmalıydı)"; fi
code=$(edge_code internal.demleme.local /internal/v1/workspaces)
if [ "$code" = 403 ]; then ok "dahili konak -> oauth2-proxy -> api /internal/v1 403 (jeton ve e-posta başlığı ulaştı)"; else bad "dahili konak /internal/v1/workspaces ($code; 403 olmalıydı)"; fi
code=$(edge_code internal.demleme.local /v1/workspaces)
if [ "$code" = 404 ]; then ok "dahili konak /internal dışındaki yolu sunmaz (404)"; else bad "dahili konak /v1/workspaces ($code; 404 olmalıydı)"; fi
# Web uygulaması konak adı (T3.1): /v1 api'ye ulaşır, /internal sunulmaz.
code=$(edge_code app.demleme.local /v1/version)
if [ "$code" = 200 ]; then ok "app konağı /v1/version api'ye ulaşır (200)"; else bad "app konağı /v1/version ($code; 200 olmalıydı)"; fi
code=$(edge_code app.demleme.local /v1/auth/github/start)
if [ "$code" = 404 ]; then ok "app konağı /v1/auth/{sağlayıcı}/start api'ye ulaşır (bilinmeyen sağlayıcı 404)"; else bad "app konağı /v1/auth/github/start ($code; 404 olmalıydı)"; fi
code=$(edge_code app.demleme.local /internal/v1/workspaces)
if [ "$code" = 404 ]; then ok "app konağı /internal/v1/... 404"; else bad "app konağı /internal/v1/... ($code; 404 olmalıydı)"; fi
kill "$pf" 2>/dev/null; wait "$pf" 2>/dev/null

echo "engine-service (gRPC)"
if out=$($K exec deploy/engine-service -- grpc_health_probe -addr=127.0.0.1:50051 2>&1); then
  ok "gRPC sağlık: $out"
else
  bad "gRPC sağlık: $out"
fi

echo "Ağ politikası ve mTLS (Istio ambient)"
# Gerçek pod'un içinden bash /dev/tcp ile denenir (imajda bash var, ek araç gerekmez). Ambient'te
# istemci tarafı connect() ztunnel'e başarılı olur; red sonradan gelir (bağlantı kapanır ya da
# zaman aşımı). Bu yüzden yalnız bağlanmak yetmez: sunucuya HTTP/2 giriş baytları gönderilir ve
# karşıdan en az bir bayt okunursa istek ulaşmış sayılır. Çıkış kodu 0 = yanıt geldi; 1 ya da
# 124 ve üstü = reddedildi (kapandı ya da zaman aşımı); 126/127 = deneme geçersiz (bash yok).
PROBE='exec 3<>/dev/tcp/$1/$2 || exit 2; printf "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" >&3; read -r -n1 -t 4 -u 3 _'
reach() { # kaynak-iş-yükü hedef-ad port; çıkış 0 = ulaştı
  $K exec "deploy/$1" -- timeout 10 bash -c "$PROBE" probe "$2" "$3" >/dev/null 2>&1
}
want_reach() { # kaynak hedef port
  reach "$1" "$2" "$3"; rc=$?
  if [ $rc = 0 ]; then ok "$1 -> $2:$3 izinli"; else bad "$1 -> $2:$3 ENGELLENDİ (çıkış $rc; izin verilmeliydi)"; fi
}
want_refuse() { # kaynak hedef port katman
  reach "$1" "$2" "$3"; rc=$?
  case $rc in
    0)       bad "$1 -> $2:$3 ERİŞTİ (reddedilmeliydi: $4)" ;;
    126|127) bad "$1 -> $2:$3 denemesi geçersiz (çıkış $rc)" ;;
    *)       ok "$1 -> $2:$3 reddedildi ($4)" ;;
  esac
}
want_reach api engine-service 50051

# Veri servisinde connect() tek başına erişimi doğrulamaz. Protokol yanıtını bekle.
# Uzak betik sonuç yazar. kubectl ya da timeout hatası bir ret sonucu sayılmaz.
DATA_PROBE='
  trap "" PIPE
  if { exec 3<>"/dev/tcp/$1/$2"; } 2>/dev/null; then
    case "$1" in
      postgres) printf "\x00\x00\x00\x08\x04\xd2\x16\x2f" >&3 ;;
      redis)    printf "*1\r\n\$4\r\nPING\r\n" >&3 ;;
      nats)     : ;; # NATS bağlantı açıldığında INFO gönderir.
      *) exit 2 ;;
    esac
    if IFS= read -r -n1 -t 4 -u 3 _; then
      printf "yanit"
      exit 0
    fi
  fi
  printf "red"
'
check_data_access() { # kaynak hedef port beklenen-sonuç
  local result
  if ! result=$($K exec "deploy/$1" -- timeout 10 bash -c "$DATA_PROBE" probe "$2" "$3" 2>/dev/null); then
    bad "$1 -> $2:$3 denemesi çalışmadı"
  elif [ "$result" = "$4" ]; then
    ok "$1 -> $2:$3 ($result)"
  else
    bad "$1 -> $2:$3 ($result; beklenen $4)"
  fi
}
check_data_access api postgres 5432 yanit
check_data_access api redis 6379 yanit
check_data_access api nats 4222 yanit
check_data_access engine-service postgres 5432 red
check_data_access engine-service redis 6379 red
check_data_access engine-service nats 4222 red

# Mesh: kimlik politikaları (L4). api'nin portu ağ politikasıyla herkese açıktır; bu yüzden yalnız
# AuthorizationPolicy reddedebilir (kimlik: ServiceAccount). Hesap motoruna yalnız api erişir.
if [ "$($K get ns "$NS" -o jsonpath='{.metadata.labels.istio\.io/dataplane-mode}' 2>/dev/null)" = ambient ]; then
  ok "ad alanı ambient modda"
else
  bad "ad alanında istio.io/dataplane-mode=ambient yok"
fi
if [ "$($K get peerauthentication varsayilan -o jsonpath='{.spec.mtls.mode}' 2>/dev/null)" = STRICT ]; then
  ok "PeerAuthentication STRICT"
else
  bad "PeerAuthentication varsayilan STRICT değil"
fi
want_refuse worker api 8080 "mesh kimliği worker, AuthorizationPolicy"
want_refuse ingest api 8080 "mesh kimliği ingest, AuthorizationPolicy"
want_refuse worker engine-service 50051 "mesh kimliği worker"
want_refuse ingest engine-service 50051 "mesh kimliği ingest"
want_refuse oauth2-proxy engine-service 50051 "mesh kimliği oauth2-proxy"

# Mesh dışı pod: düz metin istek. Ayrı ad alanında (ambient etiketi yok), api imajıyla (bash var).
OUT_NS=demleme-disi
KO="kubectl --context kind-demleme -n $OUT_NS"
kubectl --context kind-demleme delete namespace "$OUT_NS" --ignore-not-found --wait=true --timeout=90s >/dev/null 2>&1
kubectl --context kind-demleme create namespace "$OUT_NS" >/dev/null 2>&1
if $KO run disi --image=demleme/api:dev --image-pull-policy=Never --restart=Never --command -- sleep 600 >/dev/null 2>&1 \
   && $KO wait --for=condition=Ready pod/disi --timeout=60s >/dev/null 2>&1; then
  out_reach() { $KO exec disi -- timeout 10 bash -c "$PROBE" probe "$1" "$2" >/dev/null 2>&1; }
  # Kontrol: mesh dışı pod Gateway podunun düz metin portuna ulaşabilir; deneme geçerlidir.
  out_reach "kenar-istio.$NS" 80; rc=$?
  if [ $rc = 0 ]; then ok "kontrol: mesh dışı pod -> kenar-istio:80 ulaştı (deneme geçerli)"; else bad "kontrol: mesh dışı pod -> kenar-istio:80 ulaşamadı (çıkış $rc)"; fi
  out_reach "api.$NS" 8080; rc=$?
  if [ $rc != 0 ]; then ok "mesh dışı düz metin istek -> api:8080 reddedildi (STRICT mTLS)"; else bad "mesh dışı düz metin istek -> api:8080 ERİŞTİ (reddedilmeliydi)"; fi
  out_reach "engine-service.$NS" 50051; rc=$?
  if [ $rc != 0 ]; then ok "mesh dışı düz metin istek -> engine-service:50051 reddedildi"; else bad "mesh dışı düz metin istek -> engine-service:50051 ERİŞTİ (reddedilmeliydi)"; fi
else
  bad "mesh dışı deneme pod'u başlamadı"
fi
kubectl --context kind-demleme delete namespace "$OUT_NS" --wait=false >/dev/null 2>&1

if [ "${NETPOL:-1}" = 0 ] || ! kubectl -n kube-system get ds calico-node >/dev/null 2>&1; then
  echo "  UYARI: politikayı uygulayan CNI yok (calico-node bulunamadı) ya da NETPOL=0; ağ politikası denetimi atlandı"
else
  # Ağ politikası (ikinci katman): Redis'e yalnız api erişir; worker mesh kimliği olsa da erişemez.
  want_refuse worker redis 6379 "ağ politikası"
fi
[ "$fail" = 0 ] && echo "TAMAM" || echo "BAŞARISIZ"
exit "$fail"

#!/usr/bin/env bash
# kind kümesi kurar, Calico (NetworkPolicy) ve imajları yükler, yerel overlay'i uygular.
# Kullanım: kökte `make kind-up` (proxy arkasında: make kind-up BUILD_CA_FILE=/yol/ca.crt;
# derlemeyi atlamak için SKIP_BUILD=1).
set -euo pipefail
cd "$(dirname "$0")/../.."
CLUSTER=demleme
NS=demleme
IMAGES=(demleme/api:dev demleme/engine-service:dev demleme/worker:dev demleme/ingest:dev demleme/migrate:dev demleme/fake-oidc:dev)
# Küme imaj çekmesin diye bağımlılık imajları da yüklenir.
DEPS=(nats:2-alpine postgres:17-alpine redis:7-alpine
      chrislusf/seaweedfs:4.05 axllent/mailpit:v1.27 wiremock/wiremock:3.13.2
      nginxinc/nginx-unprivileged:1.27-alpine quay.io/oauth2-proxy/oauth2-proxy:v7.9.0
      curlimages/curl:8.11.1)
CALICO_VERSION=v3.30.3
CALICO_SHA256=9382d2b27a76f40c170454b408653e6d71e2205ef0aef069e942bb690e7381d0
CALICO_IMAGES=(cni node kube-controllers)
# Istio ambient (ADR-0009 › 10: servisler arası mTLS). istioctl sürüme sabit; to be verified:
# istioctl arşivinin sağlama toplamı burada sabitlenmedi, yayın sayfasındaki .sha256 dosyasıyla denetlenir.
ISTIO_VERSION=1.27.3
ISTIO_IMAGES=(pilot ztunnel install-cni)
case "$(uname -m)" in x86_64) ARCH=amd64 ;; aarch64|arm64) ARCH=arm64 ;; *) ARCH=$(uname -m) ;; esac

load_image() {
  # `kind load docker-image` Docker'ın containerd imaj deposunda ("content digest not found") başarısız
  # olabilir; tek platformlu arşivle yüklenir (--platform eski Docker'larda yoksa düz save).
  { docker save --platform "linux/$ARCH" "$1" 2>/dev/null || docker save "$1"; } \
    | kind load image-archive /dev/stdin --name "$CLUSTER"
}

if [ -n "${BUILD_CA_FILE:-}" ]; then cp "$BUILD_CA_FILE" altyapi/compose/.build-ca.pem; else : > altyapi/compose/.build-ca.pem; fi

echo "==> Uygulama imajları derleniyor (docker compose build)"
# Docker Hub 429 (hız sınırı) geçici olabilir: 3 deneme; hepsi başarısızsa ve imajlar yerelde
# zaten varsa uyarıp eldekilerle sürer. SKIP_BUILD=1 derlemeyi tümüyle atlar.
built=0
if [ "${SKIP_BUILD:-0}" != 1 ]; then
  for try in 1 2 3; do
    if docker compose -f altyapi/compose/compose.yaml build api engine-service worker ingest migrate fake-oidc; then built=1; break; fi
    echo "derleme başarısız (deneme $try/3)"; sleep 30
  done
fi
if [ "$built" = 0 ]; then
  for i in "${IMAGES[@]}"; do
    docker image inspect "$i" >/dev/null 2>&1 || { echo "HATA: $i yok ve derlenemedi" >&2; exit 1; }
  done
  echo "UYARI: derleme atlandı/başarısız; yerelde var olan imajlar kullanılıyor"
fi

echo "==> Bağımlılık imajları"
for i in "${DEPS[@]}"; do docker image inspect "$i" >/dev/null 2>&1 || docker pull "$i"; done

if kind get clusters 2>/dev/null | grep -qx "$CLUSTER"; then
  echo "==> Küme zaten var: $CLUSTER"
else
  echo "==> Küme oluşturuluyor"
  # CNI yok: düğüm Calico kurulana kadar Ready olmaz, bu yüzden --wait yok.
  kind create cluster --config altyapi/k8s/kind/kind.yaml
fi
kubectl config use-context "kind-$CLUSTER" >/dev/null

echo "==> Calico (NetworkPolicy) $CALICO_VERSION"
# Manifest sürüme sabitlenir ve sağlama toplamıyla doğrulanır. İmajlar quay.io'dan çekilip
# manifestteki docker.io adlarıyla etiketlenir (Docker Hub hız sınırını da dolaşır).
CACHE=altyapi/k8s/kind/.cache; mkdir -p "$CACHE"
MANIFEST="$CACHE/calico-$CALICO_VERSION.yaml"
if [ ! -f "$MANIFEST" ]; then
  curl -fsSL -o "$MANIFEST" "https://raw.githubusercontent.com/projectcalico/calico/$CALICO_VERSION/manifests/calico.yaml"
fi
echo "$CALICO_SHA256  $MANIFEST" | sha256sum -c - >/dev/null \
  || { echo "HATA: Calico manifest sağlama toplamı uyuşmuyor" >&2; rm -f "$MANIFEST"; exit 1; }
for c in "${CALICO_IMAGES[@]}"; do
  if ! docker image inspect "docker.io/calico/$c:$CALICO_VERSION" >/dev/null 2>&1; then
    docker pull -q "quay.io/calico/$c:$CALICO_VERSION" >/dev/null
    docker tag "quay.io/calico/$c:$CALICO_VERSION" "docker.io/calico/$c:$CALICO_VERSION"
  fi
  load_image "docker.io/calico/$c:$CALICO_VERSION"
done
kubectl apply -f "$MANIFEST" >/dev/null
kubectl -n kube-system rollout status daemonset/calico-node --timeout=240s
kubectl -n kube-system rollout status deployment/calico-kube-controllers --timeout=240s
kubectl wait --for=condition=Ready node --all --timeout=120s

echo "==> Istio ambient $ISTIO_VERSION (mTLS)"
case "$(uname -s)" in Darwin) ISTIO_OS=osx ;; *) ISTIO_OS=linux ;; esac
ISTIO_ARCH=$ARCH; [ "$ISTIO_OS" = osx ] && [ "$ARCH" = amd64 ] && ISTIO_ARCH=""
ISTIO_FILE="istioctl-$ISTIO_VERSION-$ISTIO_OS${ISTIO_ARCH:+-$ISTIO_ARCH}.tar.gz"
ISTIO_URL="https://github.com/istio/istio/releases/download/$ISTIO_VERSION/$ISTIO_FILE"
ISTIOCTL="$CACHE/istioctl-$ISTIO_VERSION"
if [ ! -x "$ISTIOCTL" ]; then
  tmp=$(mktemp -d)
  curl -fsSL -o "$tmp/$ISTIO_FILE" "$ISTIO_URL"
  curl -fsSL -o "$tmp/$ISTIO_FILE.sha256" "$ISTIO_URL.sha256"
  (cd "$tmp" && sha256sum -c "$ISTIO_FILE.sha256" >/dev/null 2>&1 || shasum -a 256 -c "$ISTIO_FILE.sha256" >/dev/null) \
    || { echo "HATA: istioctl sağlama toplamı uyuşmuyor" >&2; rm -rf "$tmp"; exit 1; }
  tar -xzf "$tmp/$ISTIO_FILE" -C "$tmp" istioctl
  mv "$tmp/istioctl" "$ISTIOCTL"; rm -rf "$tmp"
fi
for c in "${ISTIO_IMAGES[@]}"; do
  img="docker.io/istio/$c:$ISTIO_VERSION"
  docker image inspect "$img" >/dev/null 2>&1 || docker pull -q "$img" >/dev/null
  load_image "$img"
done
"$ISTIOCTL" install --set profile=ambient --skip-confirmation >/dev/null
kubectl -n istio-system rollout status deployment/istiod --timeout=240s
kubectl -n istio-system rollout status daemonset/istio-cni-node --timeout=240s
kubectl -n istio-system rollout status daemonset/ztunnel --timeout=240s

echo "==> İmajlar kümeye yükleniyor"
for i in "${IMAGES[@]}" "${DEPS[@]}"; do load_image "$i"; done

echo "==> Yerel sırlar ve sahte servis eşlemeleri"
# Ad alanı önce açılır; sırlar ve eşlemeler overlay'den önce hazır olmalı (kustomize kök dışı dosya okumaz).
kubectl create namespace "$NS" --dry-run=client -o yaml | kubectl apply -f - >/dev/null
# oauth2-proxy sırları her küme için rastgele üretilir (yalnız yerel; mevcutsa değişmez).
if ! kubectl -n "$NS" get secret demleme-oauth2-proxy >/dev/null 2>&1; then
  kubectl -n "$NS" create secret generic demleme-oauth2-proxy \
    --from-literal=OAUTH2_PROXY_CLIENT_ID=demleme-destek-yerel \
    --from-literal=OAUTH2_PROXY_CLIENT_SECRET="$(openssl rand -hex 16)" \
    --from-literal=OAUTH2_PROXY_COOKIE_SECRET="$(openssl rand -hex 16)" \
    --from-literal=INTERNAL_PROXY_TOKEN="$(openssl rand -hex 24)" >/dev/null
fi
for m in github apns; do
  kubectl -n "$NS" create configmap "demleme-$m-mock-eslemeler" \
    --from-file="altyapi/compose/mocks/$m/mappings" --dry-run=client -o yaml | kubectl apply -f - >/dev/null
done

echo "==> Yerel overlay uygulanıyor"
kubectl apply -k altyapi/k8s/overlays/yerel

echo "==> Hazır olması bekleniyor"
for r in statefulset/nats statefulset/postgres statefulset/redis statefulset/objectstore \
         deployment/mailpit deployment/github-mock deployment/apns-mock deployment/fake-oidc \
         deployment/engine-service deployment/api deployment/worker deployment/ingest \
         deployment/edge deployment/oauth2-proxy; do
  kubectl -n "$NS" rollout status "$r" --timeout=240s
done
kubectl -n "$NS" wait --for=condition=complete job/migrate --timeout=240s
kubectl -n "$NS" get pods

#!/usr/bin/env bash
# kind kümesi kurar, Calico (NetworkPolicy) ve imajları yükler, yerel overlay'i uygular.
# Kullanım: kökte `make kind-up` (proxy arkasında: make kind-up BUILD_CA_FILE=/yol/ca.crt;
# derlemeyi atlamak için SKIP_BUILD=1).
set -euo pipefail
cd "$(dirname "$0")/../.."
CLUSTER=demleme
NS=demleme
IMAGES=(demleme/api:dev demleme/engine-service:dev demleme/worker:dev demleme/ingest:dev demleme/migrate:dev)
# Küme imaj çekmesin diye bağımlılık imajları da yüklenir.
DEPS=(nats:2-alpine postgres:17-alpine redis:7-alpine)
CALICO_VERSION=v3.30.3
CALICO_SHA256=9382d2b27a76f40c170454b408653e6d71e2205ef0aef069e942bb690e7381d0
CALICO_IMAGES=(cni node kube-controllers)
GATEWAY_API_VERSION=v1.4.0
GATEWAY_API_SHA256=6a4029e661446d64add866a00ecdc40c14219b68777ab614c5cdaac0adb481f1
ISTIO_VERSION=1.28.0
ISTIOCTL_SHA256_AMD64=60bd2f5afb7f175d3851315bc75f3c1335d9e051941743b034a7ed7edccf308f
ISTIOCTL_SHA256_ARM64=7b2c873cd9d6f2ce1585affd962c5eb05b594d27e236e9ce95b6fe6e34c3dbfb
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
    if docker compose -f altyapi/compose/compose.yaml build api engine-service worker ingest migrate; then built=1; break; fi
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

echo "==> Gateway API $GATEWAY_API_VERSION CRD'leri ve Istio $ISTIO_VERSION (yalnız kenar için; ambient #207'de)"
# Gateway API standart CRD'leri ve istioctl sürüme sabitlenir, sağlama toplamıyla doğrulanır.
GWAPI="$CACHE/gateway-api-$GATEWAY_API_VERSION.yaml"
if [ ! -f "$GWAPI" ]; then
  curl -fsSL -o "$GWAPI" "https://github.com/kubernetes-sigs/gateway-api/releases/download/$GATEWAY_API_VERSION/standard-install.yaml"
fi
echo "$GATEWAY_API_SHA256  $GWAPI" | sha256sum -c - >/dev/null \
  || { echo "HATA: Gateway API manifest sağlama toplamı uyuşmuyor" >&2; rm -f "$GWAPI"; exit 1; }
kubectl apply --server-side -f "$GWAPI" >/dev/null
ISTIOCTL_TGZ="$CACHE/istioctl-$ISTIO_VERSION-linux-$ARCH.tar.gz"
case "$ARCH" in amd64) ISTIOCTL_SHA256=$ISTIOCTL_SHA256_AMD64 ;; arm64) ISTIOCTL_SHA256=$ISTIOCTL_SHA256_ARM64 ;; *) echo "HATA: istioctl için desteklenmeyen mimari $ARCH" >&2; exit 1 ;; esac
if [ ! -f "$ISTIOCTL_TGZ" ]; then
  curl -fsSL -o "$ISTIOCTL_TGZ" "https://github.com/istio/istio/releases/download/$ISTIO_VERSION/istioctl-$ISTIO_VERSION-linux-$ARCH.tar.gz"
fi
echo "$ISTIOCTL_SHA256  $ISTIOCTL_TGZ" | sha256sum -c - >/dev/null \
  || { echo "HATA: istioctl sağlama toplamı uyuşmuyor" >&2; rm -f "$ISTIOCTL_TGZ"; exit 1; }
ISTIOCTL_DIR="$CACHE/istioctl-$ISTIO_VERSION-$ARCH"; mkdir -p "$ISTIOCTL_DIR"
tar -xzf "$ISTIOCTL_TGZ" -C "$ISTIOCTL_DIR"
for i in pilot proxyv2; do
  docker image inspect "docker.io/istio/$i:$ISTIO_VERSION" >/dev/null 2>&1 || docker pull -q "docker.io/istio/$i:$ISTIO_VERSION" >/dev/null
  load_image "docker.io/istio/$i:$ISTIO_VERSION"
done
"$ISTIOCTL_DIR/istioctl" install --set profile=minimal --set hub=docker.io/istio --set tag="$ISTIO_VERSION" -y
kubectl -n istio-system rollout status deployment/istiod --timeout=240s

echo "==> İmajlar kümeye yükleniyor"
for i in "${IMAGES[@]}" "${DEPS[@]}"; do load_image "$i"; done

echo "==> Yerel overlay uygulanıyor"
kubectl apply -k altyapi/k8s/overlays/yerel

echo "==> Hazır olması bekleniyor"
for r in statefulset/nats statefulset/postgres statefulset/redis \
         deployment/engine-service deployment/api deployment/worker deployment/ingest \
         deployment/oauth2-proxy; do
  kubectl -n "$NS" rollout status "$r" --timeout=240s
done
# Istio, Gateway için `kenar-istio` Deployment'ını ve Servisini kendisi üretir.
for n in $(seq 1 60); do kubectl -n "$NS" get deployment/kenar-istio >/dev/null 2>&1 && break; sleep 2; done
kubectl -n "$NS" rollout status deployment/kenar-istio --timeout=240s
kubectl -n "$NS" wait --for=condition=complete job/migrate --timeout=240s
kubectl -n "$NS" get pods

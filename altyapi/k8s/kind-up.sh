#!/usr/bin/env bash
# kind kümesi kurar, Calico (NetworkPolicy) ve imajları yükler, yerel overlay'i uygular.
# Kullanım: kökte `make kind-up` (proxy arkasında: make kind-up BUILD_CA_FILE=/yol/ca.crt;
# derlemeyi atlamak için SKIP_BUILD=1).
set -euo pipefail
cd "$(dirname "$0")/../.."
CLUSTER=demleme
NS=demleme
IMAGES=(demleme/api:dev demleme/engine-service:dev demleme/worker:dev demleme/ingest:dev)
# Küme imaj çekmesin diye bağımlılık imajları da yüklenir.
DEPS=(nats:2-alpine postgres:17-alpine redis:7-alpine)
CALICO_VERSION=v3.30.3
CALICO_SHA256=9382d2b27a76f40c170454b408653e6d71e2205ef0aef069e942bb690e7381d0
CALICO_IMAGES=(cni node kube-controllers)
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
    if docker compose -f altyapi/compose/compose.yaml build api engine-service worker ingest; then built=1; break; fi
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

echo "==> İmajlar kümeye yükleniyor"
for i in "${IMAGES[@]}" "${DEPS[@]}"; do load_image "$i"; done

echo "==> Yerel overlay uygulanıyor"
kubectl apply -k altyapi/k8s/overlays/yerel

echo "==> Hazır olması bekleniyor"
for r in statefulset/nats statefulset/postgres statefulset/redis \
         deployment/engine-service deployment/api deployment/worker deployment/ingest; do
  kubectl -n "$NS" rollout status "$r" --timeout=240s
done
kubectl -n "$NS" get pods

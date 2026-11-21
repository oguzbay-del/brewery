# altyapi/k8s

Kubernetes tanımları (Kustomize; Helm yok). İş: T0.7.

| Yol | İçerik |
|---|---|
| `base/` | api, engine-service, worker, ingest (Deployment + Service), `migrate` (veritabanı göç Job'ı), NATS JetStream (StatefulSet), ConfigMap, NetworkPolicy |
| `overlays/yerel/` | base + PostgreSQL ve Redis (küme içinde, tek kopya), yerel imaj etiketi `dev`, yalnız yerel örnek Secret |
| `overlays/test/`, `overlays/canli/` | base + kopya sayısı ve kaynaklar; PostgreSQL ve Redis YOK (Cloud SQL, Memorystore) |
| `kind/` | yerel küme yapılandırması |

## Komutlar (depo kökünden)

| Komut | Ne yapar |
|---|---|
| `make kind-up` | Küme kurar, Calico ve imajları yükler, `overlays/yerel` uygular, hazır olmayı bekler |
| `make kind-test` | Podlar Ready, `/healthz` 200, engine-service gRPC SERVING, ağ politikası denetimi |
| `make kind-down` | Kümeyi siler |

Gereken: Docker, `kind`, `kubectl` (PATH'te). TLS proxy arkasında: `make kind-up BUILD_CA_FILE=/yol/ca.crt`. Derlemeyi atlamak: `SKIP_BUILD=1`.
Doğrulama: `kubectl kustomize altyapi/k8s/overlays/<ad>` ve `kubectl apply --dry-run=client -k ...` (ikincisi çalışan bir küme ister).

## Kararlar

- **Ağ:** ad alanında girişler varsayılan kapalı. engine-service'e yalnız api erişir; api ve ingest dışarıdan gelen trafiği alır; NATS, PostgreSQL ve Redis yalnız api, worker, ingest'e açık. Çıkış kısıtlanmaz.
- **CNI:** kind'ın varsayılan CNI'si (kindnet) NetworkPolicy için nftables ister; bazı çekirdeklerde çalışmaz. Bu yüzden kind-up Calico (iptables) kurar; manifest sürüme ve sağlama toplamına sabit, imajlar quay.io'dan.
- **Güvenlik:** root olmayan kullanıcı, salt okunur kök dosya sistemi, bütün yetenekler kapalı, `seccomp: RuntimeDefault`, ad alanında Pod Security `restricted`.
- **Göç:** `base/migrate.yaml` tek seferlik Job; `migrate` imajı `sunucu/migrations`'ı uygular, göç rolünün adresini (`MIGRATE_DATABASE_URL`) ve `app_api`/`app_worker` parolalarını `demleme-migrate` Secret'ından alır, göçten sonra bu iki role oturum açma hakkı ve parolayı verir. `make kind-up` Job'ın bitmesini bekler; servislerin göçten sonra başlaması henüz zorlanmıyor (servisler veritabanı kullanmıyor; T2.10'da eklenir). Job şablonu değişmez olduğundan imaj etiketi değişen canlı dağıtımda önce eski Job silinir (`ttlSecondsAfterFinished` 1 saat sonra kendisi siler).
- **Gizli bilgi:** test ve canlıda `demleme` ad alanında `demleme-dis-servisler` Secret'ı (anahtarlar `DATABASE_URL` = `app_api`, `WORKER_DATABASE_URL` = `app_worker`, `REDIS_URL`) ve `demleme-migrate` Secret'ı (`MIGRATE_DATABASE_URL`, `APP_API_PASSWORD`, `APP_WORKER_PASSWORD`) dışarıdan sağlanır; depoda değer yoktur. Yerelde aynı ad compose'daki örnek değerlerle üretilir.
- **İmajlar:** test ve canlıda `KAYIT-DEFTERI` ve `SURUM` yer tutucudur; CI `kustomize edit set image` ile doldurur (kayıt defteri doğrulanacak).

## Bilinen sınırlar

- NATS her ortamda tek kopya; üç düğümlü kümeleme ayrı iş (doğrulanacak). Test ve canlı overlay'leri kümede denenmedi, yalnız `kustomize` ve `dry-run` ile doğrulandı.
- kind yapılandırmasında iki ortam yaması var: cgroup v1 ana makine için `failCgroupV1: false`, CAP_SYS_RESOURCE'suz Docker için containerd `restrict_oom_score_adj` (açıklamaları `kind/kind.yaml` içinde).
- `kind load docker-image` Docker'ın containerd imaj deposunda başarısız olabildiğinden imajlar `docker save --platform` ile yüklenir.

## T2.32 notes (STE)

Layout additions:

- `base/edge.yaml`: nginx reverse proxy. It is the public ingress in kind. It answers 404 for `/internal` and removes the proxy headers from client requests.
- `base/oauth2-proxy.yaml`: OIDC reverse proxy for the support panel (`/internal`). It adds `X-Auth-Request-Email` and `X-Internal-Proxy-Token` to each request for the API.
- `base/serviceaccounts.yaml`: one service account for each workload. The mesh identity comes from the service account.
- `components/istio-ambient/`: namespace label, `PeerAuthentication` STRICT, `AuthorizationPolicy` for engine-service, api and NATS, and the NetworkPolicy for the ztunnel probe address and HBONE port 15008. `overlays/yerel`, `overlays/test` and `overlays/canli` use it. Istio ambient must be installed first (`make kind-up` installs it; to be verified for the cloud clusters).
- `components/external-secrets/`: `ExternalSecret` objects for `demleme-dis-servisler`, `demleme-migrate` and `demleme-oauth2-proxy`. The `test` and `canli` overlays use it. It needs the External Secrets Operator and a `ClusterSecretStore` named `gcp-secret-manager`. The secret IDs match `terraform/modules/gcp-secrets`.
- `overlays/yerel/sahte.yaml`: object store (SeaweedFS), fake OIDC provider, mailpit, GitHub and APNs mocks. `make kind-up` creates the mock mapping ConfigMaps from `altyapi/compose/mocks` and a random `demleme-oauth2-proxy` Secret before it applies the overlay. Run `make kind-up`, not only `kubectl apply -k`.

Checks in `make kind-test` (new):

- `/internal/...` through the `edge` service gives 404.
- `oauth2-proxy` does not let a request without a session reach the API (302, 401 or 403).
- A plain-text request from a pod outside the mesh to `api` and to `engine-service` is rejected (`PeerAuthentication` STRICT).
- A request from a pod with the wrong service account to `api` is rejected; the `edge` service account passes (`AuthorizationPolicy`).
- `worker` and `ingest` cannot connect to `engine-service`. The TCP check reads after it connects, because ztunnel accepts the connection before it rejects it.

Limits:

- ztunnel enforces only layer 4 rules. The API has no separate `/internal` port, so the mesh cannot allow only `oauth2-proxy` for `/internal`. The `edge` 404 and the API token check give the separation. A separate port or a waypoint proxy closes this gap (to be verified).
- Local PostgreSQL and Redis do not use TLS (ADR-0009 › 10 requires it in the cloud; Cloud SQL `ENCRYPTED_ONLY` and Memorystore `SERVER_AUTHENTICATION` are set in Terraform).
- Google Cloud Storage has no S3 Object Lock API. The digest bucket uses a retention policy (`gcp-storage`, 400 days). Do not set `S3_OBJECT_LOCK_MODE` on Google Cloud (to be verified). SeaweedFS object lock is not configured locally.
- The `istioctl` archive is checked against the `.sha256` file of the release. Pin the hash later (to be verified). Istio 1.27.3 and oauth2-proxy v7.9.0 were chosen from memory and not run in this container (to be verified in CI).

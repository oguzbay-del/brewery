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

# ADR-0002: Aşama 0 kapanışı ve plandan sapmalar

**Durum:** kabul · **Tarih:** 11 Kasım 2026 · **İş:** T0.11

## Kapı

| Kapı maddesi | Sonuç |
|---|---|
| Bütün sistem yerelde tek komutla | `make up`: 10 servis sağlıklı; `make smoke` geçiyor |
| CI yeşil | 6 iş akışı (sunucu, sözleşme, web, belgeler, altyapı, iOS) GitHub'da geçiyor; iOS gerçek macOS makinesinde derleniyor ve simülatörde test ediliyor |
| kind duman testi | `make kind-up && make kind-test`: 7 pod hazır, sağlık uçları, gRPC sağlığı, ağ politikası (hesap motoruna yalnız api) |

## Plandan ve ADR-0001'den sapmalar

- **Proto derleyici:** sistem `protoc` yerine saf Rust `protox`; derleme ortamına bağımlılık yok.
- **Çalışma imajı:** distroless yerine debian-slim (sağlık denetimi için kabuk gerekiyor), root olmayan kullanıcı.
- **Kubernetes ağı:** kind'da varsayılan CNI (kindnet) ağ politikasını bu ortamda uygulayamadı; yerel kümede Calico. Canlıda GKE'nin kendi ağ politikası kullanılır.
- **Ortama özgü yamalar** (yalnız yerel kind ve imaj derlemesi; canlıyı etkilemez): kind yapılandırmasında cgroup v1 izni ve `restrict_oom_score_adj`; imaj derlemesinde isteğe bağlı CA (`BUILD_CA_FILE`, TLS vekili olan ortamlar için).
- **Araç indirme:** kind ve actionlint GitHub sürümlerinden indirilemedi (vekil); Go modül deposundan sağlama toplamıyla derlendi.
- **Tasarım sistemi:** koyu tema için yüksek karşıtlık değerleri eksikti; T0.12 ile eklendi.
- **`.gitignore`:** kök `src/` kuralı her düzeydeki `src` klasörünü yok sayıyordu; `/src/` yapıldı (kaynak kodun bir kısmı ilk commit'lerde eksik kalmıştı).

## Açık kalanlar (Aşama 0'ı kapatmaya engel değil)

- Terraform: denetim kovasında WORM kilidi ve saklama süresi, CMEK kapsamı, yedeğin başka bölgeye kopyası, CI için Workload Identity.
- Test ve canlı Kubernetes overlay'leri yalnız kustomize ve dry-run ile doğrulandı; bulut hesabı (O4) gelince kurulur.
- NATS tek kopya; kümeleme Aşama 2'de.
- iOS: gerçek AppIcon (logo bekliyor); Xcode sürümünün CI'da sabitlenmesi.

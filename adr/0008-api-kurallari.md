# ADR-0008: API kuralları

**Durum:** kabul · **Tarih:** 13 Kasım 2026 · **İş:** T2.2

## Bağlam

Çekirdek API'yi üç istemci kullanacak: SwiftUI, React ve destek paneli. Swift ve TypeScript istemci kodu tek OpenAPI dosyasından üretiliyor (plan › İki kez yazmama kuralı). Bu yüzden her uç aynı kuralları izlemeli: aynı hata biçimi, aynı sayfalama, aynı tekrar ve çakışma davranışı. Böylece istemcide bunlar bir kez yazılır.

Dayanaklar:
- plan › API ve istemci iletişimi: canlı güncelleme, sürüm uyumu, tekrar güvenliği, dış olaylar;
- plan › Eşzamanlı düzenleme ve onay;
- plan › Güvenlik;
- ADR-0007 (veri modeli; birimler, sürüm numarası, yumuşak silme).

Kimlik doğrulama ve yetkinin işleyişi bu kayıtta yok, ADR-0009'da.

## Karar

### 1. Yollar ve kaynaklar

1. **Sürüm önekli:** genel uçlar `/v1/...` altında. Sürümsüz olanlar yalnız `/healthz` ve `/readyz`.
2. **Çalışma alanı yolda:** çalışma alanı içeriği `/v1/workspaces/{workspace_id}/...` altında (ör. `/v1/workspaces/{ws}/recipes/{recipe_id}/versions/{version_id}`). Hesaba ait uçlar `/v1/me/...`, giriş `/v1/auth/...`, çalışma alanından bağımsız araç hesapları (refraktometre, karbonasyon, seyreltme) `/v1/tools/...`, tanımlar `/v1/meta/...` altında.
   Çalışma alanı başlıkta değil yolda. Böylece önbellek anahtarı, günlük ve yetki denetimi aynı yerden okunur; bir istek iki çalışma alanına karışmaz. API yoldaki kimliği üyelikle denetler, sonra veritabanı bağlamına yazar (ADR-0007 madde 2.3).
3. **Adlar:**
   - yol parçaları İngilizce, çoğul ve `kebab-case` (`brew-day-packages`);
   - JSON alanları `snake_case`, veritabanı ve proto ile aynı;
   - kimlikler UUID dizesi.
4. **Yöntemler:**
   - `GET` okur.
   - `POST` koleksiyona kayıt ekler; yanıt `201`, `Location` başlığı ve kaydın kendisi.
   - `PATCH` JSON Merge Patch (RFC 7396, `application/merge-patch+json`) ile kısmi günceller. `null` değeri siler, dizi bütün olarak değişir.
   - `DELETE` yumuşak siler, yanıt `204`.
   - `PUT` kullanılmaz.
5. **Durum geçişleri:** alanı değiştirerek değil, eylem alt yoluyla yapılır: `POST .../versions/{id}/publish`, `.../submit`, `.../approve`, `POST .../batches/{id}/steps/{step_id}/complete`, `POST .../{id}/restore`. Her eylem ayrı izinle denetlenir ve kendi denetim kaydını yazar.
6. **Uzun işler** (içe aktarma, dışa aktarma, hesap verisi talebi): `202 Accepted` döner, `Location` başlığı `/v1/workspaces/{ws}/jobs/{job_id}` adresini gösterir. İş kaydı durum ve sonuç bağlantısı taşır. İş bitince SSE olayı gelir (madde 9).

### 2. Veri biçimi

1. **Birim:** API, veritabanının metrik birimlerini olduğu gibi alır ve verir (ADR-0007 madde 1.4). Alan adı birim ekini taşır (`amount_kg`, `temp_c`). Birim parametresi yoktur. Kullanıcının seçtiği birime dönüşüm istemcide, üretilmiş katsayılarla yalnız gösterim için yapılır (T2.17).
2. **Sayılar:**
   - fiziksel büyüklükler JSON sayısı;
   - para ondalık dize (`"12.3400"`) ve yanında `currency` (ISO 4217); kayan nokta yuvarlaması bu yüzden yoktur;
   - an RFC 3339, UTC ve `Z` (`2026-11-13T09:30:00Z`); takvim günü `YYYY-MM-DD`.
3. **Sabit listeler:** `snake_case` dize. İstemci tanımadığı değere çökmez: üretilen Swift ve TypeScript tipleri bilinmeyen değeri taşıyan bir durum içerir. Sunucu yeni değer ekleyebilir; bu geriye uyumlu sayılır.
4. **Boş değer:** yanıtta şemadaki her alan bulunur, değer yoksa `null`. Tek istisna yetkiyle gizlenen alanlar (madde 2.5).
5. **Yetkiyle gizlenen alanlar:** maliyet ve fiyat gibi alanlar, okuma izni olmayana yanıtta hiç gönderilmez, `null` olarak da gönderilmez (plan › Güvenlik). OpenAPI'de bu alanlar isteğe bağlıdır ve `x-permission` ekini taşır (ör. `x-permission: cost.view`). İzni olmayan biri bu alanı yazmaya çalışırsa `403` alır.
6. **Kayıt üst bilgisi:** her kayıtta `id`, `version`, `created_at`, `updated_at`, `deleted_at` (çöp kutusu görünümünde dolu) bulunur.

### 3. Hatalar

1. **Biçim:** RFC 9457 Problem Details (`application/problem+json`).

   ```json
   {
     "type": "/errors/validation_failed",
     "title": "Validation failed",
     "status": 422,
     "code": "validation_failed",
     "request_id": "01JC…",
     "errors": [
       { "field": "/fermentables/0/amount_kg", "code": "out_of_range", "params": { "min": 0, "max": 1000 } }
     ]
   }
   ```

   `type` göreli bir adrestir; ürün adı ve alan adı belli olunca değişmesi gerekmez.

   Alanlar:
   - `code`: istemcinin dallandığı kararlı koddur.
   - `title`: İngilizce bir açıklamadır ve kullanıcıya gösterilmez. Kullanıcı metni koddan ve `params`'tan metin kataloğuyla üretilir.
   - `errors[].field`: istek gövdesine JSON Pointer (RFC 6901).
   Mevcut `ErrorResponse` zarfı T2.10'da bu biçime geçer.
2. **Kodlar** (liste kodda tek bir enum'dur, OpenAPI'ye ve metin kataloğuna oradan üretilir):

   | Durum | Kod | Ne zaman |
   |---|---|---|
   | 400 | `malformed_request` | JSON çözülemiyor, tip yanlış |
   | 401 | `unauthenticated` | Oturum yok, süresi dolmuş veya kapatılmış |
   | 403 | `forbidden` | Üye ama izni yok; gizli alana yazma |
   | 403 | `step_up_required` | İkinci adım (passkey) gerekiyor (ADR-0009) |
   | 403 | `entitlement_required` | Abonelik veya deneme bu özelliği açmıyor (salt okunur) |
   | 404 | `not_found` | Kayıt yok, silinmiş ya da başka çalışma alanında |
   | 409 | `conflict` | İş kuralı çakışması (ör. son yönetici kaldırılamaz); `errors` ayrıntıyı taşır |
   | 409 | `idempotency_in_progress` | Aynı anahtarlı istek hâlâ işleniyor |
   | 412 | `version_mismatch` | `If-Match` güncel sürümle uyuşmuyor (madde 5) |
   | 413 | `payload_too_large` | Gövde sınırı aşıldı |
   | 415 | `unsupported_media_type` | Yanlış `Content-Type` |
   | 422 | `validation_failed` | Alan doğrulaması; `errors` alan başına |
   | 422 | `idempotency_mismatch` | Aynı anahtar farklı gövdeyle geldi |
   | 422 | `calculation_failed` | Motor girdiyi reddetti (`InvalidInput` ve benzerleri; alan `errors`'ta) |
   | 426 | `upgrade_required` | Uygulama sürümü desteklenen en düşük sürümün altında (madde 8) |
   | 428 | `precondition_required` | `If-Match` zorunlu ama gönderilmemiş |
   | 429 | `rate_limited` | Hız sınırı; `Retry-After` başlığıyla |
   | 500 | `internal` | Beklenmeyen hata; ayrıntı yalnız günlükte |
   | 503 | `unavailable` | Bağımlı servis (motor, veritabanı) yanıt vermiyor; `Retry-After` |

3. **Başka çalışma alanı:** başka çalışma alanındaki kayıt ve üyesi olunmayan çalışma alanı `404` döner, `403` dönmez. Böylece kaydın var olduğu sızmaz. `403` yalnız üyenin kendi çalışma alanında izni olmadığında döner.
4. **Hata günlüğü:** her hata `request_id` ve iz kimliğiyle günlüğe yazılır. Yanıtta yığın izi, SQL ve iç adres bulunmaz.

### 4. Sayfalama, süzme, sıralama

1. **İmleçli sayfalama:** `?limit=50&cursor=…`. Varsayılan 50, en çok 200. Yanıt `{ "items": [...], "next_cursor": "…" | null }`.
   - İmleç opak bir base64url dizesidir; içinde sıralama anahtarı ve `id` bulunur. Bozuk imleç `400` döner.
   - Sayfa numarası ve toplam sayı yoktur. Gereken yerde ayrı bir `.../count` ucu açılır.
2. **Sıralama:** `?sort=-updated_at`. Her uç izinli alan listesini OpenAPI'de belirtir. Eşitlikte `id` ile kesin sıra sağlanır.
3. **Süzme:** alan adıyla sorgu parametresi (`?status=draft&folder_id=…`). Önbellek tazeleme için `?updated_since=<an>` parametresi silinenleri de getirir (`deleted_at` dolu satırlar). Çöp kutusu için `?include_deleted=true`.

### 5. Çakışma ve sürüm

1. **ETag:** her kayıt yanıtı `ETag: "<version>"` başlığını taşır (güçlü ETag; değer ADR-0007'deki `version`).
2. **Koşullu okuma:** `GET` isteği `If-None-Match` taşıyabilir, kayıt değişmediyse `304` döner. Liste uçları bunu desteklemez, bunun yerine `updated_since` kullanılır. Yanıtlarda `Cache-Control: private, no-cache` bulunur. Böylece istemci önbellekten hemen gösterir, arkada tazeler (plan › Platformlar).
3. **Koşullu yazma:** `PATCH`, `DELETE` ve durum geçişi eylemleri `If-Match` taşır. Davranışı kaynak türü belirler (plan › Eşzamanlı düzenleme ve onay); OpenAPI'de `x-concurrency` ekiyle yazılıdır:
   - **`optimistic`:** tarif taslağı, tarif sürümü eylemleri, parti durum geçişleri, roller. `If-Match` gönderilmemişse `428` döner. Sürüm uyuşmazsa `412 version_mismatch` döner ve gövdede `current` alanında kaydın güncel hali bulunur; istemci iki sürümü yan yana gösterir.
   - **`field_lww`:** ekipman, profiller, kütüphane malzemesi, ayarlar ve Ev modundaki tarif taslağı. `If-Match` isteğe bağlıdır. Merge Patch yalnız gönderilen alanları yazar (alan bazında son yazan kazanır). Önceki değer `change_history`'ye yazılır (ADR-0007).
   - **`append`:** ölçüm, aşama olayı, stok hareketi. Bunlar güncellenmez; düzeltme yeni kayıt olarak girer.
4. **Tarif düzenleme ve hesap:** taslağa gönderilen `PATCH` isteğinin yanıtı kaydın yanında güncel hesap özetini de (`calc`) taşır. İstemci ayrıca hesap isteği göndermez. Hesap sunucuda yapılır ve önbellektedir (T2.16).

### 6. Tekrar güvenliği

1. **Anahtar:** kayıt oluşturan her `POST` (eylemler dahil) `Idempotency-Key` başlığını taşır (IETF httpapi taslağı; istemcinin ürettiği UUID). Başlık yoksa istek `428` ile reddedilir. Yalnız okuyan `POST` uçları muaftır (araç hesapları) ve OpenAPI'de işaretlidir.
2. **Saklama:** anahtar, istek gövdesinin özeti ve yanıt 24 saat saklanır (`idempotency_keys`, ADR-0007).
3. **Davranış:**
   - Aynı anahtar ve aynı gövdeyle gelen istek saklanan yanıtı aynen döndürür ve `Idempotent-Replayed: true` başlığını ekler.
   - Aynı anahtar farklı gövdeyle gelirse `422 idempotency_mismatch`.
   - İlk istek hâlâ işleniyorsa `409 idempotency_in_progress`.
4. **Toplu ölçüm:** demleme günü kuyruğu `POST .../batches/{id}/measurements/bulk` ile gönderilir. Her ölçüm kendi UUID'sini ve paket kimliğini taşır.
   - Yanıt öğe başına sonuç verir: `created`, `duplicate` veya hata kodu.
   - Bir öğenin hatası ötekileri durdurmaz.
   - Aynı ölçümün yeniden gönderilmesi `duplicate` döner, ikinci kayıt oluşmaz.

### 7. Alan tanımları

1. **Tek kaynak:** alan sınırları (tip, birim, en az, en çok, adım, zorunluluk, sabit liste değerleri, gösterim hassasiyeti) Rust'ta tek bir yerde tanımlanır (T2.17). Sunucu doğrulaması, OpenAPI şemasındaki `minimum` ve `maximum`, ve `GET /v1/meta/fields` aynı tanımdan üretilir.
2. **İstemci:** istemci formu bu tanımlarla kurar ve yalnız kullanıcıya erken uyarı göstermek için kullanır. Asıl doğrulama her zaman sunucudadır (plan › İki kez yazmama kuralı).
3. **Önbellek:** `GET /v1/meta/fields` yanıtı ETag taşır ve istemci önbelleğinde uzun süre kalır.
4. **Mod farkı:** Ev ve İşletme modunda farklı sınır gerekiyorsa tanım bunu `modes` alanıyla belirtir.

### 8. Sürüm uyumu

1. **Yalnız ekleme:** `/v1` içinde yalnız geriye uyumlu değişiklik yapılır:
   - yeni uç;
   - yanıtta yeni alan;
   - istekte isteğe bağlı yeni alan;
   - sabit listeye yeni değer.
   Var olan alanın anlamı, birimi veya tipi değişmez. Alan kaldırılmaz; kullanımdan kalktığı OpenAPI'de `deprecated` ile işaretlenir. Uyumsuz değişiklik `/v2` ile gelir. `/v1` ondan sonra en az 6 ay açık kalır (doğrulanacak; plan › API ve istemci iletişimi).
2. **CI denetimi:** `sozlesme/scripts/check.sh`'nin yanına geriye uyumsuzluk denetimi eklenir. Araç oasdiff: Apache 2.0 lisanslı, tek ikili, sürümü sabitlenir. Dalın `openapi.json` dosyası `main`'dekiyle karşılaştırılır; uyumsuz fark CI'ı durdurur (T2.10).
3. **Uygulama sürümü:** istemci her istekte `X-App-Version: ios/1.4.2+87` (ya da `web/<derleme>`) başlığını gönderir. Sunucu desteklenen en düşük sürümü yapılandırmadan okur. Altındaki istemciye `426 upgrade_required` döner; istemci "güncelle" ekranını gösterir. `GET /v1/meta/client-requirements` girişten önce de okunabilir.

### 9. Canlı güncelleme (SSE)

1. **Uç:** `GET /v1/workspaces/{ws}/events` (`text/event-stream`). İsteğe bağlı `?topics=batch:{id},recipe:{id}` ile süzülür; parametre yoksa çalışma alanının bütün olayları gelir.
2. **Olay biçimi:**

   ```
   id: 01JC…                (outbox olayının UUIDv7'si)
   event: batch.measurement_added
   data: {"type":"batch.measurement_added","workspace_id":"…","entity":{"kind":"batch","id":"…","version":12},"occurred_at":"…"}
   ```

3. **Olay yalnız bildirimdir.** Olay kaydın kimliğini ve sürümünü taşır, içeriğini taşımaz. İstemci kaydı normal uçtan yeniden okur. Böylece yetki ve alan gizleme tek yerde (okuma ucunda) uygulanır; maliyet alanı olayla sızmaz.
   İstisnalar küçük ve yetkiden bağımsız yüklerdir. Örnekler: yumuşak kilit ("Ayşe düzenliyor", kullanıcı kimliği ve zaman) ve bildirim kutusu sayısı. Bunlar da OpenAPI'de şema olarak yazılır.
4. **Kaçan olaylar:** istemci yeniden bağlanırken `Last-Event-ID` gönderir. Sunucu o olaydan sonrasını NATS JetStream'den yeniden oynatır. Tutma süresi 1 saattir (doğrulanacak). Olay tutma süresinin dışındaysa sunucu `event: reset` gönderir; istemci açık ekranın verisini baştan okur.
5. **Bağlantı:** sunucu her 25 saniyede bir yorum satırı (`: ping`) gönderir.
   - Yetki bağlantı açılırken denetlenir.
   - Oturum kapatılırsa ya da üyelik veya rol değişirse sunucu akışı kapatır, istemci yeniden bağlanır.
   - Uygulama arka plandayken SSE kullanılmaz, push bildirimi gelir (plan › API ve istemci iletişimi).

### 10. İç uçlar (`/internal`)

1. **Ayrı dinleyici:** destek paneli uçları API sürecinin ayrı bir portunda, ayrı bir yönlendiricide çalışır. Genel giriş (Ingress) bu portu hiç yayımlamaz. Kubernetes ağ politikası bu porta yalnız OIDC ters vekilinden (oauth2-proxy) gelen trafiği bırakır (T2.32).
2. **Destek kimliği:** her iç istek, vekilin ilettiği kimlik belirtecini taşır. Belirteç API'de doğrulanır ve `staff_users`'la eşlenir (ADR-0009). Yalnız başlıkta gelen bir e-postaya güvenilmez.
3. **Aynı kurallar:** iç uçlar bu kaydın bütün kurallarına uyar (hata, sayfalama, tekrar güvenliği). Aynı OpenAPI dosyasının ayrı bir etiket grubunda (`internal`) yazılır. Müşteri istemcilerinin üretilen kodu bu grubu içermez.

### 11. Sınırlar, izleme, üst bilgiler

1. **Gövde sınırı:** JSON gövdesi en çok 1 MiB. Toplu ölçüm ucunda en çok 500 öğe gönderilir. Dosyalar API'den geçmez; kısa süreli imzalı bağlantıyla doğrudan nesne deposuna yüklenir (T2.18).
2. **Hız sınırı:** sınır kullanıcı, çalışma alanı ve IP başına uygulanır ve Redis'te sayılır. Yanıtta `RateLimit` ve `RateLimit-Policy` başlıkları bulunur (IETF httpapi taslağı). Aşılırsa `429` ve `Retry-After` döner. Redis yoksa sınır uygulanmaz, durum günlüğe yazılır (plan: Redis kaybı yalnız yavaşlatır).
3. **İzleme:** istemci `X-Request-Id` gönderebilir; göndermezse sunucu üretir. Kimlik her yanıtta geri döner. W3C `traceparent` başlığı OpenTelemetry izine bağlanır.
4. **Dil:** API değerleri dilden bağımsızdır. `Accept-Language` yalnız sunucunun ürettiği e-posta ve belgede (PDF, brew sheet) kullanılır.

### 12. OpenAPI

1. **Koddan üretim:** OpenAPI dosyası utoipa ile koddan üretilir, elle yazılmaz (ADR-0001).
2. **Adlandırma:**
   - `operationId`: `<kaynak>_<eylem>` (`recipes_list`, `recipe_versions_publish`);
   - etiketler modül adıdır (`recipes`, `batches`, `auth`, `internal`).
3. **Her uçta:** olası her hata kodu yanıt olarak yazılır, en az bir örnek bulunur ve `x-concurrency`, `x-permission` ekleri yazılır.
4. **Kimlik doğrulama:** güvenlik şeması iki tanedir: çerez ve Bearer (ADR-0009). Giriş öncesi uçlar `security: []` ile işaretlenir.

## Sonuçlar

- İstemcide hata gösterimi, sayfalama, yeniden deneme (tekrar anahtarıyla), çakışma ekranı ve SSE yeniden bağlanması birer kez yazılır. Bunlar üretilen istemcinin üstünde ortak bir katmandır.
- SSE olayının içerik taşımaması okuma sayısını biraz artırır, ama yetkinin tek yerde kalması bu bedele değer.
- T2.10 bu kuralların ara katmanlarını yazar ve her kurala test ekler: hata biçimi, `If-Match`, tekrar anahtarı, imleç, `426`, `429`, gövde sınırı. Sonraki alan işleri bu katmanları kullanır, kendi biçimini kurmaz.
- Açık: `/v1`'in açık kalma süresi (6 ay) ve SSE tutma süresi (1 saat) doğrulanacak.
- T2.18 uygulama notları (nesne deposu, dosya uçları): [0008-uygulama-notlari/t2-18.md](0008-uygulama-notlari/t2-18.md)

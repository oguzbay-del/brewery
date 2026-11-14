# ADR-0009: Kimlik, oturum ve yetki

**Durum:** kabul · **Tarih:** 13 Kasım 2026 · **İş:** T2.3

## Bağlam

Dayanaklar:
- plan › Kimlik ve giriş: ayrı kayıt yok, giriş Apple ve Google ile, yedek olarak e-posta bağlantısı; yöneticide passkey zorunlu; davet bağlantıyla; ortak cihaz PIN'le.
- plan › Güvenlik: öncelik sırası sunucuda yetki, çalışma alanı ayrımı, değişmez denetim kaydı, ortak cihazda kişiye özel giriş.
- plan › Kullanıcılar, roller ve yetkiler: matris, kapsam, korumalar.
- plan › Destek paneli: OIDC ters vekili, Google Workspace.
- ADR-0007: tablolar ve RLS. ADR-0008: yollar, hata kodları, `/internal`.

Kimlik doğrulama kodu kendimizde, ayrı kimlik servisi yok (plan › Mimari kararlar). Kodu Claude yazdığı için bu alan F2.2'de bağımsız incelenecek ve yayından önce sızma testine girecek (plan › Planlama işleri). Bu kayıt, o incelemenin neye bakacağını da belirliyor.

## Karar

### 1. Sağlayıcı girişi (Apple, Google)

1. **Akışlar:**
   - **iOS, Apple ile giriş:** `ASAuthorizationAppleIDProvider` kimlik belirteci ve yetkilendirme kodu üretir. İstemci ikisini `POST /v1/auth/apple` ile gönderir.
   - **iOS, Google ile giriş:** Google'ın iOS kütüphanesi sistem tarayıcı oturumunu (`ASWebAuthenticationSession`) ve PKCE'yi kullanır; uygulama içi web görünümü kullanılmaz (Google kuralı). Kimlik belirteci `POST /v1/auth/google` ile gönderilir.
   - **Web:** OIDC yetkilendirme kodu akışı ve PKCE, sunucu üzerinden yürür. `GET /v1/auth/{provider}/start` sağlayıcıya yönlendirir, sağlayıcı `.../callback`'e döner. Kod değişimini API yapar, tarayıcı belirteci hiç görmez.
     - Apple web'de `response_mode=form_post` ister. Bu siteler arası bir POST'tur. Bu yüzden durum çerezi (`__Host-auth`) yalnız bu akış için `SameSite=None; Secure` olur ve 10 dakika geçerlidir.
2. **Tek kullanımlık nonce ve durum:** her giriş denemesi önce sunucuda bir `auth_requests` satırı açar: nonce, durum (`state`), PKCE doğrulayıcısı (web), 10 dakika geçerlilik.
   - iOS bu satırı `POST /v1/auth/nonce` ile alır.
   - Sunucu belirteçteki `nonce`'u satırla karşılaştırır ve satırı tüketir. Apple'da istemci nonce'un SHA-256'sını gönderir; sunucu buna göre karşılaştırır.
   - Satır PostgreSQL'dedir (ADR-0007'ye eklendi). Redis düşse de giriş çalışır.
3. **Belirteç doğrulaması:** imza ve alanlar sağlayıcının yayımladığı JWKS ile denetlenir.
   - JWKS önbellektedir. Tanınmayan `kid` gelince yenilenir. En çok 24 saat tutulur.
   - `alg` yalnız `RS256` kabul edilir. `none` ve simetrik algoritmalar reddedilir.
   - `iss`: Apple için `https://appleid.apple.com`; Google için `https://accounts.google.com` veya `accounts.google.com`.
   - `aud`: yapılandırmadaki istemci kimlikleri listesinde olmalı (iOS paket kimliği, web hizmet kimliği, Google iOS ve web istemci kimlikleri).
   - `exp` ve `iat` 60 saniye saat kayması payıyla denetlenir. `nonce` madde 1.2'ye göre denetlenir.
   - Google'da e-posta yalnız `email_verified = true` ise iletişim adresi olarak alınır.
   Doğrulama tek bir modülde yapılır (`api::auth::oidc`). Sahte sağlayıcıyla (T2.11) her red durumunun testi yazılır: yanlış imza, `kid`, `iss`, `aud`, süre, nonce ve tekrar kullanılmış nonce.
4. **Kimlik:** kullanıcı `(provider, subject)` ile bulunur (ADR-0007 `user_identities`). E-posta kimlik değildir: Apple adresi gizleyebilir, kullanıcı adresini değiştirebilir.
5. **Apple yenileme belirteci:** hesap silinirken Apple belirteci iptal edilmeli (plan › Apple kuralları). Bunun için ilk Apple girişinde API yetkilendirme kodunu Apple'la değiştirir. Apple'a kimliğini Apple anahtarıyla imzalanmış istemci sırrıyla (JWT) kanıtlar. Dönen yenileme belirteci zarf şifrelemeyle saklanır: bulutta KMS anahtarı, yerelde sabit geliştirme anahtarı. Yeri `user_identities.provider_refresh_token_enc` (ADR-0007'ye eklendi).
6. **İlk giriş:**
   - Kimlik bilinmiyorsa kullanıcı ve kimlik satırı aynı işlemde oluşur. Ardından istemci kurulum ekranına geçer (mod, bölge, ekipman); çalışma alanı orada açılır.
   - Doğrulanmış e-posta başka bir kullanıcının kimliğinde ya da iletişim adresinde varsa hesaplar kendiliğinden **birleştirilmez**. Yanıt `account_hint` taşır: "Bu e-postayla Apple ile giriş yapılmış bir hesap var." İstemci "o yöntemle gir, sonra Ayarlar › Güvenlik'ten bağla" yolunu gösterir. Böylece e-postayı ele geçiren biri hesabı ele geçiremez.

### 2. E-posta bağlantısı

1. **İstek:** `POST /v1/auth/email/start` her zaman `202` döner. Adresin kayıtlı olup olmadığı sızmaz.
   - Belirteç 32 rastgele bayttır. Veritabanına yalnız SHA-256 özeti yazılır (`email_login_tokens`).
   - 15 dakika geçerli, tek kullanımlık.
   - Bağlantı iOS'ta evrensel bağlantıyla uygulamayı, değilse web'i açar.
2. **Doğrulama:** `POST /v1/auth/email/verify` belirteci tüketir. Belirteç bağlantıdan otomatik doğrulanmaz, ayrı bir istekle doğrulanır. Böylece e-posta tarayıcılarının bağlantıyı önceden açması belirteci harcamaz.
3. **Hız sınırı:** adres başına saatte 5, IP başına saatte 20 istek (doğrulanacak).
4. **E-postanın içeriği:** isteğin geldiği cihaz türü ve yaklaşık konum (ülke) yazılır. Yazı "Bu isteği sen yapmadıysan yok say" der.
5. **Kimlik:** e-posta kimliğinin `subject`'i normalize edilmiş adrestir (küçük harf, Unicode NFC).

### 3. Oturum

1. **Biçim:** oturum belirteci opaktır, 32 rastgele bayttır. JWT kullanılmaz, çünkü oturum anında kapatılabilmeli ("her yerden çıkış", destekten oturum kapatma).
2. **Saklama:** veritabanında yalnız SHA-256 özeti tutulur (`sessions`). Redis, oturum kaydını 5 dakika önbellekte tutar. Oturum kapatılınca önbellek kaydı da silinir. Redis yoksa doğrudan veritabanına bakılır.
3. **Süreler** (doğrulanacak):

   | İstemci | İşlemsizlik | En uzun |
   |---|---|---|
   | iOS | 30 gün | 90 gün |
   | Web, "bu bilgisayarı hatırla" açık | 7 gün | 30 gün |
   | Web, kapalı (paylaşılan bilgisayar) | tarayıcı oturumu ve 12 saat | 12 saat |
   | Ortak cihazda kullanıcı oturumu | işlemsizlik kilidi (varsayılan 5 dk) | 12 saat |

   `last_activity` en çok dakikada bir güncellenir. En uzun süre dolunca kullanıcı yeniden girer.
4. **Yenileme:** giriş, ikinci adım (madde 5) ve hesap bağlama sonrasında yeni belirteç verilir, eskisi kapatılır. Böylece oturum sabitleme saldırısı işlemez.
5. **Web taşıması:**
   - Çerez `__Host-session`: `HttpOnly; Secure; SameSite=Lax; Path=/`.
   - CSRF:
     - Çerezle gelen her yazma isteği `X-CSRF-Token` başlığını taşır. Değer oturuma bağlı bir HMAC'tir ve `GET /v1/me/session` ile alınır.
     - `Origin` başlığı web adresinin izin listesinde olmalıdır.
   - CORS: yalnız web adresine, kimlik bilgisiyle açık.
   - İçerik Güvenliği Politikası (CSP) web işinde (Aşama 3) yazılır.
6. **iOS taşıması:** `Authorization: Bearer <belirteç>`; belirteç Keychain'de tutulur (`kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`). Bearer'la gelen istekte CSRF denetimi yoktur. Aynı istekte hem çerez hem Bearer varsa istek reddedilir.
7. **Kullanıcıya açık işlemler:**
   - Oturum listesi (cihaz adı, istemci, son etkinlik, yaklaşık konum).
   - Tek oturumu kapatma.
   - Her yerden çıkış.
   Üçü de `/v1/me/sessions` altında, üçü de denetim kaydına yazılır.

### 4. Hesap bağlama ve giriş yöntemleri

1. **Bağlama:** `POST /v1/me/identities` yeni sağlayıcının taze belirtecini alır (madde 1 akışıyla). Kimlik zaten başka bir kullanıcıya bağlıysa `409 conflict` (`identity_in_use`) döner. Hesap birleştirme ilk sürümde yoktur.
2. **Kaldırma:** son giriş yöntemi kaldırılamaz. Kaldırma yeni kimlik doğrulama ister: son 10 dakika içinde giriş ya da ikinci adım.

### 5. Passkey ve ikinci adım

1. **Kütüphane ve kayıt:** WebAuthn için `webauthn-rs` kullanılır.
   - RP kimliği web ve iOS'un paylaştığı alan adıdır. iOS bu alanı ilişkili alan (`webcredentials:`) olarak tanır.
   - Kullanıcı doğrulaması (`userVerification = required`) zorunludur; Face ID, Touch ID veya cihaz kodu.
   - Kayıtlar `passkeys` tablosundadır.
2. **Kullanım:** ilk sürümde passkey ikinci adımdır, tek başına giriş yöntemi değildir. Passkey ile doğrudan giriş sonra eklenebilir; tablo bunu taşır.
3. **İkinci adım (step-up):** başarılı passkey doğrulaması oturuma `step_up_at` zamanını yazar ve 15 dakika geçerlidir. İkinci adım isteyen işlemler:
   - kullanıcı, rol ve izin değişiklikleri;
   - davet gönderme;
   - abonelik;
   - çalışma alanı ve hesap silme;
   - cihaz anahtarı yenileme;
   - destek erişim izni verme;
   - giriş yöntemi kaldırma;
   - tam dışa aktarma.
   İkinci adım yoksa veya süresi dolmuşsa yanıt `403 step_up_required` olur (ADR-0008).
4. **İşletme yöneticisi:** `users.manage` iznini taşıyan üyenin passkey'i olmalıdır. Passkey kaydetmeden bu izinli işlemler `step_up_required` (alt kod `passkey_enrollment_required`) döner. Kurulum sihirbazı ilk yöneticiye passkey kaydettirir.
5. **Ev modu:** passkey isteğe bağlıdır. İkinci adım isteyen işlem passkey varsa passkey'le, yoksa yeniden girişle (son 10 dakika) yapılır.

### 6. Davet

1. **Belirteç:** 32 bayt, yalnız özeti saklanır (`invitations`), 7 gün geçerli, tek kullanımlık. Davet çalışma alanına, role ve kapsama bağlıdır; e-postaya bağlı değildir (plan › Kimlik ve giriş).
2. **Kabul:** kabul eden kişi istediği yöntemle girer. Kabulde üye satırı ve rol ataması aynı işlemde oluşur.
3. **Yönetim:** gönderen ya da yönetici daveti iptal edebilir. Süresi dolmuş veya kullanılmış davet `404` gibi görünür; istemci "davet geçersiz" ekranını gösterir.
4. **Ev modunda paylaşım:** paylaşım `recipe` veya `batch` kapsamlı bir davettir (ADR-0007 `role_assignments`).

### 7. Ortak cihaz

1. **Cihazı açma:** bir yönetici iPad'de girer, "ortak cihaz modu"nu açar (ikinci adım gerekir). Cihaz kendi anahtarını üretir ve Keychain'de saklar; sunucu yalnız özetini tutar (`shared_devices`). Bundan sonra cihaz yalnız ortak cihaz uçlarını çağırabilir, kendi başına hiçbir izni yoktur.
2. **Kullanıcı değiştirme:** çalışan yüz veya ad kutucuğunu seçer, 6 haneli PIN'ini girer. `POST /v1/shared-devices/{id}/switch` cihaz anahtarı ve PIN'le kısa ömürlü bir kullanıcı oturumu verir; oturum cihaza bağlıdır (`sessions.shared_device_id`).
   - PIN yalnız kayıtlı ortak cihazda geçerlidir.
   - PIN özeti Argon2id ile tutulur (`member_pins`).
   - Üst üste 5 hatalı denemede o üye 15 dakika kilitlenir. Cihaz başına dakikada 20 deneme sınırı vardır.
   - İşlemsizlik süresi sunucudadır. Süre dolunca oturum biter ve ekran kilitlenir.
3. **Face ID ortak cihazda kullanıcı ayırmaz.** iOS'ta Face ID cihazın sahibine kayıtlıdır; hangi çalışanın baktığını uygulamaya söylemez. Bu yüzden ortak cihazda kişiye özel giriş PIN'le yapılır. Face ID kişisel cihazda uygulama kilidi ve passkey için kalır. Plan › Ortak cihaz modu ve konsept buna göre güncellendi.
4. **Bağlantı yokken:** iPad'de indirilmiş demleme paketiyle çalışılır (plan › Ortak cihaz modu). PIN özetleri cihaza **indirilmez**: 6 haneli bir PIN'in özeti cihazdan çalınırsa dakikalar içinde kırılır.
   - Bağlantı yokken kullanıcı değiştirilirse cihaz seçilen üyeyi "doğrulanmamış" olarak kabul eder.
   - Girilen PIN, sunucunun açık anahtarıyla şifrelenip (HPKE, X25519) o üyenin kuyruktaki ölçümlerine eklenir. Cihaz PIN'i çözemez.
   - Bağlantı gelince sunucu PIN'i, üyeliği ve yetkiyi denetler. Tutmazsa o ölçümler reddedilir, partide "doğrulanamayan giriş" olarak işaretlenir ve yöneticiye bildirilir.
   - Deneme sınırı bu yolda da sayılır.

### 8. Yetki

1. **İzin listesi:** izinler kodda tek bir enum'dur (ADR-0007 madde 3). Kodlar ve plan › Yetki matrisi'ndeki karşılıkları:

   | İzin | Matristeki satır |
   |---|---|
   | `recipe.view` | Tarif görüntüle |
   | `recipe.edit` | Tarif düzenle (taslak) |
   | `recipe.approve` | Tarif onayla |
   | `batch.plan` | Parti planla, tank ata |
   | `batch.record` | Demleme kaydı gir, adım kapat |
   | `batch.correct_closed` | Kapanmış partide düzeltme |
   | `quality.record` | Kalite ölçümü ve tadım |
   | `release.approve` | Ambalaj için serbest bırak |
   | `stock.move` | Stok hareketi ve sayım |
   | `cost.view` | Maliyet ve fiyatları gör |
   | `reports.export` | Raporlar ve dışa aktarma |
   | `settings.equipment` | Ekipman ve profil ayarları |
   | `users.manage` | Kullanıcı, rol, yetki yönetimi |

   Matriste olmayan, bu kayıtta eklenen izinler:
   - `workspace.manage`: çalışma alanı ayarları ve silme;
   - `billing.manage`: abonelik;
   - `support.grant`: destek erişim izni;
   - `devices.manage`: cihaz eşleştirme ve anahtarlar.

   Hazır rollerin izinleri tohum verisidir (T2.13), plan › Yetki matrisi ile aynıdır. Yönetici rolü bütün izinleri taşır. Ev modunda sahip yönetici rolündedir. Arkadaş paylaşımında iki hazır rol vardır: düzenleyici (`recipe.view`, `recipe.edit`, `batch.record`) ve izleyici (`recipe.view`).
2. **Denetim yeri:** yetki yalnız API'de, tek modülde denetlenir (`api::authz`). Her uç gerektirdiği izni bildirir; bir test bütün yönlendiriciyi tarar ve izin bildirmeyen ucu düşürür. Giriş öncesi ve `me` uçları açıkça "izin gerekmez" diye işaretlenir. Veritabanı RLS'si yalnız çalışma alanını ayırır; izni bilmez (ADR-0007 madde 2). İki katman birbirinin yerine geçmez.
3. **Kapsam:**
   - Kapsamsız atama bütün çalışma alanında geçerlidir.
   - `recipe` kapsamı o tarifi ve sürümlerini kapsar.
   - `batch` kapsamı o partiyi; adımlarını, ölçümlerini ve paketini kapsar.
   - `vessel` ve `area` kapsamı, o tank veya alanda bulunan partilerin parti, kalite ve stok işlemlerini kapsar.
   - Liste uçları kapsamı SQL koşuluna çevirir; kapsam dışındaki kayıt listede görünmez, doğrudan istenirse `404` döner.
4. **Etkin izinler:** istek başında üyenin bütün atamalarından hesaplanır. Sonuç Redis'te `(üye, authz_version)` anahtarıyla önbelleğe alınır. Çalışma alanının `authz_version` değeri her rol, atama veya üyelik değişikliğinde artar, eski önbellek kendiliğinden geçersiz olur.
5. **Alan gizleme:** `cost.view` izni olmayana maliyet alanları yanıttan çıkarılır (ADR-0008 madde 2.5). Gizleme tek bir seri hale getirme katmanında, izin bağlamıyla yapılır; her uç ayrıca yazmaz.
6. **Korumalar** (plan › Ayarlarda yönetim):
   - Çalışma alanında en az bir etkin yönetici kalır.
   - Kimse kendi `users.manage` iznini kaldıramaz.
   - Rol ve atama değişiklikleri denetim kaydına yazılır.
   - Devre dışı bırakılan üyenin oturumları o çalışma alanı için hemen geçersiz olur: `authz_version` artar, SSE akışı kapanır.
7. **Abonelik:** yetki ayrıca aboneliğe (Entitlement) bakar. Deneme bitince ücretli özelliklerin yazma uçları `403 entitlement_required` döner. Okuma ve dışa aktarma hep açıktır (plan › Deneme süresi). Bu denetim de `api::authz` içindedir.

### 9. Destek ekibi kimliği

1. **Giriş:** destek paneli oauth2-proxy arkasındadır (ADR-0008 madde 10). Giriş şirketin Google Workspace hesabıyla yapılır.
   - Passkey veya güvenlik anahtarıyla ikinci adım Workspace yönetiminde zorunlu tutulur.
   - Vekil, Google kimlik belirtecini `Authorization` başlığıyla iletir.
2. **Doğrulama:** API belirteci madde 1.3'teki gibi doğrular, ayrıca `hd` alanının şirket alan adı olmasını ister. Belirteç `staff_users` ile eşlenir; etkin destek kullanıcısı değilse istek `403` döner.
3. **Müşteri içeriğine erişim:** yalnız geçerli bir `support_grants` satırıyla açılır. Bağlam `actor_kind = support` ve salt okunur işlemdir (ADR-0007 madde 2.3). Acil erişim iki ayrı `support_admin` onayı ister.

### 10. Servisler arası güvenlik

1. **mTLS:** Kubernetes'te Istio ambient modu kullanılır.
   - **Neden Istio:** açık kaynak (Apache 2.0), CNCF'den mezun, yan konteyner istemiyor. GKE, EKS, AKS ve kind'de aynı tanımlarla çalışıyor.
   - Uygulama kodu mTLS bilmez.
   - `PeerAuthentication` STRICT modda. Kimlikler SPIFFE biçiminde.
   - `AuthorizationPolicy`: engine-service'e yalnız api, `/internal` portuna yalnız oauth2-proxy erişir. Kubernetes ağ politikaları ikinci katman olarak kalır.
   - **Seçilmeyenler:** uygulama içinde rustls ve cert-manager ile mTLS her serviste sertifika yenileme kodu gerektirirdi. Linkerd'in kararlı sürümleri ticari dağıtıma bağlı.
   - Yerel compose'da mTLS yoktur. kind testi (T2.32) düz metin isteğin reddedildiğini doğrular.
2. **Veri servisleri:**
   - PostgreSQL ve Redis bağlantıları TLS'lidir (`sslmode=verify-full`; Memorystore'da aktarım şifrelemesi).
   - NATS'ta hesap ve izin vardır: api ve ingest yalnız kendi konularına yayımlar, worker tüketir. Kimlik NKey ile kanıtlanır.
3. **Dış olay imzaları:** dış olay alımında (ingest) üç tür imza denetlenir.
   - GitHub `X-Hub-Signature-256` (HMAC-SHA256, sabit zamanlı karşılaştırma).
   - App Store sunucu bildirimleri (JWS, Apple kök sertifika zinciri).
   - Cihaz adresi anahtarı (özetle eşleşme).
   İmzası tutmayan istek kuyruğa yazılmaz.

### 11. Sırlar ve yapılandırma

1. **Yapılandırma:** yalnız ortam değişkenlerinden okunur (12 faktör, madde 3). Sırlar ya ortam değişkeninden ya da `*_FILE` değişkeninin gösterdiği dosyadan okunur; dosya Kubernetes'te bağlanan sır hacmidir. `common::config` bu kuralı tek yerde uygular.
2. **Sırların yeri:**
   - Kümede sırlar bulutun sır yöneticisindedir (Google Cloud'da Secret Manager). Kubernetes'e External Secrets Operator ile gelir.
   - Yerelde `make secrets` geliştirme sırlarını üretir: depoya girmeyen `altyapi/compose/.secrets/`.
   - Depoda sır yoktur; CI'da gizli anahtar taraması çalışır.
3. **Anahtar listesi:**
   - CSRF HMAC anahtarı;
   - demleme paketi imza anahtarı (Ed25519);
   - Apple giriş anahtarı (`.p8`);
   - Google istemci sırrı;
   - GitHub App özel anahtarı ve webhook sırrı;
   - APNs anahtarı;
   - e-posta sağlayıcısı anahtarı;
   - saklanan sağlayıcı belirteçleri için KMS anahtarı;
   - ortak cihazın çevrimdışı PIN'i şifrelediği X25519 anahtar çifti (cihaza yalnız açık anahtar gider);
   - NATS NKey'leri.
4. **Anahtar yenileme:** imza anahtarları kimlik (`kid`) taşır. Doğrulama birden çok geçerli anahtarı kabul eder; böylece anahtar kesintisiz yenilenir.

### 12. Tehdit listesi (STRIDE, kısa)

| Tehdit | Örnek | Önlem |
|---|---|---|
| Kimlik taklidi | Çalınmış veya tekrar kullanılan kimlik belirteci | Tek kullanımlık nonce, `aud`, `iss`, süre; opak oturum ve özetle saklama; oturum yenileme |
| Kimlik taklidi | E-postayla hesap ele geçirme | Otomatik birleştirme yok (madde 1.6); bağlantı 15 dk, tek kullanımlık |
| Kimlik taklidi | Ortak cihazda başkasının PIN'i | Deneme sınırı, PIN yalnız kayıtlı cihazda, işlemsizlik kilidi |
| Kurcalama | Demleme paketinin veya kuyruktaki ölçümün değiştirilmesi | Ed25519 imzası ve süre; ölçümün paket ve partiyle eşleşmesi (T2.22) |
| Kurcalama | Denetim kaydının değiştirilmesi | Yalnız ekleme hakkı, özet zinciri (ADR-0007 madde 5) |
| İnkâr | "Ben yapmadım" | Her işlem gerçek kullanıcıya yazılır (ortak cihazda dahil); denetim kaydı |
| Bilgi sızması | Başka çalışma alanının verisi | RLS, bileşik anahtar, `404`; SSE olayı içerik taşımaz |
| Bilgi sızması | Yetkisize maliyet alanı | Seri hale getirmede alan gizleme; olayda içerik yok |
| Bilgi sızması | Kayıtlı e-posta tespiti | `email/start` her zaman `202` |
| Hizmet engelleme | Giriş ve PIN denemesi bombardımanı | Hız sınırları (ADR-0008 madde 11) |
| Yetki yükseltme | İzinsiz uç | Her uç izin bildirir, test denetler; yönetici işlemlerinde passkey |
| Yetki yükseltme | Destek ekibinin içeriğe erişimi | Süreli izin, salt okunur işlem, iki onaylı acil erişim, müşteriye görünen geçmiş |
| Yetki yükseltme | Servisler arası doğrudan erişim | Istio mTLS ve `AuthorizationPolicy`, ağ politikası |

## Uygulama notları (T2.11, 13.11.2026)

Karar değişmedi; yazarken netleşenler:
- **Oturum ucu:** `GET|DELETE /v1/me/session` (mevcut oturum, çıkış), `GET|DELETE /v1/me/sessions` (liste, her yerden çıkış), `DELETE /v1/me/sessions/{id}`. Web girişi çerezi `Set-Cookie` ile verir, gövdede belirteç yoktur; CSRF belirteci giriş yanıtında ve `GET /v1/me/session`'dadır. Çerezle gelen güvenli olmayan istek `Origin` (izin listesi) ve `X-CSRF-Token` ister, ikisi de `403 forbidden` (`origin_not_allowed`, `csrf_invalid`).
- **Hesap ipucu** yalnız yeni hesap açılırken döner ve sağlayıcı adlarını taşır (`account_hint.providers`); kullanıcı kimliği sızmaz. Giriş yine de yeni hesapla açılır.
- **E-posta bağlantısı:** hız sınırı adres başına saatte 5, IP başına saatte 20 (aşılırsa `429`); SMTP hatası yanıtı değiştirmez, yalnız günlüğe yazılır. Adres geçersizse `422` (kayıt durumu sızmaz).
- **Nonce:** `POST /v1/auth/nonce` ham değeri verir; Apple belirtecindeki `nonce` bunun SHA-256 hex'idir, Google'da kendisidir. Nonce, belirteç imzası ve alanları geçtikten sonra tüketilir; geçmeyen belirteç nonce'u yakmaz, geçen ama nonce'u tutmayan yakar.
- **JWKS yenileme aralığı:** bilinmeyen `kid` yenilemeyi tetikler, ama yenilemeler arası en az 30 sn (`JWKS_MIN_REFRESH_SECS`); sahte `kid` yağmuru sağlayıcıya istek yağdıramaz. Yenileme başarısızsa `503`.
- **Giriş yöntemi kaldırma:** son 10 dakikada giriş ya da ikinci adım (`step_up_at`) şartı; passkey gelene kadar (T2.12) `step_up_at` boştur, yani eski oturumda kaldırma `403 step_up_required` verir.
- **İlk çalışma alanı:** `POST /v1/workspaces` (Idempotency-Key şart) çalışma alanını, üyeyi, tüm izinleri taşıyan "Administrator" rolünü ve atamayı tek işlemde açar. Hazır rollerin geri kalanı ve izin önbelleği T2.13'tedir; `membership` her istekte iki sorgu yapar.
- **Yapılmayanlar:** web yönlendirmeli kod akışı (`/v1/auth/{provider}/start` ve `callback`) ve CORS Aşama 3'te (web istemcisi) gelir; Apple yenileme belirteci değişimi (madde 1.5) hesap silmeyle (T2.31); oturum işlemlerinin denetim kaydına yazılması T2.14'te (şimdilik günlük).

## Uygulama notları (T2.13, 14.11.2026)

Karar değişmedi; yazarken netleşenler:
- **Yer:** izin listesi, hazır roller, kapsam ve alan gizleme `api::authz`'de; uçlar `api::auth::access`'te. `Permission::ALL` ile veritabanı CHECK'i ve plan matrisi testle karşılaştırılır (tablo güdümlü, rol × izin).
- **Tohum:** çalışma alanı açılırken `seed_roles`. İşletme: Administrator, Brewmaster, Operator, Quality, Warehouse, Viewer (matris). Ev: Administrator, Editor (`share_editor`), Viewer (`share_viewer`). Yeni üç izin (`workspace.manage`, `billing.manage`, `support.grant`, `devices.manage`) yalnız yöneticide.
- **Uçlar** (hepsi `users.manage`, kabul hariç): `GET|POST /roles`, `POST /roles/{id}/copy`, `PATCH|DELETE /roles/{id}` (If-Match şart), `GET /members`, `PATCH|DELETE /members/{id}`, `POST /members/{id}/assignments`, `DELETE /members/{id}/assignments/{aid}`, `GET|POST /invitations`, `DELETE /invitations/{id}`, `POST /shares` (Ev), `POST /v1/invitations/accept` (oturum açmış herkes, çalışma alanı yolu yok).
- **Etkin izinler:** kapsamsız atamalar `Membership.permissions`, kapsamlılar `Membership.scoped`. Redis anahtarı `authz:{ws}:{üye}:{authz_version}` (300 sn); her değişiklik `authz_version` artırır, bu yüzden değişiklik sonraki istekte görünür. Redis yoksa yalnız sorgu yapılır.
- **Kapsam kuralı:** `RequestContext::require_for(izin, Target)`: izin tüm çalışma alanında ya da kaydı kapsayan kapsamlı atamada varsa geçer. Aksi halde kayıt kullanıcıya görünüyorsa (çalışma alanında `recipe.view` ya da kaydı kapsayan herhangi bir atama) `403`, görünmüyorsa `404`. Liste uçları `scope_filter(izin)` ile SQL koşulu kurar. Kapsamlı atama yalnız çalışma alanı düzeyi izin taşımayan rolle yapılır (`users.manage`, `workspace.manage`, `billing.manage`, `support.grant`, `devices.manage` varsa `422 role_not_scopable`).
- **Korumalar:** her değişiklik çalışma alanı satırını kilitleyen işlemde yürür; değişiklikten sonra `last_admin` (etkin `users.manage` sahibi kalmadı) ve `self_lockout` (çağıran kendi iznini ya da üyeliğini kaybediyor) denetlenir, ikisi `409 conflict`. Yönetici sayılan: kapsamsız `users.manage` taşıyan etkin üye.
- **Davet:** e-posta adresi saklanmaz, yalnız postayla gider (bağlantı `{WEB_BASE_URL}/invite?token=...`). Kabul tek `UPDATE ... WHERE accepted_at IS NULL AND expires_at > now()` ile tek kullanımlıktır; çakışan iki istekten biri `404` alır. Devre dışı üye davetle geri dönemez (`409 member_disabled`, davet yanmaz); çıkarılmış üye döner.
- **Ev paylaşımı:** `POST /shares {kind: recipe|batch, id, access: editor|viewer, email}` kapsamlı davettir; yalnız Ev çalışma alanında (`409 home_mode_only`). Paylaşımı kaldırmak atamayı silmektir.
- **Alan gizleme:** `RequestContext::redact` / `redacted_json`: anahtarın `_` ile ayrılmış parçalarından biri `cost` veya `price` ise (`unit_cost`, `price_currency`, ...) `cost.view` yokken alan yanıttan çıkar. `reject_cost_writes` aynı alanları yazmaya çalışana `403` verir. Kaynak uçları bu yardımcıları kullanır (şimdilik yalnız test uçlarında).
- **Denetim:** rol, izin kümesi, atama, davet ve üyelik değişiklikleri `append_audit` ile aynı işlemde yazılır; ad ve e-posta için yalnız `changed_field` ("değişti" işareti; değerin özeti yazılmaz, F2.1 bulgu 9).
- **Yapılmayanlar:** abonelik denetimi (madde 7, T2.2x), üye adının listede görünmesi (`users` satırı RLS'ye göre yalnız kendisine açık), kapsamın dosya uçlarına (`files.rs`) uygulanması (kaynak uçlarıyla birlikte).
## Uygulama notları (T2.12, 14.11.2026)

Karar değişmedi; yazarken netleşenler:
- **Kütüphane:** `webauthn-rs` 0.6.2 (sürüm sabit), yazılımsal doğrulayıcı testte `webauthn-authenticator-rs` `SoftPasskey`. RP kimliği ve kökenler `WEBAUTHN_RP_ID`, `WEBAUTHN_RP_NAME`, `WEBAUTHN_ORIGINS` ile gelir.
- **Saklama:** `passkeys.public_key` kütüphanenin serileştirilmiş `Passkey` kaydını (COSE anahtarı ve bayraklar) taşır; `credential_id` ve `sign_count` ayrı sütundur. Tören durumu Redis'te değil `webauthn_challenges` tablosunda (tek kullanımlık, 5 dk; göç 0090).
- **Uçlar:** `GET /v1/me/passkeys`, `POST /v1/me/passkeys/registration/{start,finish}`, `DELETE /v1/me/passkeys/{id}`, `POST /v1/me/step-up/{start,finish}`. Kayıt ve silme ikinci adım ister (passkey yoksa son 10 dakikada giriş); kayıt bitince ve ikinci adımda oturum belirteci yenilenir ve `step_up_at` yazılır. Ortak cihaz oturumu ikinci adım yapamaz.
- **İkinci adım denetimi:** `auth::step_up::require_step_up`. Alt kodlar `errors[0].code`: `passkey_enrollment_required` (`users.manage` izni olan, passkey'siz üye), `passkey_required` (passkey var, `step_up_at` 15 dakikadan eski), `reauthentication_required` (Ev modu, passkey yok, giriş 10 dakikadan eski). Korumalı işlem listesi `ProtectedAction` enum'unda; izin listesine (T2.13) dokunmaz. Şimdilik uygulayanlar: ortak cihaz açma, anahtar yenileme, passkey kaydı ve silme. Kullanıcı, davet, abonelik, silme, destek erişimi ve tam dışa aktarma ilgili işler gelince aynı işlevi çağırır.
- **Ortak cihaz uçları:** yönetici tarafı `/v1/workspaces/{ws}/shared-devices` (oluştur, listele, `DELETE` iptal, `rotate-key`), `devices.manage` ister; açma ve anahtar yenileme ikinci adım ister. Cihaz anahtarını cihaz üretir (32 ile 128 URL-güvenli karakter), sunucu yalnız SHA-256 özetini tutar. PIN: `PUT /v1/workspaces/{ws}/me/pin` (üye kendi PIN'ini koyar; kilidi de açar), `DELETE .../members/{member}/pin` (yönetici siler). Cihaz tarafı (`X-Device-Key` başlığı, oturumsuz): `GET /v1/shared-devices/{id}/members` (PIN'i olan üyeler), `POST .../switch`, `POST .../lock`. Anahtar tanınmazsa `401` (`device_key_invalid`), tanınır ama yol başka cihazı gösterirse `404`.
- **PIN kuralları:** 6 rakam, Argon2id (`argon2` 0.6.0 varsayılanları). Üst üste 5 hata 15 dakika kilit (`429`, `pin_locked`, `Retry-After`); kilitliyken doğru PIN de reddedilir. Sayaç `member_pins` satırı kilitlenerek (`FOR UPDATE`) tutulur, paralel denemeler sayaçtan kaçamaz. Cihaz başına dakikada 20 deneme Redis'tedir (Redis yoksa açık kalır; üye kilidi veritabanındadır, o yine korur). Üye ya da PIN yoksa da sahte bir doğrulama yapılır (süre sızmaz).
- **Oturum:** `sessions.idle_timeout_secs` cihazın işlemsizlik süresidir (`find_session_by_token_hash` bu sütunu da döndürür). `last_activity_at` dakikada bir yazıldığı için kilit en çok bir dakika geç tutabilir. Yeni `switch`, `lock`, iptal ve anahtar yenileme cihazın bütün oturumlarını `close_shared_device_sessions` işleviyle kapatır (oturumlar başka kullanıcıya ait olduğu için satır güvenliği doğrudan güncellemeye izin vermez). Açık `device_sessions` kaydının `ended_at` değeri bu yollarda yazılır; işlemsizlikten biten oturumun kaydı bir sonraki değişimde kapanır.
- **Yapılmayanlar:** çevrimdışı PIN (HPKE, madde 7.4) ve "doğrulanamayan giriş" işaretleme çevrimdışı ölçüm kuyruğuyla (Aşama 4) gelir; passkey ile doğrudan giriş sonraki sürüm; denetim kaydı T2.14.

## Sonuçlar

- ADR-0007'ye iki ek: `auth_requests` tablosu (nonce, durum, PKCE) ve `user_identities.provider_refresh_token_enc`.
- T2.11 giriş ve oturumu, T2.12 passkey ve ortak cihazı, T2.13 yetkiyi, T2.32 Istio ve sırları uygular. Her iş bu kayıttaki red durumlarının testini getirir.
- Plan › Ortak cihaz modu ve konsept: ortak cihazda kullanıcı değiştirme PIN'le (madde 7.3).
- Açık: oturum süreleri ve e-posta hız sınırları doğrulanacak. Passkey ile doğrudan giriş sonraki sürümde. Hesap birleştirme yok.

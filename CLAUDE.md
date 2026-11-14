# Demleme uygulaması: çalışma kuralları

Bu depo bira demleme uygulamasının (Ev ve İşletme modu) planı, tasarımı ve kodudur. Kullanıcıyla Türkçe konuş.

## Proje durumu ve kararlar

- **Geliştirme aşamasındayız** (11 Kasım 2026'dan). İş `gelistirme/ISLER.md`'deki sırayla, adım adım yapılır. Görüşmeler ve altın test kodun değil Aşama 5'in (beta) kapısıdır.
- **Kapsam dışı:** ERP (satış, sipariş, sevkiyat, fatura, muhasebe, keg takibi), topluluk ve herkese açık tarif paylaşımı, Android, ayrı Mac/Windows uygulaması.
- **Mimari:** SwiftUI (iPad, iPhone; Apple Watch ileride opsiyonel), React web, Rust hesap motoru yalnız sunucuda, ayrı servis olarak (istemciye hesap kodu gitmez). Altı katman (istemci, kenar, uygulama, hesap, veri, platform); Kubernetes üzerinde, açık standartlarla, bulut sağlayıcıdan bağımsız (plan › Platformlar ve mimari). Sunucu kodunun tamamı Rust; ilk bulut Google Cloud; giriş Apple ve Google ile (yedek e-posta bağlantısı), ayrı kayıt ve şifre yok.
- **Kodu Claude yazar:** derleme, tip, lint, test vektörü ve sözleşme testinden geçmeyen kod ana dala girmez. Geçici mimari yok: Kubernetes, ayrı hesap motoru servisi, Redis ve NATS ilk sürümden. Bütün sistem yerelde tek komutla (Docker Compose, kind) çalışır ve Claude her değişikliği orada uçtan uca test eder. iOS'ta macOS CI ve gerçek cihaz testi ürün sahibinde (plan › Geliştirme düzeni). İnternetli çalışma; yalnız demleme günü indirilen paketle yerelde çalışır. Çevrimdışı senkron yok.
- **Hedef pazar:** ABD ve İngiltere öncelikli.
- **Lisans konusu kullanıcıda;** açma.
- **Güncellemeler için izin isteme:** plan, konsept ve tasarım sistemine yapılacak değişiklikleri doğrudan yap, sonra ne değiştiğini söyle. Kapsam veya ürün kararı gerektiren konuları ise sor.

## "Devam" dendiğinde

1. `gelistirme/DURUM.md`'yi oku; `gelistirme/ISLER.md`'den yalnız içinde bulunulan aşamayı oku. Planın tamamını okuma.
2. Bağlı olduğu işler bitmiş, kimi `Claude`, durumu `todo` olan ilk işi al. Kullanıcı başka bir iş ya da sayı söylediyse ona göre yap (ör. "devam, 3 iş").
3. İşi ISLER.md'deki **Model** sütunundaki modelle yap: Opus işini ana oturumda; Sonnet, Haiku ve Fable işini Agent aracıyla alt ajana ver (`model` parametresiyle). Alt ajana okunacak dosyaları, bitti ölçütünü ve "testleri çalıştır, en fazla 20 satır rapor ver" talimatını yaz; iş tarifi STE ile yazılır (aşağıda › Yazım standardı). Birbirine bağlı olmayan işler paralel alt ajanlarla yürüyebilir.
4. Bitti ölçütünü kendin doğrula (testleri çalıştır, farkı incele). Geçmezse işi alt ajana geri gönder ya da düzelt.
5. ISLER.md'de durumu `done (YYYY-AA-GG)` yap; DURUM.md'de sıradaki işi, engelleri ve son kararları STE ile güncelle; commit mesajını iş numarasıyla başlat (`T0.2: ...`), commit at ve `main`'e gönder.
   Commit ve push'u Claude onay istemeden yönetir; oturum dalına her işten sonra, `main`'e testler yeşilken ve iş tutarlı bir noktadayken (hızlı ileri, geçmiş yeniden yazılmaz) gönderir (Oğuz, 11.11.2026).
6. Kullanıcıya en fazla beş satırla bildir: ne bitti, test sonucu, sıradaki iş. Kullanıcıdan karar gerekirse sor ve dur.
7. Aşama kapısı kapanınca kullanıcıya yeni oturum açmasını öner.

## Token verimliliği (Oğuz, 14.11.2026)

- **Oturum kısa tutulur:** her aşamada ve ana oturumun bağlamı büyüyünce (yaklaşık 5–6 iş birleştirildikten sonra) iş tutarlı bir noktada kesilir, DURUM.md güncellenir ve kullanıcıya yeni oturum önerilir. Kayıt DURUM.md'dedir, sohbet geçmişine güvenilmez.
- **Bağlı işler tek ajana, sırayla:** aynı modüle dokunan, birbirine bağlı işler (ör. API alan uçları T2.19→T2.21→T2.22) tek alt ajana zincir olarak verilir; ajan her işten sonra kendi commit'ini atar. Paralel ajan yalnız birbirinden ayrık işlerde (farklı crate, göç ile istemci, inceleme) kullanılır; aynı anda en çok 2–3 ajan (5 saatlik limit).
- **Birleştirmeyi ajan yapar:** worktree ajanı işi bitirince `git fetch origin main && git rebase origin/main` yapar, çakışmayı kendisi çözer, testleri yeniden alır. Ana oturum çakışma çözmez; yalnız farkı inceler, testi doğrular, gönderir.
- **Okuma dar tutulur:** alt ajan talimatında belgenin tamamı değil ilgili bölümü (başlık adıyla) ve crate'in CLAUDE.md'si gösterilir. Uzun ADR'ler ve planın tamamı okutulmaz.
- **Çıktı süzülür:** test ve derleme çıktısı `grep -E "^test result|FAILED|panicked"` ya da `tail` ile okunur; tam workspace testi, clippy ve sqlx denetimi işin sonunda bir kez. compose `make up` ve kind CI'dadır (`altyapi.yml`); gönderimden sonra CI sonucu `actions_list` ile denetlenir.
- **Disk:** worktree ajanları kendi `sunucu/target-yerel` dizinini kullanır (`CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`), bitince siler. Ortak `CARGO_TARGET_DIR` kullanılmaz. Commit'ten önce `git status` ile derleme dizini girmediği denetlenir; `git add -A` yerine yol verilir.
- **Model:** ana oturumda rutin koordinasyon (birleştirme, takip) için Sonnet önerilir; karar, inceleme ve spesifikasyon işinde Opus. Haiku yalnız mekanik işlerde (yeniden adlandırma, liste); kod ve göç işinde kullanılmaz.

## Yazım standardı: ASD-STE100 (Oğuz, 14.11.2026)

İş listesi ve notlar Basitleştirilmiş Teknik İngilizce ile yazılır (ASD-STE100, Issue 9). Amaç: her oturum ve her alt ajan aynı metni tek anlamla okusun.

- **Kapsam (STE):** `gelistirme/ISLER.md` (tanım, bitti ölçütü, durum), `gelistirme/DURUM.md`, crate ve istemci `CLAUDE.md` dosyaları (`sunucu/`, `web/`, `ios/`), alt ajan iş tarifleri, koddaki `TODO` notları.
- **Kapsam dışı (Türkçe kalır):** kullanıcıyla konuşma ve ona bildirim, bu kök `CLAUDE.md`, plan, konsept, tasarım sistemi, `formuller/`, ADR'ler, `gelistirme/raporlar/`, commit mesajları. Ürün metni (TR/EN katalog) STE değildir.
- **Eski kayıt:** kapanmış aşamaların ve bitmiş işlerin Türkçe tanımları yeniden yazılmaz. Bir satır değişince o satır STE'ye çevrilir.
- **Durum etiketleri:** `todo` · `active` · `done (YYYY-MM-DD)` · `blocked (reason)`. Tarih ISO biçimindedir. Kim: `Claude` · `Oğuz`.
- **Teknik adlar değişmez:** kod adları, dosya yolları, tablo ve varlık adları (`RecipeVersion`), ürün ve servis adları, iş numaraları (T2.9, O6), komutlar. Türkçe belge başlığına başvuru Türkçe kalır: `plan › Veri modeli`.
- **Doğrulanmamış bilgi:** "doğrulanacak" yerine `to be verified` yazılır.

**Kurallar (Claude'un uyguladığı özet; tam sözlük ASD'nin lisanslı belgesindedir):**

1. Onaylı sözcüğü yalnız onaylı anlamıyla ve sözcük türüyle kullan. Bir sözcük bir anlam; bir anlam bir sözcük. Aynı şeye her yerde aynı adı ver.
2. Teknik ad ve teknik fiil serbesttir. Proje teknik fiilleri: build, compile, commit, push, merge, rebase, deploy, migrate, seed, sign, verify, encrypt, cache, parse, export, import, log. Yeni fiil gerekirse bu listeye eklenir.
3. Deyimsel fiil kullanma (`set up` değil `install`/`configure`, `find out` değil `find`).
4. Ad öbeği en çok üç sözcük. Uzunsa `of`, `for`, `in` ile aç (`session token hash cache` değil `the cache for session token hashes`).
5. `the`, `a`, `this` gibi belirleyicileri atma; telgraf dili yok. İstisna: tablo hücresindeki ad öbeği, başlık ve liste maddesi başlığı.
6. Fiil zamanı: geniş zaman, geçmiş zaman, gelecek zaman (`will`). Geçmiş ortaç yalnız sıfat olarak ya da tanım metninde edilgen yapıda.
7. Yönergede etken çatı ve emir kipi: `Run the tests.` Edilgen yalnız tanım metninde, yapanın önemi yoksa.
8. `-ing` fiil biçimi kullanma (teknik ad değilse). `while testing` değil `when you test`.
9. Yönerge cümlesi en çok 20, tanım cümlesi en çok 25 sözcük. Bir cümlede bir yönerge (aynı anda yapılan iki iş hariç).
10. Kısaltma biçimi yok (`do not`, `cannot`). Gerekçe ve koşul ayrı kısa cümlede: önce yönerge, sonra neden.
11. Paragraf bir konu, en çok altı cümle, konu cümlesi başta. Üç ve daha çok adım ya da koşul dikey listeyle yazılır.
12. Uyarı (engel, yasak) önce gelir ve açık yazılır: `Do not use a shared CARGO_TARGET_DIR.` Ardından tek cümle neden.

Örnek. Kötü: `Paralel işlerde api/src/lib.rs'e dokunmamaya dikkat edilmeli.` İyi: `Do not change api/src/lib.rs in parallel tasks. Two tasks that change it cause a merge conflict.`

## Dosyalar ve bağlantılar

- Geliştirme takibi: `gelistirme/ISLER.md` (iş listesi, model dağılımı), `gelistirme/DURUM.md` (neredeyiz), `adr/` (karar kayıtları).

- Plan (Claude Docs, asıl kaynak): https://claude.ai/artifact/1pUDt9xJQHdj2WxhMphDQy. `plan/demleme-plani.md` yalnız bir kopyadır; plan değişince yeniden dışa aktarılır. Kopya doğrudan düzenlenmez (başka ajanlar dahil); kopyada asıl kaynakta olmayan bir değişiklik görülürse önce asıl kaynağa taşınır.
- Arayüz konsepti: `arayuz/demleme-arayuz-konsepti.html` (asıl kaynak bu dosya), yayını https://claude.ai/artifact/BCEHj3wb6xj7eShvjTxKn6
- Tasarım sistemi: https://claude.ai/artifact/ATVvgzfCftjr9YzVjhSe28 (asıl kaynak); kopyası `tasarim-sistemi/` (önce `tasarim-sistemi/README.md` okunur). Biri değişince öbürü de güncellenir.
- Formül spesifikasyonları ve test vektörleri: `formuller/` (dizin: `formuller/00-OKU-BENI.md`)
- BeerSmith–Brewtarget karşılaştırması: `karsilastirma/`
- Brewtarget kaynak kodu depoda yok (üçüncü taraf kodu); gerekirse ayrıca klonlanır ve yalnız okunur.

## Tutarlılık

- Plan, konsept ve tasarım sistemi birlikte güncellenir: bir ekran veya karar değişince üçünde de karşılığını kontrol et (ekran listesi, mod ayrımı tablosu, Apple karşılıkları tablosu, ekran sayıları).
- Konsepti yayımlamadan önce bilgisayar (1280 px) ve telefon (400 px) genişliğinde taşma ve hata kontrolü yap.
- Plan, konsept veya formüllerde büyük değişiklikten sonra (başka bir ajan çalıştıktan sonra da) `python3 -I araclar/tutarlilik.py` çalıştır; önce plan kopyasını yeniden dışa aktar. Ad farkı yüzünden çıkan uyarıyı betikteki `BILINEN` eşleşmelerine ekle.
- Doğrulanmamış rakam ve mevzuat bilgisini "doğrulanacak" diye işaretle.

## Model seçimi

Geliştirme işlerinde model `gelistirme/ISLER.md`'deki Model sütunundan alınır. Aşağıdaki kurallar oradaki dağılımın dayanağıdır.

Ana oturumun modeli kullanıcı tarafından `/model` ile seçilir; Claude kendi modelini değiştiremez. Bu yüzden:

- **Alt görevlerde (Agent aracı) iş türüne göre model seç:**
  - `sonnet`: web araması ve sayfa okuma, dosya tarama ve listeleme, metin bulup değiştirme, biçim düzeltme, tablo veya liste dönüştürme, tekrar eden küçük düzenlemeler, ekran görüntüsü kontrolü.
  - `haiku`: çok basit ve çok sayıda tekrar eden işler (yeniden adlandırma listesi, sayım).
  - Ana model (Opus): plan ve kapsam kararları, formül ve hesap motoru spesifikasyonu, mevzuat ve vergi yorumu, tasarım kararları, plan incelemesi, birden çok kaynağı birleştiren araştırma özeti.
- **Ana oturum için öneri yap:** Kullanıcının isteği baştan sona düşük kaynaklı bir işse (yalnız metin düzeltme, küçük konsept değişikliği, dosya düzenleme), işe başlamadan önce tek satırla `/model` ile Sonnet'e geçmeyi öner. Karar veya derin analiz gerektiren bir iş gelince Opus'a dönmeyi öner. Öneriyi bir kez yap, ısrar etme.
- Emin değilsen ana modelde kal; yanlış sonuçla tekrar yapılan iş, ucuz modelin tasarrufundan pahalıdır.

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
2. Bağlı olduğu işler bitmiş, kimi `Claude`, durumu `bekliyor` olan ilk işi al. Kullanıcı başka bir iş ya da sayı söylediyse ona göre yap (ör. "devam, 3 iş").
3. İşi ISLER.md'deki **Model** sütunundaki modelle yap: Opus işini ana oturumda; Sonnet, Haiku ve Fable işini Agent aracıyla alt ajana ver (`model` parametresiyle). Alt ajana okunacak dosyaları, bitti ölçütünü ve "testleri çalıştır, en fazla 20 satır rapor ver" talimatını yaz. Birbirine bağlı olmayan işler paralel alt ajanlarla yürüyebilir.
4. Bitti ölçütünü kendin doğrula (testleri çalıştır, farkı incele). Geçmezse işi alt ajana geri gönder ya da düzelt.
5. ISLER.md'de durumu `bitti (tarih, commit)` yap; DURUM.md'de sıradaki işi, engelleri ve "Son biten işler"i güncelle; commit at ve `main`'e gönder.
6. Kullanıcıya en fazla beş satırla bildir: ne bitti, test sonucu, sıradaki iş. Kullanıcıdan karar gerekirse sor ve dur.
7. Aşama kapısı kapanınca kullanıcıya yeni oturum açmasını öner.

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

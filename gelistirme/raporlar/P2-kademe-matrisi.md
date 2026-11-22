# P2 — Home ve Business kademe matrisi taslağı

**Durum:** Taslak; ürün sahibinin O8 kararı için  
**Kaynaklar:** `plan/demleme-plani.md` › Modüller ve Deneme süresi; `web-stratejisi/45-gristora-pricing-membership-onboarding-plan.md`; `gelistirme/DURUM.md` › T2.28  
**Kural:** Doğrulanmamış sayısal sınırlar `to be verified` olarak işaretlenmiştir. Bu belge ürün planını veya pazarlama sitesini değiştirmez.

## 1. Kademe özeti

| Alan | Home | Business | Gerekçe / kaynak |
|---|---|---|---|
| Satın alma birimi | Hesap başına tek kullanıcı / 1 koltuk | Adlandırılmış kullanıcı / koltuk | 45 numaralı fiyatlandırma planı Home'u tek koltuk, Business'ı koltuk başına tanımlar |
| Kullanıcı sayısı | 1 koltuk; ek ortak kullanıcı sınırı ve davet davranışı **to be verified** | 5, 10, 15 veya 20 koltuk; 20 üzeri için satışla iletişim | Fiyatlandırma planındaki ticari paketler; ürün planı Ev modunda tarif veya parti için düzenleyici/izleyici davetini de tanımlar |
| Cihaz sayısı | **to be verified** | **to be verified** | Kaynaklarda sayısal cihaz kotası yok; cihaz entegrasyonlarının varlığı kota anlamına gelmez |
| Depolama | **to be verified** | **to be verified** | Kaynaklarda GB veya dosya kotası yok |
| Veri dışa aktarma | Sınırsız | Sınırsız | Ürün planı deneme sonrası verinin her zaman dışa aktarılmasını ve hiçbir katmanda dışa aktarmanın kapatılmamasını söyler |
| Deneme süresi | 3 ay (fiyatlandırma planı) | 1 ay (fiyatlandırma planı) | 45 numaralı fiyatlandırma planındaki ticari öneri |
| Faturalama dönemi | Aylık, yıllık, 2 yıl, 3 yıl | Aylık, yıllık, 2 yıl, 3 yıl | 45 numaralı fiyatlandırma planı; indirimler öneridir, onaylı fiyat değildir |

## 2. Modül matrisi

| Modül | Home | Business | Gerekçe |
|---|---|---|---|
| Çekirdek | Dahil | Dahil | Tarif, hesaplar, stil kılavuzu, demleme günü, fermantasyon, ölçüm, bildirim, içe/dışa aktarma iki modda da bulunur |
| Cihazlar | Dahil | Dahil | iSpindel, RAPT ve Tilt gibi entegrasyonlar iki modda da tanımlıdır; İşletme tarafında tank sensörleri de vardır |
| Stok | Sade | Tam | Home miktar, minimum seviye ve alışveriş listesiyle sınırlı sade deneyim; Business lot, mal kabul, sayım, ayırma ve izlenebilirlik içerir |
| Üretim | Hariç | Dahil | Tanklar, transfer, üretim panosu, tank takvimi, ambalaj kaydı ve saha görünümü İşletme modülüdür |
| Kalite | Hariç | Dahil | Spesifikasyon, laboratuvar sonucu, serbest bırakma, CIP ve maya jenerasyonu İşletme modülüdür |
| Maliyet ve raporlar | Hariç | Dahil | Parti/litre maliyeti, stok değerleme ve rapor dışa aktarma İşletme modülüdür |
| Vergi ve uyum | Hariç | Dahil | TTB/HMRC hesapları ve mevzuat raporları İşletme modülüdür |
| Ekip | Hariç | Dahil | Roller, yetki, ortak cihaz, onay akışı ve denetim kaydı İşletme modülüdür |
| Bağlantılar | 2. sürüm | 2. sürüm | Açık API ve MCP bağlantısı iki mod için de planlıdır; ürün planı her ikisini de 2. sürüm olarak işaretler |

## 3. Sınırlar ve gerekçeler

| Sınır | Home | Business | Gerekçe |
|---|---|---|---|
| Koltuk | 1 | 5 / 10 / 15 / 20; 20 üzeri satış iletişimi | Ticari paket yapısı 45 numaralı fiyatlandırma planında açıkça tanımlıdır |
| Cihaz kotası | **to be verified** | **to be verified** | Hiçbir kaynak cihaz başına veya eşleştirilmiş cihaz toplamı için sayı vermiyor |
| Depolama kotası | **to be verified** | **to be verified** | Hiçbir kaynak hesap başına saklama alanı miktarı vermiyor |
| Veri dışa aktarma | Sınırsız | Sınırsız | Veri sahipliği ilkesi; deneme bitince de dışa aktarma açık kalır |
| Üretim / kalite / ekip özellikleri | Dahil değil | Dahil | Bu modüller ürün planında yalnız İşletme için tanımlanmıştır |

## 4. Deneme ve ödeme notları

- 45 numaralı fiyatlandırma planı Home için 3 ay, Business için 1 ay deneme önerir.
- Ürün planındaki genel deneme kuralı 1 ay tam denemedir. `gelistirme/DURUM.md` › T2.28, mevcut ürün uygulaması için denemeyi `workspaces.created_at + 1 month` olarak kaydeder.
- Bu nedenle Home deneme süresi 3 ay, ürün planı ve mevcut uygulama kaydıyla çelişir. O8 kararı olmadan ürün davranışı değiştirilmemelidir.
- Deneme sırasında tüm özellikler açık kalır. Deneme bitince veriler silinmez ve dışa aktarma her zaman açık kalır; ücretli özellikler salt okunur olur.
- 15% / 25% / 35% yıllık / iki yıllık / üç yıllık indirimler fiyatlandırma planında öneridir; kesin fiyat, para birimi, vergi ve tahsilat koşulları onaylanmamıştır. Bu taslak parasal fiyat üretmez.
- Gerçek ödeme ve abonelik aktivasyonu bu işin kapsamında değildir.

## 5. Differences

1. **Home denemesi:** 45 numaralı web fiyatlandırma planı 3 ay önerir; ürün planı ve T2.28 uygulama kaydı 1 ay belirtir. O8 ürün kararı gerekir.
2. **Home koltuk sayısı:** 45 numaralı fiyatlandırma planı Home'u tek koltuk olarak fiyatlandırır. Ürün planı Ev modunda tarif veya parti için düzenleyici/izleyici daveti de tanımlar. Bu davetlerin ücretli koltuk sayımına etkisi açık değildir; doğrulanmalıdır.
3. **Cihaz ve depolama kotaları:** Ürün planı bunları ücretsiz/ücretli katman ayrımında kullanılacak alanlar olarak sayar, ancak sayısal değer vermez. Her iki kademe için de `to be verified`.
4. **Business koltuk aralığı:** 45 numaralı fiyatlandırma planı 5 kişilik artışlarla 5–20 koltuk sunar; 20 üzeri için satış iletişimi gerekir. Ürün planında bu ticari sayı aralığı tanımlı değildir.

## 6. O8 için karar listesi

- Home için 3 aylık deneme önerisi, ürün planındaki 1 aylık kuralın yerini alacak mı?
- Home'da paylaşılan tarif/parti davetleri ücretli koltuk sayılır mı?
- Home ve Business için eşleştirilmiş cihaz kotası kaçtır?
- Home ve Business için dosya/depolama kotası kaç GB'dir?
- Business 20 koltuk üstü satış iletişimi ve 5 koltukluk artış kuralı onaylanıyor mu?

**Sonuç:** Kaynaklarla doğrulanabilen modül ve koltuk yapısı taslaklandı. Sayısal cihaz/depolama kotaları ile Home deneme süresi çelişkisi açık karar olarak bırakıldı.

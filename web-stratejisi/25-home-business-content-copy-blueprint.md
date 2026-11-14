# Demleme — Home / Business içerik ve kopya taslağı

**Work package:** WP-14  
**Tarih:** 15 Kasım 2026  
**Durum:** Editoryal taslak; claim review ve uygulama onayı bekliyor.  
**Bağımlılıklar:** WP-03, WP-05, WP-07, WP-10, WP-12, WP-13.  
**Kapsam:** Mevcut Home ve Business sayfalarının metin risklerini gidermek ve iki profil için ayrı, kanıt odaklı içerik akışını tarif etmek. Bu paket site kodunu değiştirmez.

## 1. Editoryal karar

FoamByte'ın ekran görüntüsü, fiyat, testimonial veya feature iddiaları taklit edilmeyecek. Demleme'nin farkı iki ayrı kullanıcı yolculuğu, teknik açıklanabilirlik, ürün aşamasının dürüst gösterimi ve profesyonel üretim kayıtlarının ERP'den net ayrılması olacak.

Her sayfa ve feature açıklamasında şu durum sözlüğü kullanılmalı:
- **Available:** Üretimde yayınlanmış, uçtan uca test edilmiş ve kullanıma açık.
- **In development:** Aktif geliştirme kapsamında; henüz genel kullanım vaadi değil.
- **Planned:** Onaylı ürün planında; henüz mevcut değil.
- **Under consideration:** Araştırma/karar aşamasında; yol haritası taahhüdü değil.
- **Requires validation:** Teknik, mevzuat veya ürün sahibi doğrulaması tamamlanmadan iddia edilemez.

## 2. P0 metin düzeltmeleri — önerilen nihai metin

| Sayfa / mevcut risk | Önerilen metin | Gerekçe |
|---|---|---|
| Home: “A complete toolkit for your next brew” | **“A more considered approach to every brew.”** | Tamamlanmış ürün algısını kaldırır; marka tonunu korur. |
| Home: “selected advanced features available to enable” | **Kaldır.** Yerine: “Explore the product areas in development, with each capability labelled by its current stage.” | Mevcut/etkin özellik izlenimi yaratmaz. |
| Business: “From formulation to production control” | **“From recipe review to production records.”** | Otomatik kontrol veya kapsamı aşan operasyonel kontrol iddiasını önler. |

Bu kopyalar öneridir; yayın öncesi WP-03 claim register ve güncel ürün planıyla son eşleştirme zorunludur.

## 3. Homebrewers — sayfa içerik sırası ve taslak

### Hero
- **Eyebrow:** FOR HOME BREWERS
- **H1:** Make every brew more considered.
- **Body:** Demleme is brewing software in development for homebrewers who want to understand the ingredients, calculations and observations behind each batch.
- **Primary link:** Explore the homebrewing workflow
- **Secondary link:** Explore product areas

### The work behind a brew
Başlık: **A brew is a series of connected decisions.**

1. **Recipe design** — What are you trying to brew, and which recipe inputs shape the result?
2. **Ingredients and water** — Which ingredient and water assumptions are you using?
3. **Brewing calculations** — What does each gravity, volume or bitterness estimate mean, and what assumptions affect it?
4. **Brew-day measurements** — Which planned values should be compared with actual observations?
5. **Fermentation records** — Which measurements and notes describe how the batch developed?
6. **Batch review** — What did you learn that should inform the next recipe?

Her adım yalnız WP-03 ve ürün planıyla doğrulanmış feature'a bağlanır. Fermentation finish-date/FG prediction, deviation alerts ve model-driven calibration recommendations mevcut özellik gibi yazılmaz; Under consideration / Requires validation olarak kalır.

### Technical clarity
Başlık: **Understand the number, not just the result.**

Bir teknik rehber veya hesaplama açıklaması yayınlandığında yöntem, girdiler, birimler, varsayımlar, örnek ve sınırlar aynı yerde gösterilir. Doğrulanmamış public calculator'a CTA verilmez.

### Product stage
Başlık: **Know what is being built.**

“Demleme is in development. Product areas are labelled by stage so you can distinguish what is available from what is planned or still being evaluated.”

### Home FAQ başlıkları
- Who is Demleme being built for?
- Is Demleme available to use today?
- Which brewing calculations are supported?
- How are units and calculation assumptions explained?
- Does Demleme predict fermentation outcomes?
- Can I import or export recipe and batch data?
- How are product capabilities and development stages labelled?

Yanıtlar yalnız doğrulanmış ürün/teknik kaynaklarla yazılır. Veri saklama, privacy, import/export veya cihaz desteği hakkında tahmin yürütülmez.

## 4. Professional Breweries — sayfa içerik sırası ve taslak

### Hero
- **Eyebrow:** FOR PROFESSIONAL BREWERIES
- **H1:** Build consistency around better production records.
- **Body:** Demleme is being developed for professional brewing teams that need a clearer view of recipe decisions, batch information and production records.
- **Primary link:** Explore the brewery workflow
- **Secondary link:** Understand the product scope

### A shared view of the batch
Başlık: **Connect the decisions with the records.**

1. **Recipe review** — What recipe definition and target values is the team working from?
2. **Production planning** — Which approved planning information is relevant before a batch begins?
3. **Batch records** — Which planned and actual values need to be recorded and reviewed?
4. **Fermentation and vessel observations** — Which observations belong to a batch or vessel record, subject to confirmed product scope?
5. **Quality and traceability** — Which quality results, lots and relationships are in the approved product scope?

Bu sıra hedeflenen anlatı modelidir; her adımın ürün durumu ayrıca gösterilir. Planning, vessel observations, quality veya traceability ifadeleri mevcut kabiliyet oldukları anlamına gelmez.

### Roles and information
Başlık: **Different roles. A clearer production picture.**

| Role | Primary information need | Editorial emphasis |
|---|---|---|
| Head brewer | Recipe intent, target values, batch review | Recipe decisions and consistency |
| Production operator | Planned vs actual batch information, observations | Clear records and handover |
| Quality role | Measurements, test results, traceability relationships | Evidence, context and review |

Role-based permissions ve approval workflows ayrıca doğrulanmalıdır; rol anlatımı bunların mevcut olduğu iddiasına dönüştürülmemelidir.

### What Demleme is — and is not
Başlık: **A brewing workspace, not a general ERP.**

“Demleme focuses on brewing-related recipe, batch and production information. It is not positioned as a general ERP, and it does not claim to manage sales, customer orders, distribution or shipping, keg tracking, CRM or accounting.”

Bu bölüm görünür olmalı; footer dipnotuna taşınmamalıdır.

### Business FAQ başlıkları
- Which types of breweries is Demleme being designed for?
- Does Demleme replace an ERP or brewery-wide business system?
- Does it manage sales, orders, distribution or keg tracking?
- Which production and quality records are in scope?
- Are APIs, webhooks and integrations available today?
- How are user roles, permissions and approvals handled?
- What is available now versus planned?

API/webhook, role permission, audit, quality/compliance ve integration yanıtları ilgili onaylar olmadan Available şeklinde sunulmaz.

## 5. Home / Business ayrımının kabul kriterleri

| Boyut | Homebrewers | Professional Breweries |
|---|---|---|
| Ana kullanıcı işi | Kendi reçetesini tasarlamak, demlemek, ölçmek ve öğrenmek | Ekip içinde reçete kararlarını ve batch/production kayıtlarını gözden geçirmek |
| Dil | Öğrenme, deney, ölçüm, tarif, batch review | Ekip, üretim kaydı, planned vs actual, kalite ve izlenebilirlik |
| Örnek | Tek bir homebrew batch ve ölçüm günlüğü | Reçete hedefi ile batch kaydı ilişkisi |
| Teknik güven | Formül, birim, varsayım, ölçüm | Kayıt kaynağı, lot/batch ilişkisi, rol ve review sınırları |
| CTA | Teknik rehberler ve homebrew workflow | Business workflow ve kapsam açıklaması |
| Kapsam dışı | Onaysız tahmin/otomasyon ve mevcut olmayan cihaz akışları | ERP, satış, sipariş, dağıtım, keg tracking, CRM, muhasebe |

Kabul koşulları:
- Her audience sayfası diğer sayfadan yalnız başlık değiştirerek türetilmiş olmayacak.
- Her birinde en az beş ayrı kullanıcı işi açıkça anlatılacak.
- Her feature link'i ilgili audience ve ürün aşamasıyla eşleşecek.
- Mevcut olmayan signup, trial, pricing, demo, waitlist veya sosyal kanıt CTA'sı bulunmayacak.
- Her teknik iddia WP-03 claim register ve WP-10 evidence register'a bağlanacak.
- Business ERP sınırı görünür ve açık olacak.

## 6. Tasarım / içerik uygulama notları

- FoamByte'ın gerçek ürün ekranları yerine sahte dashboard veya sahte uygulama ekranı üretme.
- Home için homebrew sürecini; Business için ekip/üretim kaydı ilişkisini anlatan ayrı, sade süreç diyagramları tercih et.
- Editorial + technical tasarım sistemi ve canonical tokens.json korunmalı.
- Gerçek fotoğraflar yalnız kullanıcı tarafından sağlandığında veya açıkça onaylandığında eklenmeli.
- Kartlarda başlık, kullanıcı sorusu, ürün aşaması ve ilişkili rehber/feature bağlantısı bulunmalı.
- Gerçek browser QA olmadan responsive, contrast, layout veya interaction kalitesi onaylanmış sayılmaz.

## 7. Bağımlılıklar ve sonraki adım

1. WP-03 ve ürün planı üzerinden her önerilen cümleyi claim-level kontrol et.
2. WP-10 evidence register'a her teknik açıklama ve FAQ yanıtı için kanıt sahibi/kanıt türü ekle.
3. Kullanıcı onayı ve WP-12 implementation gate olmadan marketing-site dosyalarını değiştirme.
4. İçerik onayı sonrası Home ve Business metinlerini ayrı PR'da uygula; calculator, fiyat, CTA veya görsel kapsamını bu PR'a ekleme.
5. Vercel Preview ve gerçek tarayıcı QA'yı ayrı launch gate olarak tut.

**Bu WP yalnızca editoryal taslaktır. Production implementation yetkisi vermez.**
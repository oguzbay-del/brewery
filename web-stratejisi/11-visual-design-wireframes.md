# Pazarlama sitesi — görsel tasarım ve responsive wireframe

**Durum:** İlk tasarım yönü ve içerik wireframe'i. Kodlama veya görsel üretim başlamadı.  
**Kapsam:** Yalnızca Demleme pazarlama sitesi. Ürün uygulaması ve Claude geliştirme kuyruğu kapsam dışıdır.  
**Hedef:** ABD ve İngiltere pazarı; İngilizce içerik; iki ayrı audience yolu.

## 1. Tasarım kararı

Demleme'nin sitesi bir AI/SaaS şablonu gibi değil, bira üretimini gerçekten bilen bir yazılım markası gibi görünmeli.

**Görsel yön: Editorial + Technical + Human**

- Editorial: güçlü tipografi, net başlık hiyerarşisi, geniş ve kontrollü boşluk, düzenli grid.
- Technical: gerçek reçete/ölçüm bağlamı, açıklanabilir sayılar, net etiketler, doğru terminoloji.
- Human: evde bira üreten kişi ve gerçek profesyonel üretim ortamı; stok teknoloji görselleri değil.
- Amber yalnızca vurgu ve ana eylem için; sayfayı turuncuya boyama.
- Her bölümün ayrı bir amacı ve tek bir ana mesajı olsun.

### Kesinlikle kullanma

- Mor/mavi gradient, neon glow, blur orb, cam kart, yapay 3D küre.
- Her içerik parçasını kart içine koymak; gereksiz border ve gölge.
- Sahte dashboard, uydurma grafik, hayali ürün ekran görüntüsü veya gerçek olmayan metrik.
- Üst üste binen rastgele şekiller, anlamsız ikon dizileri, emoji.
- "Revolutionize", "Unlock", "Supercharge", "AI-powered" gibi ürünün gerçek değerini açıklamayan sloganlar.
- Otomatik kayan carousel, video autoplay, scroll-jacking ve gereksiz animasyon.
- Masaüstü sayfasını mobilde küçültüp sıkıştırmak.

## 2. Tasarım sistemi yaklaşımı

Mevcut `tasarim-sistemi/tokens.json` tek token kaynağı olmaya devam eder. Marketing site, uygulamanın iOS arayüzünü taklit etmez; aynı renk ve erişilebilirlik prensiplerini daha editorial bir web düzeninde kullanır.

### Renk

- Ana zemin: mevcut açık nötr `bg`; içerik yüzeyi beyaz `card`.
- Ana metin: `label`; açıklama: `label2`.
- Amber `tint`: link, küçük vurgu, aktif navigasyon.
- Amber `tint-fill`: yalnız ana CTA ve az sayıda önemli vurgu.
- İnce ayırıcı: `sep`.
- Durum renkleri pazarlama dekorasyonu olarak kullanılmaz.
- Koyu tema ilk sürümün zorunlu kapsamı değildir; kontrast ve token altyapısı bozulmadan sonraki iterasyonda değerlendirilebilir.

### Tipografi

- Lisanslı ve gömülü font ekleme. Sistem font stack'i kullan.
- Başlıklar güçlü ama ölçülü; 2–3 satırı aşan hero başlığından kaçın.
- Gövde metni 16–18 px; satır uzunluğu masaüstünde yaklaşık 60–75 karakter.
- Teknik değerler ve birimler eş aralıklı rakamla gösterilebilir; teknik olmayan metni monospace yapma.
- Tamamı büyük harf, geniş harf aralığı ve küçük gri metinle uzun açıklamalar yazma.

### Grid ve boşluk

- Desktop içerik genişliği: yaklaşık 1,160–1,240 px; ortalı.
- Desktop: 12 kolon; ana boşluk geniş ve tutarlı.
- Tablet: 8 kolon.
- Mobile: 4 kolon, 20–24 px yatay sayfa boşluğu.
- Bölümler arası boşluk masaüstünde yaklaşık 88–120 px, mobilde 56–72 px.
- Kart köşeleri ve gölgeler minimum; bölümler çoğunlukla boşluk, çizgi ve görsel kompozisyonla ayrılır.

### Görsel dili

- Homebrewers: gerçek ev tipi demleme alanı, malt/şerbetçiotu/maya detayları, kettle ve ölçüm anı.
- Breweries: gerçek craft brewery, paslanmaz tanklar, hortum/vanalar, üretim ekibi ve gerçek çalışma ortamı.
- Görsel seçimi sayfanın audience'ına ait olmalı; aynı hero görselini iki sayfada kullanma.
- Fotoğraf gerçek ürün ekranı değildir. Ürün ekranı kullanılacaksa gerçek, güncel uygulama sürümünden alınmış olmalı.
- Görsel yoksa uydurma dashboard yerine tipografik/diagramatik, açıklayıcı bir kompozisyon kullan.
- Fotoğraflar doğal ışıklı, gerçekçi, editorial; aşırı doygun turuncu ve yapay sinematik efekt yok.
- Her görselin anlamlı alt metni olmalı; dekoratif görsel boş alt metin kullanmalı.

## 3. Navigasyon ve ortak şablon

### Desktop header

Sol: Demleme wordmark (logo hazır olana kadar metin).  
Orta: Homebrewers · Breweries · Platform · Resources.  
Sağ: Sign in (text link) · Get started (tek amber CTA).

- Header beyaz/açık zemin; gölge yok; alt çizgi yalnız gerekiyorsa.
- Pricing ticari kararlar netleşene kadar ana navigasyona konmaz.
- Platform ve Resources gerçek içerik sayfalarına gitmeli; boş kategori açma.
- Sign in mevcut uygulamaya gider. URL yapılandırmadan gelir.

### Mobile header

- Sol wordmark; sağ menü ikonu.
- Menü açıldığında büyük ve okunur bağlantılar; Homebrewers ve Breweries ilk iki sırada.
- Get started menü içinde belirgin.
- Header ekranı kaplamaz; sabit alt CTA barı ilk sürümde kullanılmaz.
- Dokunma alanları en az 44 px.

### Footer

Dört sade kolon: Product / For brewers / Resources / Company.  
Alt satır: copyright, privacy, terms. Yalnız gerçekten mevcut sayfalar gösterilir. Sosyal ikonlar boş link olarak eklenmez.

## 4. Ana sayfa wireframe

### Desktop (yaklaşık 1440 px)

```text
┌─────────────────────────────────────────────────────────────────────┐
│ DEMLEME     Homebrewers  Breweries  Platform  Resources   Sign in   │
│                                                        [Get started]│
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  BREWING SOFTWARE                                                   │
│  Brewing software for                         [REAL BREWING PHOTO   │
│  people who care about                         OR VERIFIED PRODUCT  │
│  the details.                                  SCREEN, NOT A MOCKUP]│
│  A technical workspace for homebrewers                              │
│  and professional breweries.                                       │
│  [Explore Homebrewing]  [Explore Breweries]                         │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  TWO WAYS TO BREW WITH DEMLEME                                      │
│  ┌────────────────────────────┐  ┌───────────────────────────────┐  │
│  │ [HOME BREWING PHOTO]       │  │ [REAL BREWERY PHOTO]          │  │
│  │ For Homebrewers            │  │ For Professional Breweries    │  │
│  │ Plan your next brew...     │  │ Bring more structure...       │  │
│  │ Explore Homebrewing →      │  │ Explore Breweries →           │  │
│  └────────────────────────────┘  └───────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────┤
│  THE DETAILS MATTER                                                 │
│  Clear inputs. Understandable calculations. Useful records.         │
│  [Three editorial columns separated by whitespace / fine rules]    │
├─────────────────────────────────────────────────────────────────────┤
│  BUILT AROUND THE BREWING PROCESS                                   │
│  [REAL PRODUCT SCREEN]  Short explanation                           │
│                        [REAL PRODUCT SCREEN]  Short explanation     │
├─────────────────────────────────────────────────────────────────────┤
│  TECHNICAL CLARITY                                                  │
│  Explain only verified calculation scope, units and terminology.    │
│  Link to technical methodology / calculations page.                │
├─────────────────────────────────────────────────────────────────────┤
│  Ready to plan your next brew?                 [Get started]        │
├─────────────────────────────────────────────────────────────────────┤
│  Footer                                                             │
└─────────────────────────────────────────────────────────────────────┘
```

### Mobile (yaklaşık 390 px)

```text
┌──────────────────────────────┐
│ DEMLEME                  [≡] │
├──────────────────────────────┤
│ BREWING SOFTWARE             │
│                              │
│ Brewing software for people  │
│ who care about the details.  │
│                              │
│ A technical workspace for    │
│ homebrewers and professional │
│ breweries.                   │
│                              │
│ [Explore Homebrewing]        │
│ [Explore Breweries]          │
│                              │
│ [FULL-WIDTH REAL PHOTO]      │
├──────────────────────────────┤
│ TWO WAYS TO BREW             │
│                              │
│ [HOME BREWING PHOTO]         │
│ For Homebrewers              │
│ Short, specific description  │
│ Explore Homebrewing →        │
│                              │
│ [BREWERY PHOTO]              │
│ For Professional Breweries   │
│ Short, specific description  │
│ Explore Breweries →          │
├──────────────────────────────┤
│ THE DETAILS MATTER           │
│ One short statement          │
│ One supporting detail        │
│ One supporting detail        │
│ One supporting detail        │
├──────────────────────────────┤
│ REAL PRODUCT SCREEN          │
│ Short explanation            │
│                              │
│ TECHNICAL CLARITY            │
│ Short explanation + link     │
├──────────────────────────────┤
│ [Get started]                │
│ Footer                       │
└──────────────────────────────┘
```

### Ana sayfa içerik taslağı

**Eyebrow:** BREWING SOFTWARE

**H1:** Brewing software for people who care about the details.

**Intro:** A technical workspace for homebrewers and professional breweries. Explore a clearer way to work with the details behind every brew.

**İki ana yol:**
- **For Homebrewers** — Plan your recipes, understand your brewing numbers, and learn from each batch.
- **For Professional Breweries** — Bring structure to recipe and batch information across your brewing operation.

**Bölüm başlığı:** The details matter.

**Destek mesajı:** Brewing decisions depend on good information. Demleme is being built around the recipes, calculations and records that help brewers understand their process.

Bu metinler tasarım için başlangıç kopyasıdır. Ürün kapsamı ve mevcut ekranlarla doğrulanmadan kesin özellik vaadi olarak yayınlanmaz.

## 5. Homebrewers sayfası wireframe

### Desktop

```text
┌─────────────────────────────────────────────────────────────────────┐
│ Header                                                              │
├─────────────────────────────────────────────────────────────────────┤
│ FOR HOMEBREWERS                                                     │
│ Make your next brew a more informed one.    [REAL HOME BREW PHOTO]  │
│ A focused brewing workspace for people                              │
│ who want to understand their recipes and                            │
│ the numbers behind them.                                            │
│ [Get started]   [Explore features]                                  │
├─────────────────────────────────────────────────────────────────────┤
│ YOUR RECIPE, WITH THE DETAILS IN VIEW                               │
│ [Verified recipe screen]                 Plain-language explanation│
├─────────────────────────────────────────────────────────────────────┤
│ UNDERSTAND THE NUMBERS                                              │
│ OG / FG / ABV / IBU — only metrics supported by the product.        │
│ Explain inputs, units and calculation boundaries.                   │
├─────────────────────────────────────────────────────────────────────┤
│ FROM PLAN TO BREW DAY                                               │
│ [Real brew-day photo]                   Verified workflow detail    │
├─────────────────────────────────────────────────────────────────────┤
│ A BETTER RECORD OF WHAT YOU BREW                                    │
│ Show only existing recipe / batch history capability.              │
├─────────────────────────────────────────────────────────────────────┤
│ FAQ + Get started                                                   │
│ Footer                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

### Mobile

```text
┌──────────────────────────────┐
│ Header                       │
├──────────────────────────────┤
│ FOR HOMEBREWERS              │
│ Make your next brew a more   │
│ informed one.                │
│ Short intro                  │
│ [Get started]                │
│ [Explore features →]         │
│ [Homebrew photo]             │
├──────────────────────────────┤
│ YOUR RECIPE, WITH THE        │
│ DETAILS IN VIEW              │
│ [Verified product screen]    │
│ Explanation                  │
├──────────────────────────────┤
│ UNDERSTAND THE NUMBERS       │
│ Short technical explanation  │
├──────────────────────────────┤
│ FROM PLAN TO BREW DAY        │
│ [Photo]                      │
│ Explanation                  │
├──────────────────────────────┤
│ FAQ                          │
│ [Get started]                │
│ Footer                       │
└──────────────────────────────┘
```

## 6. Professional Breweries sayfası wireframe

### Desktop

```text
┌─────────────────────────────────────────────────────────────────────┐
│ Header                                                              │
├─────────────────────────────────────────────────────────────────────┤
│ FOR PROFESSIONAL BREWERIES                                          │
│ Bring more structure to your                 [REAL BREWERY PHOTO]   │
│ brewing operation.                                                  │
│ Technical brewing software for teams                                │
│ that value clear information and                                    │
│ repeatable processes.                                               │
│ [Get started with Business]  [Explore capabilities]                 │
├─────────────────────────────────────────────────────────────────────┤
│ KEEP RECIPE INFORMATION CONSISTENT                                 │
│ [Verified product screen]                 Explanation              │
├─────────────────────────────────────────────────────────────────────┤
│ MAKE BATCH INFORMATION EASIER TO FOLLOW                             │
│ Explain actual batch workflow; no invented team permissions.        │
├─────────────────────────────────────────────────────────────────────┤
│ TECHNICAL INFORMATION, NOT BLACK-BOX CLAIMS                         │
│ Calculation methodology, units, traceability — verified only.      │
├─────────────────────────────────────────────────────────────────────┤
│ WHAT DEMLEME IS — AND IS NOT                                        │
│ Brewing software. Not an ERP, accounting, sales or shipping system.│
├─────────────────────────────────────────────────────────────────────┤
│ FAQ + Get started with Business                                     │
│ Footer                                                              │
└─────────────────────────────────────────────────────────────────────┘
```

### Mobile

```text
┌──────────────────────────────┐
│ Header                       │
├──────────────────────────────┤
│ FOR PROFESSIONAL BREWERIES   │
│ Bring more structure to your │
│ brewing operation.           │
│ Short intro                  │
│ [Get started with Business]  │
│ [Explore capabilities →]     │
│ [Brewery photo]              │
├──────────────────────────────┤
│ KEEP RECIPE INFORMATION     │
│ CONSISTENT                   │
│ [Verified product screen]    │
│ Explanation                  │
├──────────────────────────────┤
│ BATCH INFORMATION            │
│ Explanation                  │
├──────────────────────────────┤
│ TECHNICAL CLARITY            │
│ Explanation                  │
├──────────────────────────────┤
│ WHAT DEMLEME IS — AND IS NOT │
│ Clear product boundaries     │
├──────────────────────────────┤
│ FAQ                          │
│ [Get started with Business]  │
│ Footer                       │
└──────────────────────────────┘
```

## 7. İçerik doğruluğu — yayın öncesi kontrol

Her sayfa bölümü için aşağıdaki kontrol yapılır:

| İçerik iddiası | Kontrol |
| --- | --- |
| Recipe creation / editing | Gerçek ürün akışı ve ekranı |
| Ingredients management | Gerçek veri modeli ve kullanılabilir akış |
| OG, FG, ABV, IBU, colour, water calculations | Formül spesifikasyonu ve test vektörleri |
| Brew-day planning / measurements | Gerçek uygulama akışı |
| Recipe history / batch comparison | Mevcut özellik ve veri modeli |
| Multi-user / team permissions | Yetki ve workspace kapsamı |
| Batch records / traceability | Gerçek kayıt alanları ve geçmiş |
| Data security / backup / ownership | Belgelenmiş teknik uygulama ve politika |
| Mobile / offline / integrations | Gerçek platform ve entegrasyon durumu |

**Yayın kuralı:** Ürün kanıtı bulunmayan iddia çıkarılır veya geleceğe yönelik plan olarak açıkça işaretlenir. Mockup, özelliğin varlığının kanıtı değildir.

## 8. Responsive davranış

- Breakpoint başlangıçları: mobile 0–767 px; tablet 768–1023 px; desktop 1024 px ve üzeri. İçeriğe göre gerektiğinde ayarlanır.
- Desktop hero iki kolon; mobile tek kolon. Mobilde önce başlık, açıklama ve CTA; görsel sonra.
- İki audience yolu desktop'ta yan yana; mobilde dikey.
- Teknik karşılaştırma tablosu mobilde yatay kaydırmaya zorlanmaz; anlamlı satır bloklarına dönüşür.
- Ekran görüntüsü mobilde okunabilir kalır; kırpıp önemli UI öğelerini gizleme.
- Body metin en az 16 px; dokunma hedefleri en az 44 px.
- Menü klavye, ekran okuyucu ve dokunma ile kullanılabilir; focus görünür.
- Hareket azaltma tercihi desteklenir; animasyon bilgi taşımak için zorunlu olmaz.

## 9. İlk görsel tasarım teslimat sırası

1. Önce üç sayfanın low-fidelity wireframe'i (desktop + mobile) ve içerik onayı.
2. Sonra ana sayfanın high-fidelity görsel yönü.
3. Aynı görsel dilde Homebrewers ve Breweries sayfaları.
4. Sonra ortak header/footer, feature ve resource şablonları.
5. Gerçek ürün ekranları hazır olduğunda screenshot entegrasyonu.
6. Kodlama ve responsive QA.

**İlk tasarım onayında henüz fiyat sayfası, müşteri logosu, testimonial, sahte ürün dashboard'u, blog içeriği veya ürün özellik iddiaları üretme.**

## 10. Açık içerik / varlık ihtiyaçları

- Güncel ürün ekran görüntüleri (recipe, calculation, brew-day, batch — yalnız mevcut olanlar).
- Homebrewer ortamından lisanslı veya kendi çektiğimiz gerçek fotoğraf.
- Professional brewery ortamından lisanslı veya kendi çektiğimiz gerçek fotoğraf.
- Gerçek ürün logosu/wordmark (henüz yoksa geçici metin).
- App URL ve Sign in URL.
- Özellik doğrulama kaynağı: ürün ekranı, ürün dokümanı, test/formül spesifikasyonu.


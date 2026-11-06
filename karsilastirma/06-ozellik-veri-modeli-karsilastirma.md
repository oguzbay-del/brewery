# BeerSmith 4 ve Brewtarget: Özellik, Veri Modeli ve Mimari Karşılaştırması

**Amaç:** Kendi demleme uygulamanızda hangi özellikleri ve hangi veri modelini seçeceğinize karar vermek.
**Kapsam dışı:** Lisans konusu ve kod. Bu belge yalnızca inceleme sonucudur.

**İncelenen kaynaklar**

| Program | Sürüm | İncelenen kaynak |
|---|---|---|
| BeerSmith | 4.0.22 (macOS paketi) | `Resources.sqlite` (Menu/Option/Field/String), `BeerSmith.sqlite` (62 tablo), `DefRecipe.sqlite`, rapor ve BeerXML şablonları, `/home/user/beersmith4-ozellik-analizi.md`, `/home/user/formuller/*.md` |
| Brewtarget | 5.2.0 (+5.3.0 taslak), C++/Qt6 | `src/model`, `src/database/ObjectStoreTyped.cpp`, `DatabaseSchemaHelper.cpp` (şema sürümü 21), `src/serialization`, `src/undoRedo`, `src/unitTests`, `ui/*.ui`, `schemas/`, `translations/`, `CHANGES.markdown`, `dev-doc/` |

**Kanıt düzeyi:** BeerSmith'in çalıştırılabilir dosyası incelenmedi. Bu yüzden BeerSmith tarafındaki bilgiler menü, alan ve ayar tanımlarından ve veri şemasından çıkarıldı. Bir menü öğesinin var olması, özelliğin var olduğunu gösterir ama nasıl çalıştığını göstermez. Brewtarget tarafındaki bilgiler doğrudan kaynak koddan alındı.

**Simgeler:** ✅ var · ◐ kısmen var · ❌ yok · 🔜 Brewtarget'ın bir sonraki sürümü (5.3.0) için duyurulmuş

---

## 1. Özellik matrisi

### 1.1 İçecek türleri

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Bira: all-grain, extract, partial mash | ✅ | ✅ (`Recipe::Type`) | |
| Mead | ✅ Besin, sülfit ve tatlandırma araçları, ayrı brewsheet | ◐ Yalnızca tür olarak (`Mead`) | Brewtarget'ta mead'e özel hesap yok. |
| Şarap | ✅ | ◐ Tür olarak (`Wine`) | |
| Cider | ✅ Ayrı brewsheet (`CiderBrewSheet.htm`) | ◐ Tür olarak (`Cider`) | |
| Seltzer | ✅ (4.x sürüm notları) | ❌ (`Soda` / `Other` var) | |
| Kombucha, soda | ❌ | ◐ Tür olarak | Brewtarget bu türleri BeerJSON uyumu için ekledi. |
| Bal, meyve ve meyve suyu fermentable türleri | ✅ Brix/SG girişiyle | ✅ `Fruit`, `Juice`, `Honey` | |
| Maya dışı kültürler (Brett, Lacto, Kveik, bakteri) | ◐ Maya türü alanı üzerinden | ✅ `Yeast::Type` içinde | |

### 1.2 Tarif editörü ve tersine ayar araçları

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| OG, FG, ABV, IBU, renk ve stile uygunluk göstergesi | ✅ Alan yanında gösterge topları | ✅ `StyleRangeWidget`, `IbuGuSlider` | |
| IBU formülleri | Tinseth, Rager, Garetz | Tinseth, Rager, Noonan | 5.3.0 için ek IBU yöntemleri duyuruldu. |
| Whirlpool sıcaklığına bağlı hop kullanımı | ✅ | ❌ | |
| Mash hop ve first wort düzeltmesi | ✅ | ✅ (Seçenekler → IBU Adjustments) | |
| Renk formülleri | `formuller/01` | Mosher, Daniels, Morey | |
| Hedef OG'ye göre ayarlama (Adjust Gravity) | ✅ Kaydırıcıyla | ❌ | |
| Hedef renge göre ayarlama (Adjust Color) | ✅ | ❌ | |
| Hedef IBU'ya göre ayarlama (Adjust Bitterness, hop başına IBU) | ✅ | ❌ | |
| Tahılı yüzdeyle girme (Grain Pct) | ✅ | ◐ Yüzde sütunu düzenlenebilir (5.0.4 düzeltmesi) | |
| All-grain ↔ extract dönüşümü (Convert Recipe) | ✅ | ❌ | |
| Ölçekleme (Scale) | ✅ Boyut, verim veya ekipmana göre, sonrasında yeniden dengeleme | ✅ Ekipman ve verime göre, sihirbazla | |
| Malzeme ikamesi (Substitute), miktarları yuvarlama | ✅ | ❌ | |
| Check Recipe (eksiklik ve tutarlılık denetimi) | ✅ | ❌ | |
| Tarif kilitleme | ✅ `F_R_LOCKED` | ✅ `locked` | |
| Özel alanlar (custom fields) | ✅ `M_CUSTOM_FIELDS` | ❌ | |
| Tarif görseli | ✅ | ❌ | |
| Tadım puanı ve tadım notu | ✅ | ✅ `tasteRating`, `tasteNotes` | |
| Kalori | ✅ Tahmini ve ölçülen | ✅ Üç porsiyon birimiyle | |
| Fermentasyonda eklenen şekerleri sayan sanal OG ("Est OG Through Tertiary") | ✅ | ◐ Eklemelerin `stage` alanı var; böyle bir "sanal OG" alanı yok | |
| Maya starter, canlılık ve pitch rate | ✅ Tarife bağlı: 2 kademe, stir plate, hücre/ml-°P | ◐ Ayrı pitch aracı: üretim tarihine göre canlılık, havalandırma | |
| Mash sıcaklığına göre FG düzeltmesi | ✅ | ❌ | |

### 1.3 Malzeme ve profil türleri

| Tür | BeerSmith tablosu | Brewtarget sınıfı | Not |
|---|---|---|---|
| Fermentable | `M_GRAIN` | `Fermentable` | |
| Hop | `M_HOPS` | `Hop` | Brewtarget'ta yağ bileşenleri ve alfa/beta min–max aralığı da var. |
| Maya | `M_YEAST` | `Yeast` | |
| Misc | `M_MISC` | `Misc` | Brewtarget 5.2'de tuzlar Misc'in alt kategorisi oldu (`WaterAgent`). |
| Su | `M_WATER` | `Water` | |
| Stil | `M_STYLE` (BJCP 2015 dahil) | `Style` (varsayılan içerik BJCP 2021) | |
| Ekipman | `M_EQUIPMENT` (34 alan) | `Equipment` (yaklaşık 53 alan) | Brewtarget ekipmanı kap kap modelliyor (BeerJSON'daki gibi HLT, mash tun, lauter, kettle, fermenter, packaging). |
| Mash profili | `M_MASH` (adımlar JSON içinde) | `Mash` + `MashStep` | |
| Kaynatma profili | ❌ (ekipman ve tarif alanları) | ✅ `Boil` + `BoilStep` | |
| Fermentasyon profili | `M_AGE` (sabit: primary, secondary, tertiary, bulk, aging) | `Fermentation` + `FermentationStep` (sınırsız adım) | |
| Karbonasyon profili | ✅ `M_CARB` | ❌ (alanlar tarifin içinde, ayrıca Priming aracı) | |

### 1.4 Demleme günü, session ve brew log

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Adım adım demleme talimatı | ✅ Şablonla üretiliyor (`Brewsheet.htm` vb.), özel adım eklenebiliyor | ✅ `Instruction` varlığı, `BrewDayScrollWidget`, `BrewDayFormatter` | |
| Ölçülen değerler (pre-boil, OG, FG, hacimler) | ✅ Tarifteki `F_R_*_MEASURED` ve `*_SET` sütunları | ✅ Ayrı `BrewLog` kaydında beklenen/ölçülen çiftleri | Bu fark mimari açıdan önemli (bkz. §2). |
| Verim, ABV ve attenuation hesabı | ✅ | ✅ `computed_*` alanları | |
| Birden çok demleme kaydı | ◐ "New Session" tarifi Brew Log klasörüne kopyalıyor | ✅ Bir tarife bağlı N adet `BrewLog` | |
| Batch numarası | ◐ Sürüm numarası artırılıyor | ✅ (5.1.0) | |
| Ölçüm alanında açılır refraktometre | ✅ | ❌ (ayrı araç) | |

### 1.5 Zamanlayıcılar

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Mash/steep, kaynatma ve whirlpool zamanlayıcıları | ✅ Üç ayrı sekme; ayarla, duraklat, sıfırla | ✅ Çoklu zamanlayıcı (`TimerMainDialog`, `TimerListDialog`, `BoilTimer`) | |
| Ekleme zamanında alarm | ✅ Özel alarm sesi | ✅ Sesli klipler ("aroma hops", "check boil" vb.) | |
| Zamanlayıcı durumunun tarifte saklanması | ✅ `F_R_MASH_TIMER`, `F_R_BOIL_TIMER` | ❌ | |

### 1.6 Fermentasyon takibi ve cihazlar

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Fermentasyon okumaları (tarih, SG, sıcaklık, pH) | ✅ `AgeData` (tarifin içinde JSON) | ❌ | |
| Okuma grafiği | ✅ | ❌ | |
| CSV içe/dışa aktarma | ✅ | ❌ | |
| Tilt | ✅ Google Sheets bağlantısı ve CSV (`F_R_TILT_LINK`, renk, filtre süresi) | ❌ | |
| iSpindel, RAPT, Plaato | ❌ | ❌ | Rakiplerin ikisinde de olmayan bir boşluk. |

### 1.7 Envanter, alışveriş ve maliyet

| Özellik | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Stok miktarı | ✅ Malzemedeki `F_*_INVENTORY` alanı ve ayrı `M_INVENTORY` | ✅ Lot bazlı: `StockPurchase*` + `StockUse*` | Brewtarget her alışı ayrı lot olarak tutuyor (tarih, tedarikçi, son kullanma, fiyat; hop için alfa, form ve yıl). |
| Tariften stoktan düşme | ✅ Çift düşmeye karşı uyarıyla | ✅ `StockUse` (sebep, tarih ve `brew_log_id` tutuluyor) | |
| "Yalnızca stokta olanları göster" | ✅ | ◐ Envanter sütunu, sıralama ve anlık görüntü | |
| Alışveriş listesi | ✅ `M_SHOP`, sipariş eşiği | 🔜 5.3.0 | |
| Fiyat ve tarif maliyeti | ✅ Birim fiyat, toplam maliyet, para birimi ayarları | ◐ Alış fiyatı var (`CurrencyAmount`), tarif maliyeti yok | |

### 1.8 Takvim ve notlar

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Demleme takvimi | ✅ Klasöre veya demleyene göre filtrelenebiliyor | ❌ |
| Not ve görev listesi (öncelik, başlangıç/bitiş tarihi) | ✅ `M_NOTES` | ❌ (yalnızca tarif ve brew log notları) |

### 1.9 Hesap araçları

| Araç | BeerSmith | Brewtarget |
|---|---|---|
| Alcohol & Attenuation (kalori ve karbonhidrat dahil) | ✅ | ◐ Alcohol Percentage Tool (kalori yok) |
| Hop Bitterness (bağımsız IBU hesabı) | ✅ | ❌ |
| Hop Age (HSI ile alfa kaybı) | ✅ | ❌ (HSI alanı var, araç yok) |
| Hop Rate (g/l, oz/gal) | ✅ | ❌ |
| Boil Off | ✅ | ❌ |
| Carbonation / priming | ✅ | ✅ Priming Calculator (4 şeker türü) |
| Decoction Volume | ✅ | ◐ Mash Designer içinde |
| Dilution | ✅ | ❌ |
| Adjust Gravity | ✅ | ✅ Gravity (OG) Correction Tool |
| Hydrometer Adjust | ✅ | ✅ |
| Infusion / Strike | ✅ | ✅ Strike Water Calculator |
| Mash Adjust | ✅ | ◐ Mash Wizard / Mash Designer |
| Mash pH ve asit | ✅ Tahminden veya ölçümden | ◐ Water Adjuster (Kai Troester modeli, tuzlar, RO yüzdesi) |
| Water Profile (en uygun tuz eşleştirmesi) | ✅ | ◐ Elle tuz ekleme, hedef profil karşılaştırması |
| Water Needed | ✅ | ◐ Mash Wizard'ın toplam su hesabı |
| Mead Nutrients, Sulfite, Backsweeten | ✅ | ❌ |
| Parti-Gyle | ✅ | ❌ |
| Refractometer | ✅ Kalibrasyonlu | ✅ (OG/FG, ABV, ABW, gerçek ekstrakt) |
| Weight-to-Volume (fıçı boyutları) | ✅ | ❌ |
| Yeast Starter | ✅ | ◐ Pitch Rate Calculator |
| Birim dönüştürücüler | ✅ 7 ayrı araç | ✅ Tek araç (Unit Conversion) |
| Mash Wizard: no/fly/batch sparge | ◐ Hazır mash profilleriyle | ✅ |
| Equipment Wizard | ✅ | ❌ |

Not: BeerSmith her aracın son girdilerini kendi tablosunda saklıyor (29 adet `M_TOOL_*` tablosu). Brewtarget'ın araçları ise geçici; girdileri kaydetmiyor.

### 1.10 İçe ve dışa aktarma

| Format | BeerSmith | Brewtarget | Not |
|---|---|---|---|
| Kendi formatı | `.bsmx` (XML benzeri; içe ve dışa) | Kendi dosya formatı yok; DotBeer'i destekliyor | |
| BeerXML 1.0 | ✅ Şablonla dışa, içe aktarma | ✅ Kodla tanımlı eşleme (XSD doğrulamalı), uzantı etiketleri | Ayrıntılar §4'te. |
| BeerJSON 1.0 | ❌ Kaynaklarda hiç geçmiyor | ✅ İçe ve dışa | |
| DotBeer 1.0.0 (MPL 2.0 lisanslı açık şema) | ❌ | ✅ (5.2.0) | Referansla çalışan JSON formatı. |
| CSV | ✅ Rapor, fermentasyon verisi, Tilt | ❌ | |
| BBCode ve düz metin (panoya) | ❌ | ✅ | |
| Eski sürümlerden içe aktarma | ✅ BeerSmith 1.4, 2 ve 3 | ✅ Eski Brewtarget veritabanını yükseltme | |
| Belirli bir klasöre içe aktarma | ❌ | ✅ (5.2.0) | |

### 1.11 Raporlar ve yazdırma

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Rapor üretimi | HTML ve metin şablonları, `$DEĞİŞKEN` yer tutucuları (213 ayrı değişken; brewsheet'te 46) | C++ ile HTML üretimi (`RecipeFormatter`, `BrewDayFormatter`, `StockFormatter`) |
| Kullanıcının rapor şablonu eklemesi | ✅ İçe ve dışa aktarma | ❌ |
| İçeceğe özel brewsheet | ✅ Bira, extract, mead, cider; yarışma, alışveriş | ❌ |
| Önizleme ve yazdırma | ✅ Sekme ve liste yazdırma | ✅ `PrintAndPreviewDialog` |
| Açık ve koyu CSS | ✅ `style.css`, `styledark.css` | ◐ Qt teması |

### 1.12 Bulut, senkron ve paylaşım

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Bulut ve yerel depolama modu | ✅ İlk açılışta seçiliyor | ❌ |
| Senkron alanları | ✅ `_CLOUDID_`, `_CLOUD_STATE_`, `_MOD_` ve her türün `M_CLOUD_*` ikiz tablosu | ❌ |
| Çok cihazdan ortak kullanım | ✅ Bulut senkronuyla | ◐ Paylaşılan PostgreSQL sunucusu |
| Tarif paylaşma (özel/genel/listelenmemiş), puanlama, yorum | ✅ | ❌ |
| Bulut arama (sıralama, sayfalama) | ✅ | ❌ |
| Güncelleme denetimi | Lisans etkinleştirme | ✅ GitHub sürüm denetimi |

### 1.13 Yedekleme, geri alma, arşiv ve sürüm geçmişi

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Geri al / yinele | ✅ Ayrıca "Undo Last" ile son malzeme değişikliği | ✅ `QUndoStack` tabanlı (`undoRedo/`) |
| Otomatik yedek ve geri yükleme | ✅ | ◐ Elle yedekleme ve geri yükleme (veritabanı dosyası) |
| ZIP ile tam yedek | ✅ Tablo seçerek geri yükleme | ❌ |
| Tarif arşivi | ✅ Ekleme, düzenleme, silme ve taşıma ayrı ayrı açılabiliyor; saklama süresi ve sayı sınırı; ayrı veritabanı | ❌ |
| Sürüm geçmişi | ◐ `F_R_VERSION` sayısı, Brew Log kopyası, "bump version" | ✅ Otomatik snapshot (`ancestor_id`), Ancestors diyaloğu |
| Silme | Kalıcı (arşive kopya alınabiliyor) | Yumuşak silme (`deleted` bayrağı) |

### 1.14 Birimler, dil ve platform

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Birim sistemleri | Metric, US (English), Imperial hazır ayarları; her nicelik için ayrı birim ve hassasiyet (`Units.*`) | Imperial, US Customary, Metric, MetricAlternate; SRM/EBC/Lovibond; SG/Plato/Brix; alan bazında birim değiştirme |
| İç birim | İngiliz birimleri (fl oz, °F, oz) | SI (L, kg, °C) |
| Para birimi | ✅ Sembol, konum ve hassasiyet | ✅ `CurrencyAmount` |
| Dil desteği | Değiştirilebilir dil SQLite dosyası (paketle yalnızca İngilizce geliyor) | 29 Qt `.ts` dosyası (Türkçe dahil), çalışırken dil değiştirme |
| Platformlar | Windows, macOS (Intel + Apple Silicon), Linux (wxWidgets) | Windows, macOS, Linux (deb, rpm, AppImage, Flatpak) |
| Mobil ve web | Bu pakette yok | Yok |

### 1.15 Eklentiler ve arayüz kolaylıkları

| Özellik | BeerSmith | Brewtarget |
|---|---|---|
| Add-on (veri paketleri: hop, maya, tahıl, misc, su, stil, ekipman, mash, karbonasyon, fermentasyon) | ✅ | ❌ (varsayılan içerik veritabanına yükleniyor) |
| Kod eklentisi | ❌ | ❌ |
| Sekmeli ve pencereli görünüm, önizleme paneli | ✅ | ✅ Ağaç, sekmeler, katalog pencereleri |
| Favoriler ve yer imleri çubuğu | ✅ | ❌ |
| Gelişmiş arama (malzemeye göre) | ✅ | ◐ Ağaç filtresi |
| Özelleştirilebilir sütunlar | ✅ | ✅ Sütun gizleme |
| Klasörler | ✅ Tarifler için | ✅ Her tür için birinci sınıf varlık (5.1.0), sürükle-bırak |
| Aynı kayıtları birleştirme | ❌ | ✅ (4.2.0) |
| Bir profilin kaç tarifte kullanıldığı | ❌ | ✅ `numRecipesUsedIn` |
| Karşılama ekranı ve başlangıç rehberleri | ✅ | ❌ (PDF kılavuz var) |

---

## 2. Veri modeli karşılaştırması

### 2.1 Genel yaklaşım

| Konu | BeerSmith | Brewtarget |
|---|---|---|
| Model tipi | Belge yönelimli. Tarif tek bir geniş satır; alt nesneler JSON metni olarak gömülü. | İlişkisel ve normalize. Her alt nesne ayrı tablo; yabancı anahtarlarla bağlı. |
| Tablo sayısı | 62. Bunların 11'i ana varlık, 10'u bulut ikizi; ayrıca klasör, alışveriş, envanter ve tarif için bulut tabloları, 29 araç tablosu, arama ayarları ve not tablosu. | Şema v21. 11 `folder_for_*` tablosu, 13 varlık ve adım tablosu, 4 `*_in_recipe` tablosu, `brew_log`, `instruction`, 4 `*_stock_purchase`, 4 `*_stock_use` tablosu ve ayarlar. |
| Birincil anahtar | `_PERMID_` (INTEGER) | `id` (INTEGER) |
| Ortak alanlar | `_PERMID_`, `_MOD_` (epoch metni), `_CLOUDID_`, `_CLOUD_STATE_`, `_EXTRA_` | `id`, `name`, `deleted` |
| Yabancı anahtar ve index | Yok | Var |
| Sayıların saklanması | SQL sütunlarında doğal tip; JSON içinde 7 ondalıklı metin (`"640.0000000"`) | SI birimiyle `REAL` |
| Enum'lar | Tam sayı kodu (`F_H_USE = 0..`) | Metin eşlemesi (`EnumStringMapping`); BeerJSON enum değerleriyle aynı |

### 2.2 Alan sayıları (temsilî)

Brewtarget sayıları `ObjectStoreTyped.cpp`'den otomatik sayıldı ve ortak alanları (`id`, `name`, `deleted`) içeriyor. ±2 sapma olabilir.

| Varlık | BeerSmith | Brewtarget |
|---|---|---|
| Tarif | 112 (`M_RECIPE`; 7 gömülü JSON sütunu dahil) | ≈36 (`recipe`) + 33 (`brew_log`) + eklemeler |
| Hop | 26 | ≈33 (+ `hop_in_recipe` ≈13) |
| Fermentable | 33 | ≈38 (+ `fermentable_in_recipe` ≈13) |
| Maya | 38 | ≈24 (+ `yeast_in_recipe` ≈16) |
| Misc | 22 | ≈11 (+ `misc_in_recipe` ≈13) |
| Su | 26 | ≈20 |
| Stil | 29 | ≈29 |
| Ekipman | 34 | ≈53 |
| Mash ve adım | 26 + adım JSON'u (yaklaşık 18 alan) | 12 + 16 |
| Fermentasyon | 24 (`M_AGE`, sabit 5 aşama) | 6 + adım başına 15 |
| Alan tanımı metaverisi | 1593 `Field` satırı (etiket, açıklama, birim, ipucu) ve 219 `Option` satırı | 532 `Q_PROPERTY` (model sınıflarında) ve `TypeLookup` |

### 2.3 Tarif ile malzeme arasındaki bağ

**BeerSmith malzemeyi tarifin içine kopyalıyor.**

- `M_RECIPE.Ingredients` sütunu bir JSON dizisi tutuyor. Dizideki her öğe, kütüphane satırının tam kopyası. `_Schema_` kodu türü belirtiyor: 7406 tahıl, 7403 hop, 7421 misc, 7426 maya, 7423 su.
- Kopyada kütüphane kaydının ID'si yok. Bağ yalnızca isim üzerinden. "Update Prices" işleminin hata mesajı (`S_ERR_PRICE_MATCH`) da eşleştirmenin isimle yapıldığını gösteriyor.
- Ekipman, stil, mash (adımları dahil), karbonasyon, fermentasyon profili ve baz tahıl da tek tek JSON kopyası olarak tutuluyor (`F_R_EQUIPMENT`, `F_R_STYLE`, `F_R_MASH`…).
- Kütüphane tablosu ile tarifteki kopya aynı şemayı kullanıyor. Örneğin `M_HOPS` içinde `F_H_AMOUNT`, `F_H_BOIL_TIME`, `F_H_IN_RECIPE` ve `F_ORDER` gibi tarife özgü alanlar da var.

**Brewtarget malzemeye referans veriyor.**

- Bağ `RecipeAdditionHop`, `RecipeAdditionFermentable`, `RecipeAdditionMisc` ve `RecipeAdditionYeast` ara tablolarıyla kuruluyor (`hop_in_recipe` vb.).
- Bu tablolar `recipe_id` ve `hop_id` anahtarlarını, miktarı (`quantity`, `unit`) ve zamanlamayı tutuyor: `stage` (mash, boil, fermentation, packaging), `step`, `add_at_time_mins`, `add_at_gravity_sg`, `add_at_acidity_ph`, `duration_mins`.
- Malzeme verisi kopyalanmıyor. Kaynak koddaki yorum bunun bilinçli bir karar olduğunu söylüyor: BeerJSON'un `HopAdditionType` tipinin aksine, `Hop` sınıfındaki bilgi tekrarlanmıyor.
- Mash, kaynatma, fermentasyon, stil, ekipman ve su profilleri de paylaşılan kayıtlara yabancı anahtarla bağlı (`mash_id`, `boil_id`, `water_base`, `water_target` vb.).
- Su, bir "ekleme" olarak modellenmemiş. Tarifte bir baz su ve bir hedef su tutuluyor; mash ve sparge için RO yüzdesi de var.

**İki yaklaşımın sonuçları**

| | Kopya (BeerSmith) | Referans (Brewtarget) |
|---|---|---|
| Tarifin tarihsel doğruluğu | ✅ Kütüphanede alfa değeri değişse de eski tarif değişmez | ❌ Kütüphanedeki hop düzenlenince eski tarifler ve snapshot'lar da yeniden hesaplanır |
| Tarifin taşınabilirliği | ✅ Tarif kendi kendine yeterli; senkron ve dışa aktarma kolay | ◐ Dışa aktarırken bağlı kayıtlar toplanmalı (DotBeer referansları dosya içi ID'yle çözüyor) |
| Veri tekrarı | ❌ Aynı hop onlarca kez kopyalanıyor | ✅ Normalize |
| Sorgulama ("bu hop hangi tariflerde var?") | ❌ JSON taramak gerekiyor; bağ isimle | ✅ Basit bir join; `numRecipesUsedIn` |
| Envantere bağlama | ◐ İsim eşleştirmesiyle | ✅ Yabancı anahtar ve lot |
| Şema değişikliği | `_Schema_` sürüm koduyla esnek | Göç kodu gerekiyor (`DatabaseSchemaHelper`, v1'den v21'e) |

### 2.4 Tarif içindeki adımlar

| | BeerSmith | Brewtarget |
|---|---|---|
| Mash adımları | `M_MASH.steps` içinde JSON dizisi (`F_MS_TYPE`, sıcaklık, süre, rampa, infüzyon/dekoksiyon miktarı) | `mash_step` tablosu; sıralı, `mash_id` ile bağlı |
| Kaynatma adımları | Yok (kaynatma süresi ekipman ve tarifte) | `boil_step` tablosu (ön ısıtma, kaynatma, soğutma) |
| Fermentasyon | `M_AGE` içinde sabit sütunlar (`PRIM`, `SEC`, `TERT`, `BULK`, `AGE` için sıcaklık ve gün) | `fermentation_step` tablosu, sınırsız adım |
| Malzemenin hangi adıma eklendiği | Kullanım türüne göre (`F_H_USE`, `F_H_DRY_START`, `F_H_DRY_PHASE`) | `stage` + `step` + zaman / gravite / pH koşulu |
| Talimatlar | Rapor şablonunda üretiliyor (kalıcı değil) | `instruction` tablosu (kalıcı, tarife bağlı) |

### 2.5 Demleme kayıtları ve ölçümler

- **BeerSmith:** Ölçümler tarif satırındaki sütunlarda tutuluyor. Her ölçüm bir değer ve bir "girildi mi" bayrağından oluşuyor (`F_R_OG_MEASURED` + `F_R_OG_MEASURED_SET`). Birden fazla demleme kaydı için tarifin tamamı Brew Log klasörüne kopyalanıyor. Fermentasyon okumaları da tarifin içinde (`AgeData` JSON'u: `Date`, `F_AD_TEMP`, `F_AD_PH`, gravite ve kontrol bayrakları).
- **Brewtarget:** `brew_log` tablosu `recipe_id` ile tarife bağlı ve her ölçüm için beklenen/ölçülen/hesaplanan üçlüsünü tutuyor (`expected_original_gravity_sg`, `measured_original_gravity_sg`, `computed_efficiency_pct` vb.). Fermentasyon okumaları için bir tablo yok.

### 2.6 Profiller

Profillerin ikisi de kütüphanede yaşıyor. Fark tarife bağlanma biçiminde: BeerSmith seçilen profili tarife kopyalıyor, Brewtarget referans veriyor. Bunun bir sonucu: Brewtarget'ta bir mash profilini düzenlemek, onu kullanan bütün tarifleri etkiliyor. Bu yüzden editör kaç tarifin etkileneceğini gösteriyor (4.1.0, #948).

### 2.7 Envanter

- **BeerSmith:** İki yerde tutuluyor.
  1. Her malzeme satırında bir sayı alanı (`F_H_INVENTORY` vb.).
  2. Ayrı `M_INVENTORY` ve `M_SHOP` tabloları (`F_SH_NAME`, `F_SH_AMOUNT`, `F_SH_TYPE`, `F_SH_ING_ID`, fiyat ve sipariş eşiği). İki tablonun şeması aynı.

  Sürüm notlarına göre 4.x'te envanter malzeme tablolarından ayrıldı.
- **Brewtarget:** Bir hareket defteri gibi çalışıyor. `*_stock_purchase` her alışı tutuyor: miktar, sipariş, teslim ve son kullanma tarihleri, tedarikçi, fiyat ve hop'a özgü alfa, form ve yıl. `*_stock_use` her kullanımı tutuyor: sıra, tarih, sebep, miktar ve `brew_log_id`. Kalan stok bu hareketlerden hesaplanıyor. Lot bazlı FIFO, fiyat geçmişi ve demlemeyle izlenebilirlik bu sayede mümkün.

### 2.8 Klasörler

- **BeerSmith:** `M_FOLDER` (`F_F_NAME`, `F_F_PARENT`, `F_F_PRIVATE`). Tarif hem `F_R_PARENT` (klasör ID'si) hem de `F_R_FOLDER_NAME` (metin yolu, örneğin `/BeerSmith 3 Samples/`) tutuyor; yani bilgi tekrarlanıyor. Klasör yalnızca tarifler için var.
- **Brewtarget:** Her varlık türü için ayrı bir `folder_for_<tür>` tablosu var (`contained_in_folder_id` ile iç içe). Varlıkta da `contained_in_folder_id` sütunu var. Tabloların ayrı olması bilinçli: hop'un fermentable klasörüne konmasını şema düzeyinde engelliyor. BeerXML'e `FOLDER_PATH` uzantı etiketiyle yazılıyor.

### 2.9 Sürümleme

> **Düzeltme:** BeerSmith'teki `F_R_PARENT` alanı sürümle ilgili değil. `Field` tablosundaki tanımı "Parent folder id used for organizing recipes"; yani bir **klasör** referansı. Örnek veride bütün tariflerde değeri 1 ve bu, `M_FOLDER` tablosundaki tek klasörün ID'si.

**BeerSmith'in sürümleme mekanizmaları:**

1. `F_R_VERSION`: Elle girilen bir sayı. İsteğe bağlı olarak sürüm değişince "farklı isimle kaydet" önerisi çıkıyor (`FO_CONFIRM_VERSION_CHANGE`).
2. New Session diyaloğu: tarifin bir kopyasını Brew Log'a (yerel ya da bulut) alıyor ve sürümü artırıyor (`F_R_ARCHIVE`, `F_R_BUMP_VERSION`).
3. Recipe Archive: ekleme, düzenleme, silme ve taşıma işlemlerinin her biri için ayrı açılıp kapatılabiliyor. Ay ve adet sınırı var, arşivden kurtarma yapılabiliyor ve arşiv ayrı bir veritabanında tutuluyor.

Kopyalar arasında bir soy bağı (parent ID) tutulmuyor.

**Brewtarget'ın sürümleme mekanizması:**

- `recipe.ancestor_id` aynı tabloya işaret eden bir yabancı anahtar; kökte kaydın kendi ID'si.
- Tarif demlendikten sonra yapılan bir değişiklik otomatik snapshot alıyor (copy-on-write). Eski sürüm kilitleniyor ve ağaçta "subsidiary" olarak gizleniyor.
- Dal (branch) yok; doğrusal bir zincir. Elle fork alma, kilitleme ve ata atama (`AncestorDialog`) mümkün.
- Seçeneklerden kapatılabiliyor. Tarif silinince snapshot'ların da silinip silinmeyeceği seçilebiliyor.
- Zayıf nokta: snapshot, malzemeleri referansla tuttuğu için geçmiş tam olarak dondurulmuyor (bkz. §2.3).

### 2.10 Bulut alanları (yalnızca BeerSmith)

| Alan | Anlamı (alan tanımından ve kullanımdan) |
|---|---|
| `_PERMID_` | Yerel kalıcı ID (INTEGER PK) |
| `_CLOUDID_` | Sunucudaki karşılığın ID'si (0 = senkronize edilmemiş) |
| `_MOD_` | Son değişiklik zamanı (epoch, metin) |
| `_CLOUD_STATE_` | Senkron durumu |
| `_EXTRA_` | Ayrılmış alan |
| `M_CLOUD_*` | Buluttan inen kayıtların ayrı ikiz tabloları (aynı şema) |
| `F_C_PRIVATE`, `F_C_UNLISTED`, `F_C_RATING`, `F_C_COMMENTCOUNT` | Paylaşım ve topluluk alanları |
| `F_R_JSON` | "JSON version of the recipe – used for cloud sync". Sunucuyla alışveriş JSON ile, ama özel bir şemayla. |

Brewtarget'ta bunların hiçbirinin karşılığı yok. Kayıtların genel bir kimliği (UUID) ya da değişiklik zamanı tutulmuyor.

### 2.11 Güçlü ve zayıf yanlar

| | BeerSmith | Brewtarget |
|---|---|---|
| **Güçlü** | Tarif kendi kendine yeterli ve tarihsel olarak donmuş. Senkron alanları baştan tasarlanmış. Arşiv ayrıntılı. Araç girdileri kalıcı. Özel alanlar var. Fermentasyon okumaları var. | Normalize ve tutarlı. Eklemelerin zamanlaması BeerJSON'la uyumlu (stage, step, gravite, pH). Lot bazlı envanter. Profiller adım adım ve sınırsız. Soy zinciri var. Her tür için klasör. Yumuşak silme. Hem SQLite hem PostgreSQL. |
| **Zayıf** | 112 sütunlu "tanrı tablosu". JSON içinde sayılar metin. Kopyalar isimle bağlı. Yabancı anahtar ve index yok. Ölçümler tarifte; birden fazla demleme kaydı için tarifin tamamı kopyalanıyor. Fermentasyon 5 sabit aşama. İç birim İngiliz. `F_R_FOLDER_NAME` ile `F_R_PARENT` tekrar ediyor. | Kütüphanede yapılan bir düzenleme geçmiş tarifleri ve snapshot'ları değiştiriyor. Senkron ve UUID yok. Fermentasyon okuması yok. Maliyet modeli zayıf. Çok tablolu ve şablon ağırlıklı C++ kodu, değişiklik maliyetini artırıyor (21 şema göçü). |

---

## 3. Mimari

| Konu | BeerSmith | Brewtarget |
|---|---|---|
| Dil ve arayüz | C++ ve wxWidgets (Windows paketindeki lisans dosyasından) | C++20 ve Qt6 (Widgets, `.ui` dosyaları) |
| Depolama | Yerel SQLite (`BeerSmith.sqlite`), varsayılan tarif için `DefRecipe.sqlite`, ayrı arşiv veritabanı ve bulut sunucusu. Sürüm notlarına göre her kayıt anında commit ediliyor. | SQLite veya PostgreSQL (`QSQLITE` / `QPSQL`; seçeneklerde motor seçimi ve bağlantı testi). ObjectStore önbelleği; okumalar önbellekten, yazmalar anında. |
| Şema tanımı | Tablo başına sütunlar ve gömülü JSON'da `_Schema_` kodları | Kodda `TableDefinition` / `TableField` ile tanımlı; göçler `DatabaseSchemaHelper` içinde (v21) |
| Alan tanımları | **Veri olarak** tutuluyor: `Resources.sqlite.Field` her alan için etiket, açıklama, `Column_Name`, birim ve ipucu içeriyor; `Option` her ayar için varsayılan değer ve tipi; `Menu` her komut için metin, ikon, yardım ve ipucunu; `String` ise mesajları. Arayüz bu tablolardan üretiliyor. | **Kod olarak** tutuluyor: Qt property sistemi (`Q_PROPERTY`, `PropertyNames::*` sabitleri, `TypeLookup` ile birim ve nicelik tipi). Editörler `EditorBaseField` ile property'lere bağlanıyor. Etiketler `tr()` ile `localisedName_*` fonksiyonlarında. |
| Serileştirme | BeerXML ve `.bsmx` metin şablonuyla (`Recipe.xml` içinde `$NAME`); bulut için JSON | Bildirimsel eşleme tabloları: `XmlRecordDefinition` (BeerXML'de yaklaşık 466 alan eşlemesi), `JsonRecordDefinition` (BeerJSON'da yaklaşık 310, DotBeer'de yaklaşık 314); şema doğrulamalı (libxml2, Boost.JSON) |
| Raporlama | Şablon motoru: HTML ve TXT şablonları, `$VAR` ve `$STYLE.OG_RANGE` gibi noktalı yollar, CSS teması; kullanıcı kendi şablonunu ekleyebiliyor | Kodla üretilen HTML (Formatter sınıfları); özelleştirilemiyor |
| Çeviri | Bütün metinler SQLite'ta; dil dosyası alternatif bir SQLite olarak yükleniyor; teknik olmayan çevirmen için kolay | Qt Linguist `.ts` / `.qm`, 29 dil; çalışırken dil değişimi; çevirmen araç zinciri olgun |
| Undo/redo | Menüde var (`M_UNDO`, `M_REDO`, `M_R_UNDO`); klasör işlemleri için `F_F_OLD_ID` var; nasıl uygulandığı bilinmiyor | Komut deseni: `SimpleUndoableUpdate`, `UndoableAddOrRemove(List)`, `RelationalUndoableUpdate`; `QUndoStack` |
| Test kapsamı | Görünür bir test yok (kapalı kaynak) | QtTest, yaklaşık 2.900 satır, yaklaşık 33 test fonksiyonu: birim dönüşümü, all-grain hesabı, kaynatma zamanlaması, envanter, mash hacimleri, buharlaşma, ekipman hacimleri, BeerJSON gidiş-dönüş, BeerXML null değerler, DotBeer içe/dışa aktarma, klasör yolları. Hesap testleri sınırlı: tüm IBU ve renk yöntemleri test edilmiyor. |
| Dağıtım | NSIS (Windows), DMG (macOS); lisans etkinleştirme | CMake ve Meson; deb, rpm, AppImage, Flatpak, Windows, macOS |

**Çıkarım:** BeerSmith'in "alan tanımı veri olarak tutulur" yaklaşımı, çeviri, ipucu, birim etiketi, rapor değişkeni ve form üretimini tek bir kaynaktan besliyor. Brewtarget'ın property sistemi ise tip güvenliği ve serileştirme eşlemesi açısından güçlü. Yeni bir uygulamada iki yaklaşım birleştirilebilir: alan kataloğu (ID, tip, nicelik türü, iç birim, etiket anahtarı, BeerJSON yolu, BeerXML etiketi) tek bir bildirimsel şemada tutulur; tipler ve formlar bu şemadan üretilir.

---

## 4. Standartlarla uyum

### 4.1 Destek özeti

| Standart | BeerSmith | Brewtarget |
|---|---|---|
| BeerXML 1.0 | İçe ve dışa aktarma. Dışa aktarma, Reports klasöründeki `.xml` şablonlarıyla yapılıyor. Şablon bulunamazsa hata veriyor (`S_ERR_XML_TEMPLATE`). Bazı tablo türleri dışa aktarılamıyor (`S_ERR_XML_TYPE`). | İçe ve dışa aktarma; XSD (`schemas/beerxml/v1`). |
| BeerJSON 1.0 | **Yok.** Menü, metin ve alan tablolarında "json" yalnızca bir yerde geçiyor: bulut senkronu için olan `F_R_JSON`. | Var (`schemas/beerjson/1.0`). Veri modeli zaten BeerJSON'a göre yeniden yazılmış (enum'lar, eklemeler, boil ve fermentation profilleri). |
| DotBeer 1.0.0 | Yok | Var |

### 4.2 Alan eşlemesi (seçme)

| Kavram | BeerXML etiketi | BeerJSON | BeerSmith alanı | Brewtarget property'si |
|---|---|---|---|---|
| Hop alfa | `HOP/ALPHA` | `alpha_acid` | `F_H_ALPHA` | `Hop::alpha_pct` |
| Hop kullanım ve zamanlama | `USE`, `TIME` | `timing.use`, `timing.time`, `timing.duration` | `F_H_USE`, `F_H_BOIL_TIME`, `F_H_DRY_HOP_TIME`, `F_H_DRY_START` | `RecipeAddition::stage`, `step`, `addAtTime_mins`, `duration_mins` |
| Whirlpool sıcaklığı | – | `timing` + `temperature` (adım düzeyinde) | `F_H_WHIRLPOOL_TEMP` | – |
| HSI | `HSI` | `percent_lost` | `F_H_HSI` | `sixMonthAlphaLoss_pct` |
| Hop yağları | `MYRCENE` vb. (BeerJSON uzantısı) | `oil_content` | – | ✅ 13 yağ alanı |
| Fermentable verimi | `YIELD` | `yield.fine_grind` vb. | `F_G_YIELD` | `fineGrindYield_pct` vb. |
| Maya attenuation | `ATTENUATION` | `attenuation_range` | `F_Y_MIN_ATTENUATION`, `F_Y_MAX_ATTENUATION` | min/max |
| Mash adımı | `MASH_STEP` | `MashStepType` | `steps` JSON'u | `MashStep` |
| Kaynatma adımı | – | `BoilProcedureType` | – | `BoilStep` |
| Fermentasyon adımı | `PRIMARY_AGE`, `PRIMARY_TEMP`… (sabit) | `FermentationStepType[]` | `M_AGE` sütunları | `FermentationStep` |
| Ekipman | `EQUIPMENT` (tek nesne) | `equipment_items[]` (kap kap) | `M_EQUIPMENT` (tek nesne) | `Equipment` (kap bazlı alanlar) |
| Envanter | `INVENTORY` (Appendix A, metin) | `inventory` | `F_*_INVENTORY` | `StockPurchase` (BeerJSON'daki `inventory` yerine) |
| Ölçümler | `OG`, `FG` (gerçekleşen) | `BatchType` 1.0'da yok | `F_R_*_MEASURED` | `BrewLog` |
| Klasör | – | – | `F_R_FOLDER_NAME` | `FOLDER_PATH` uzantı etiketi |

**Dikkat edilecek noktalar:**

- BeerSmith'in BeerXML şablonları `DISPLAY_*` ve `INVENTORY` gibi Appendix A alanlarını yazıyor. Whirlpool sıcaklığı, kuru hop fazı, pitch rate, su hedefi ve fermentasyon okumaları gibi BeerSmith'e özgü alanlar BeerXML'e aktarılmıyor.
- Brewtarget bu `DISPLAY_*` etiketlerini okurken yok sayıyor (eşlemede `NULL_STR`). BeerXML'de karşılığı olmayan BeerJSON alanlarını ise uzantı etiketi olarak yazıyor.
- BeerXML'deki 3 aşamalı fermentasyon ile BeerJSON'daki N adımlı fermentasyon arasında gidiş-dönüş kayıpsız değil.
- BeerSmith'ten veri taşımak için tek ortak format BeerXML. Bu yüzden kendi uygulamanızda **BeerXML içe aktarma zorunlu**. Bunun yanında `.bsmx` içe aktarma da düşünülebilir; `.bsmx`, BeerXML benzeri XML ve kendi alan adlarını kullanıyor.

---

## 5. Kendi uygulamanız için öneriler

### 5.1 MVP kapsamı

| Öncelik | Özellik | Neden |
|---|---|---|
| **MVP** | Tarif editörü: bira türleri, OG/FG/ABV/IBU (Tinseth ve Rager; Garetz sonra), renk (Morey), stil göstergesi | İki programın da çekirdeği |
| **MVP** | Tersine ayar: hedef OG, hedef IBU, hedef renk ve yüzdeyle tahıl | BeerSmith'in en belirgin üstünlüğü; Brewtarget'ta yok |
| **MVP** | Ekipman profili ve hacim zinciri (`formuller/05`), mash profili ve strike hesabı | Doğru hacim olmadan hesaplar anlamsız |
| **MVP** | Brew day ekranı: talimat listesi, zamanlayıcılar (mash, kaynatma, whirlpool) ve ölçüm girişi ile verim hesabı (ayrı Batch kaydı olarak) | Mobil ve web için en değerli ekran |
| **MVP** | BeerXML içe ve dışa aktarma, BeerJSON içe ve dışa aktarma | Kullanıcıları taşımak ve gelecekle uyum |
| **MVP** | Temel araçlar: ABV, hidrometre sıcaklık düzeltmesi, refraktometre, priming/karbonasyon, birim dönüştürücü | Düşük maliyet, yüksek kullanım |
| **MVP** | Metrik ve İngiliz birimleri, i18n (TR ve EN) | Hedef kitle için |
| **MVP** | Undo/redo ve otomatik kayıt | İki programda da var; beklenen bir özellik |
| v1.1 | Su kimyası, tuz eşleştirme, mash pH (asit dahil; molar kütleyle doğru faktörler) | BeerSmith bile yıllarca hatalı faktör kullanmış; dikkatli ve testli yapılmalı |
| v1.1 | Maya: canlılık, starter, pitch rate | |
| v1.1 | Fermentasyon okumaları, grafik, CSV; Tilt, iSpindel ve RAPT köprüsü | BeerSmith yalnızca Tilt'i destekliyor, Brewtarget hiçbirini; bu bir farklılaşma alanı |
| v1.1 | Lot bazlı envanter (Brewtarget modeli), alışveriş listesi ve tarif maliyeti (BeerSmith modeli) | İki programın en iyi taraflarının birleşimi |
| v1.2 | Mead, cider ve seltzer modülü (besin, sülfit, backsweeten, sanal OG) | BeerSmith'te var, Brewtarget'ta yok |
| v1.2 | Bulut senkronu, paylaşım, puanlama ve yorum, topluluk araması | |
| v1.2 | Kullanıcı rapor şablonları, takvim ve görev listesi, parti-gyle, dilution, boil-off araçları | |

### 5.2 Önerilen veri modeli

İki programın en iyi fikirleri şöyle birleştirilebilir:

1. **Kütüphane malzemesi ile tarif eklemesi ayrı tutulsun, ama ekleme bir anlık görüntü (snapshot) taşısın.**
   - `ingredient` (kütüphane): `id` (UUID), tür, kimlik alanları ve varsayılan özellikler.
   - `recipe_addition`:
     - `recipe_id` ve `ingredient_id` (nullable referans; Brewtarget'taki gibi);
     - hesaplamayı etkileyen özelliklerin anlık kopyası (alfa, yield, renk, attenuation; BeerJSON `HopAdditionType` yaklaşımı ve BeerSmith'in tarihsel doğruluğu);
     - miktar ve birim;
     - zamanlama: `stage`, `step_id`, `time`, `duration`, `temperature`, `at_gravity`, `at_ph`.
   - Kütüphanede bir değişiklik yapıldığında "tariflerdeki kopyaları güncelle" seçeneği sunulsun.
2. **Profiller tarife kopyalanarak eklensin; adımlar ayrı tablolarda dursun.**
   - Mash, boil ve fermentation profilleri adım tablolarıyla tutulsun (Brewtarget, BeerJSON).
   - Tarife eklenen profil `source_profile_id` ile kopyalansın; böylece tarif kendi kendine yeterli olur.
3. **Demleme kaydı (Batch) tariften ayrı bir varlık olsun.** Tarif sürümünün dondurulmuş bir kopyasını ya da `recipe_version_id` referansını taşısın. Ölçümler beklenen/ölçülen çiftleri olarak tutulsun (Brewtarget `BrewLog`).
   - Fermentasyon okumaları ayrı bir `batch_reading` tablosunda tutulsun: zaman, SG, sıcaklık, pH, kaynak cihaz, ham veri (BeerSmith `AgeData`).
4. **Sürümleme:** `recipe_version` tablosu (`recipe_id`, `version_no`, `parent_version_id`, `created_at`, `snapshot_json`). Brewtarget'ın ata zinciri ile BeerSmith'in arşiv ayarları birleşebilir. Demleme yapılmış bir sürüm değişmez olsun.
5. **Senkron alanları her tabloda bulunsun:** `id` (UUID; BeerSmith'teki `_PERMID_` / `_CLOUDID_` ikiliği yerine tek ve global bir kimlik), `created_at`, `updated_at`, `deleted_at` (yumuşak silme), `version` / `etag`, `sync_state`. BeerSmith'teki gibi ayrı `M_CLOUD_*` ikiz tabloları kullanılmasın; sahiplik ve görünürlük alanlarıyla tek tablo yeterli.
6. **Envanter bir hareket defteri gibi tutulsun:** `stock_lot` (alış, fiyat, tarih, lot özellikleri) ve `stock_movement` (kullanım veya düzeltme, `batch_id`). Alışveriş listesi tariften ve eşiklerden türetilsin.
7. **Her tür için klasör ya da etiket** (Brewtarget); tek bir yol kaynağı tutulsun, metin yolu tekrarlanmasın.
8. **İç birimler SI olsun** (Brewtarget). Görüntüleme birimi kullanıcı tercihine bağlansın. Sayılar metin olarak tutulmasın.
9. **Alan kataloğu veri olarak tutulsun** (BeerSmith `Field` / `Option` fikri): her alan için ID, tip, nicelik türü, varsayılan değer, min/max, i18n anahtarı, BeerJSON yolu ve BeerXML etiketi. Formlar, doğrulama, rapor değişkenleri ve içe/dışa aktarma eşlemesi bu katalogdan üretilsin.
10. **Özel alanlar** (BeerSmith) için `custom_field_def` ve `custom_field_value` tabloları (ya da JSONB sütunu) kullanılsın.
11. **Araç girdileri** (BeerSmith `M_TOOL_*`) kullanıcı tercihleri altında tek bir JSON belgesinde tutulsun; 29 ayrı tabloya gerek yok.

### 5.3 Standartlar

- **Varsayılan iç model BeerJSON 1.0'a yakın tasarlansın.** Enum değerleri BeerJSON'dakilerle aynı olsun (Brewtarget de böyle yapıyor). Tarifin dışa aktarımı doğrudan BeerJSON olsun.
- **BeerXML 1.0 içe aktarma zorunlu** (BeerSmith ve eski araçlardan taşıma). Dışa aktarımda Appendix A'daki `DISPLAY_*` alanları da yazılsın; kendi alanlarınız uzantı etiketi olarak eklensin.
- **DotBeer** (referanslı JSON, MPL 2.0 lisanslı şema), yedekleme ve tam veritabanı dışa aktarımı için örnek alınabilir. Bu format malzemeleri ve stok alımlarını ID ile taşıyor.
- **Cihaz verisi:** Tilt (CSV ve Google Sheets), iSpindel (HTTP/JSON) ve RAPT API'si için bir adaptör katmanı kurulsun. Bu alanda iki programın da standardı yok.
- **Hesap motoru:** `/home/user/formuller/*.md` içindeki test vektörleriyle birim testleri yazılsın. Brewtarget'ın test yapısı (hesap, serileştirme gidiş-dönüş testleri ve birim dönüşümü) örnek alınabilir, ama kapsamı daha geniş tutulmalı: her IBU ve renk yöntemi, su kimyası ve hacim zinciri.

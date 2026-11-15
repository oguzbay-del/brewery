# Demleme

Ev biracıları ve küçük bira fabrikaları için iPad, iPhone ve web'de çalışacak tarif ve üretim uygulamasının plan ve tasarım çalışması. Geliştirme sürüyor; durum için `gelistirme/DURUM.md`.

| Klasör | İçerik |
| --- | --- |
| `plan/` | Ürün planının Markdown kopyası (asıl kaynak: [Claude Docs](https://claude.ai/artifact/1pUDt9xJQHdj2WxhMphDQy); kopya doğrudan düzenlenmez) ve kullanıcı görüşmesi kılavuzu (`gorusme.md`) |
| `arayuz/` | Tıklanabilir arayüz konsepti, tek HTML dosyası ([yayın](https://claude.ai/artifact/BCEHj3wb6xj7eShvjTxKn6)) |
| `tasarim-sistemi/` | Renkler, yazı ölçeği, boşluklar ve bileşenler ([yayın](https://claude.ai/artifact/ATVvgzfCftjr9YzVjhSe28)) |
| `formuller/` | Hesap motoru formül spesifikasyonları ve Python test vektörleri |
| `erisilebilirlik/` | Erişilebilirlik incelemeleri (WCAG 2.2 AA) |
| `karsilastirma/` | BeerSmith ve Brewtarget hesaplarının karşılaştırması; FoamByte ile özellik, hesaplayıcı, SSS ve site karşılaştırmaları (`foambyte-*.md`) |
| `gelistirme/` | Geliştirme takibi: `ISLER.md` (iş listesi, hangi işi hangi model yapar, durum), `DURUM.md` (neredeyiz, sıradaki iş) |
| `araclar/` | `tutarlilik.py`: plan ile konsept arasında ekran sayıları, sekmeler, mod haritası, bağlantılar ve formül göndermeleri; test betiklerini çalıştırır, "doğrulanacak" sayar (`python3 -I araclar/tutarlilik.py`) |
| `CLAUDE.md` | Claude ile çalışırken uyulan kurallar |


## Pazarlama sitesi

`marketing-site/`, Demleme ürün uygulamasından ayrı olan ticari tanıtım ve self-service keşif sitesidir. Birincil hedef profesyonel bira üreticileri; Homebrewers ayrı ve ikincil bir yol olarak ele alınır. Site şu an ürünün kendisi değildir ve geliştirme durumunu açıkça belirtir.

Geliştirme, build, sosyal kart üretimi ve test komutları için [marketing-site/README.md](marketing-site/README.md).

Yayın bağlantıları özeldir; yalnız sahibine ve paylaşılan kişilere açılır.

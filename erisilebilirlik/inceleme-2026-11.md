# Erişilebilirlik incelemesi: arayüz konsepti ve tasarım sistemi

**Standart:** WCAG 2.2 AA · **Tarih:** 6 Kasım 2026
**Kapsam:** `arayuz/demleme-arayuz-konsepti.html` (57 sekme, açık ve koyu tema) ve `tasarim-sistemi/`
**Yöntem:** axe-core 4.10.2 ile her sekmede otomatik tarama, renk çiftlerinin elle hesaplanması, kod incelemesi. Ekran okuyucu ve gerçek cihaz testi yapılmadı.

## Özet

| | Önce | Sonra |
|---|---|---|
| Karşıtlık hatası, açık tema | 1.596 öge | 48 öge |
| Karşıtlık hatası, koyu tema | 57 öge | 57 öge (çoğu pasif öge) |
| ARIA ve yapı hataları | 68 | 13 (konsept sayfasının kendisinde, ürün tasarımını etkilemiyor) |

## Düzeltilenler

| # | Sorun | WCAG | Önem | Düzeltme |
|---|---|---|---|---|
| 1 | Açık temada ikincil yazı (`label2`, Apple %60) zeminlerde 3.3–3.6:1; bütün alt satırlar, değerler, bölüm başlıkları | 1.4.3 | Kritik | Opaklık %75: `card` 5.2:1, `bg` 4.8:1, `fill` 4.5:1 |
| 2 | Amber yazı (`tint` #A85F00) gri zeminde 4.4:1, düğme zemininde 3.9:1 | 1.4.3 | Büyük | #985600: 5.1:1 ve 4.6:1 |
| 3 | Durum renkleri (yeşil, turuncu, kırmızı) metin ve durum çiplerinde 3.5–4.4:1 | 1.4.3 | Büyük | Bir ton koyu: #1D7233, #A04800, #C40013 (hepsi düğme zemininde en az 4.7:1) |
| 4 | Operatör görünümünde alarm 2 ve 3 metin olarak 3.1–3.7:1 | 1.4.3, 1.4.11 | Büyük | #9A4D00 ve #086F86; alarm 2 rakamı için yeni `hp-a2-ink` |
| 5 | Odak halkası parlak amber, gri zeminde 2.2:1 | 2.4.7, 1.4.11 | Büyük | Mavi, 3 px (5.0:1) |
| 6 | Tema seçici düğmeleri `aria-selected` kullanıyordu | 4.1.2 | Büyük | `aria-pressed` |
| 7 | "Yeni" noktaları rolsüz `aria-label` taşıyordu (87 yer) | 4.1.2 | Küçük | `role="img"` |
| 8 | Sayfada `main` bölgesi ve dil bilgisi yoktu | 1.3.1, 3.1.1 | Küçük | `main` ve `lang="tr"` |

## Bilerek bırakılanlar

| Sorun | Neden |
|---|---|
| Soluk (`label3`) metinler: mod haritasında kapalı ekranlar, "yetki gerekir" satırları, bağlantı bekleyen hesap | Pasif ögeler; WCAG 1.4.3 pasif ögeleri muaf tutar. Durum ayrıca metinle yazılı. |
| Koyu temada beyaz anahtar topuzu yeşil zeminde 2.0:1 | iOS'un kendi anahtarıyla aynı; durum topuzun yerinden de anlaşılıyor. Native uygulamada sistem bileşeni kullanılacak. |
| Koyu temada renkli ikon kutucuklarında beyaz ikon | İkon süs; yanındaki hücre metni aynı şeyi söylüyor. |
| Kaynatma programı ve takvimdeki renkli şeritlerin üzerindeki 10–11 px etiketler (3.7–4.3:1) | Konseptteki örnek çizimler; uygulamada şerit etiketi şeridin dışında veya 13 px üstünde yazılmalı. |

## Konsept sayfasında kalan küçük işler

- 9 tablo başlığı boş (ilk sütun başlıkları); ekran okuyucu için gizli metin eklenebilir.
- İki örnek ekranda kaydırılabilir alan klavyeyle odaklanamıyor.
- Kağıt form örneklerinde başlık sırası atlıyor (h2'den h4'e).
- Sekmeler fareyle ve Tab tuşuyla çalışıyor, ok tuşlarıyla geçiş yok (WAI-ARIA sekme kalıbında beklenir).

## Ürün için kurallar (plana ve tasarım sistemine işlendi)

- Okunması gereken her metin iki temada da en az 4.5:1; büyük metin ve anlam taşıyan işaretler en az 3:1.
- Native uygulamada ikincil yazı için sistem rengi yerine tasarım sistemindeki değer, "Karşıtlığı Artır" varyantıyla asset kataloğunda tanımlanır.
- Durum hiçbir zaman yalnız renkle anlatılmaz: alarm işaretleri renk + şekil + numara, uyarılar ikon + metin.
- Dokunma alanı en az 44 px; Dinamik Yazı ve Hareketi Azalt desteklenir.

## Yapılmayan testler

- VoiceOver (iPhone, iPad, Mac) ile gerçek ekran okuyucu testi
- %200 yakınlaştırma ve en büyük Dinamik Yazı boyutunda düzen kontrolü
- Islak elle ve eldivenle dokunma testi (üretim ortamı)

Bunlar uygulama yazıldıktan sonra gerçek cihazda yapılmalı.

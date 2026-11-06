# SummaryCard

Bir sayıyı öne çıkaran kart (Sağlık uygulamasındaki özet kartı gibi).

- `card` zemin, `radius-lg`. Üstte ikon + etiket (`tint`, 13 px, 600), ortada değer (`t-metric`, SF Pro Rounded, eş genişlikli rakam), birim küçük ve `label2`, altta bir satır açıklama (`label2`).
- Tahmin her zaman aralıkla gösterilir ("±0.002"); veri yetmezse sayı yerine nedenini yaz.
- Birim kullanıcının bölge ayarından gelir (°C/°F, SG/°P); ondalık ayırıcı bölgeye göre (TR virgül).

Kullanan sağlar: etiket, değer, birim, açıklama, isteğe bağlı ikon.
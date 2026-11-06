# GroupedList

iOS gruplu listesi: bölüm başlığı (`.dm-ghead`) ve altında `card` zeminli bir grup (`.dm-group`) içinde hücreler (`.dm-cell`).

- Ayarlar, ayrıntı ekranları, sheet formları ve iPhone listelerinin temel yapısı.
- Hücre: solda başlık, isteğe bağlı alt satır (`.sub`, `label2`, 12 px), sağda değer (`.v`, eş genişlikli rakam) veya ok (`.chev`, `label3`, yalnız süs).
- Renkli ikon kutucuğu `.dm-badge` (`badge-size`, `radius-xs`); zemin bir durum veya vurgu rengi, üzerindeki ikon beyaz ya da `tint-fill` üzerinde `tint-ink`.
- Seçili satır `.sel` (`tint-soft`), eylem satırı `.link` (`tint` yazı).
- Ayırıcı `sep`, soldan 14 px içeride başlar. Hücre yüksekliği en az `tap-min`.
- Bölüm başlığı büyük harfle ve kısa ("KAP", "KARBONASYON").

Kullanan sağlar: satırlar, her satırın başlığı, değeri ve dokunma eylemi.
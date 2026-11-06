# AlarmMark

Operatör görünümünde (ISA-101) alarm işareti; renk, şekil ve numara aynı şeyi söyler.

- **1 Acil:** `hp-a1` kırmızı kare, beyaz "1". Onaylanana kadar yanıp söner (yalnız bu).
- **2 Uyarı:** `hp-a2` turuncu eşkenar dörtgen, rakam `hp-a2-ink` (açık temada beyaz, koyu temada koyu).
- **3 Bilgi:** `hp-a3` camgöbeği daire, beyaz "3".
- Operatör görünümünde normal durum renksizdir: zemin `hp-bg`, kutucuk `hp-tile`, yazı `hp-ink`/`hp-ink2`; renk yalnız sapmada.
- Değerin yanında ölçek (`.dm-hpscale`): ray `hp-track`, normal aralık `hp-band`, değer çizgisi `hp-ink`.
- Aynı üç seviye bildirim önceliklerinde de kullanılır (Acil, Uyarı, Bilgi).
- Açık temada üç alarm rengi de `hp-tile` üzerinde en az 4.5:1, `hp-bg` üzerinde en az 3.8:1; yanında her zaman numara ve metin var.

Kullanan sağlar: seviye (1–3) ve yanındaki metin.
# AlarmMark

Operatör görünümünde (ISA-101) alarm işareti; renk, şekil ve numara aynı şeyi söyler.

- **1 Acil:** `hp-a1` kırmızı kare, beyaz "1". Onaylanana kadar yanıp söner (yalnız bu).
- **2 Uyarı:** `hp-a2` turuncu eşkenar dörtgen, koyu "2".
- **3 Bilgi:** `hp-a3` camgöbeği daire, beyaz "3".
- Operatör görünümünde normal durum renksizdir: zemin `hp-bg`, kutucuk `hp-tile`, yazı `hp-ink`/`hp-ink2`; renk yalnız sapmada.
- Değerin yanında ölçek (`.dm-hpscale`): ray `hp-track`, normal aralık `hp-band`, değer çizgisi `hp-ink`.
- Aynı üç seviye bildirim önceliklerinde de kullanılır (Acil, Uyarı, Bilgi).
- Açık temada `hp-a2` ve `hp-a3` `hp-bg` üzerinde yaklaşık 3:1; işaret olarak sınırda, yanında her zaman numara ve metin var.

Kullanan sağlar: seviye (1–3) ve yanındaki metin.
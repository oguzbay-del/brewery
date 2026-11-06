# NavBar

Ekranın üstündeki gezinme çubuğu; kullanıcıya nerede olduğunu ve nasıl döneceğini söyler.

- **İleri itilen ekran:** solda "‹" + önceki ekranın adı ("‹ Kaplar"), ortada ekranın adı (600). Soldan kaydırınca da geri gider.
- **Sheet:** solda "Vazgeç", sağda "Kaydet" veya "Bitti" (600). Kaydedilmemiş değişiklik varsa Vazgeç onay ister.
- **Sekme kökü:** büyük başlık (`.dm-large`, `t-large-title`), kaydırınca küçük başlığa döner.
- **Tam ekran** (demleme günü, sihirbaz): solda "Küçült" veya "Vazgeç".
- Düğme metinleri `tint`, başlık `label`. Sheet üstüne sheet açılmaz.

Kullanan sağlar: başlık, geri hedefinin adı veya sheet eylemleri.
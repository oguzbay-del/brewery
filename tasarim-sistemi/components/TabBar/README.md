# TabBar

iPhone'un alt sekme çubuğu; uygulamanın kök ekranları arasında geçiş.

- En fazla 5 sekme. Ev modu: Tarifler, Demle, Fermantasyon, Stok. İşletme operatörü: Bugün, Partiler, Fermantasyon, Stok.
- Seçili sekme `tint` (ikon + etiket), diğerleri `label2`. Etiket 10 px (`t-caption-2` boyutu).
- Seçili sekmeye ikinci dokunuş o sekmenin köküne döner; her sekme kendi yerini hatırlar.
- iPad ve web'de sekme çubuğu yerine kenar çubuğu kullanılır.
- Uygulamada sistem bileşeni (SwiftUI `TabView`) kullanılır; iOS 26'da cam görünüm sistemden gelir.

Kullanan sağlar: sekmeler (ikon, etiket), seçili sekme.
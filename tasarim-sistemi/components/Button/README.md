# Button

Eylem düğmesi; ekranda en fazla bir ana düğme (`.fill`) olur.

- **Ana eylem:** `.dm-btn.fill`: `tint-fill` zemin, `tint-ink` yazı. Ekrandaki tek ana eylem için (Demlemeyi başlat, Kaydet).
- **İkincil eylem:** `.dm-btn`: `fill` zemin, `tint` yazı. Bu çift açık temada 3.9:1; küçük metin sınırının altında. Ana eylem yerine kullanma; Yüksek karşıtlık temasında 5.3:1.
- **Küçük:** `.sm` (13 px, `radius-sm`), satır içinde ve başlık yanında. **Büyük:** `.big`, tam genişlik, demleme günü ve ıslak ortam için.
- **Yıkıcı:** `.danger` metni `red`; her zaman onay ister (eylem sayfası).
- Düğme metni fiille başlar ve ne olacağını söyler ("Demlemeyi başlat", "Arşivle"); "Tamam" kullanma.
- Yükseklik en az `tap-min` (44 px). Odak halkası `blue`, 3 px.

Kullanan sağlar: metin, `onClick`, gerekiyorsa `disabled`.
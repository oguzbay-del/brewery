#!/usr/bin/env python3
"""Plan <-> arayüz konsepti <-> formül tutarlılık denetimi (yalnız okur).

Kullanım (repo kökünden): python3 -I araclar/tutarlilik.py
Çıkış kodu: 1 = gerçek hata bulundu, 0 = yalnız bilgi / inceleme notları ya da temiz.
"INCELEME" ve "BİLGİ" satırları çıkış kodunu etkilemez.
"""
import difflib
import html
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

KOK = Path(__file__).resolve().parent.parent
PLAN = KOK / "plan" / "demleme-plani.md"
KONSEPT = KOK / "arayuz" / "demleme-arayuz-konsepti.html"
FORM = KOK / "formuller"

hatalar = 0
incelemeler = 0


def baslik(n, ad):
    print(f"\n== {n}. {ad} ==")


def hata(m):
    global hatalar
    hatalar += 1
    print(f"  HATA: {m}")


def incele(m):
    global incelemeler
    incelemeler += 1
    print(f"  INCELEME: {m}")


def bilgi(m):
    print(f"  BILGI: {m}")


def tamam():
    print("  TAMAM")


def satir_no(metin, pos):
    return metin.count("\n", 0, pos) + 1


def temiz(s):
    s = re.sub(r"<[^>]+>", "", s)
    return html.unescape(s).strip()


def norm(s):
    s = s.lower().replace("İ", "i").replace("ı", "i")
    tr = str.maketrans("çğöşüâî", "cgosuai")
    s = s.translate(tr)
    return re.sub(r"[^a-z0-9 ]+", " ", s).split()


plan = PLAN.read_text(encoding="utf-8")
kons = KONSEPT.read_text(encoding="utf-8")
plan_satir = plan.split("\n")

# ---------------------------------------------------------------- plan ekran tablosu
def bolum_bul(metin, baslik_re):
    m = re.search(baslik_re, metin, re.M)
    return m


ek = bolum_bul(plan, r"^## Ekran listesi\s*$")
ek_son = re.search(r"^## ", plan[ek.end():], re.M)
ek_metin = plan[ek.end(): ek.end() + (ek_son.start() if ek_son else len(plan))]
ek_ofs = ek.end()
ekranlar = []  # (ad, durum, segment, asama, satir)
tablo_ust = []
for mm in re.finditer(r"^(.*)$", ek_metin, re.M):
    ln = mm.group(1)
    if ln.startswith("|"):
        h = [x.strip() for x in ln.strip().strip("|").split("|")]
        if h[0] in ("Ekran",) or set("".join(h)) <= set("- :"):
            continue
        ekranlar.append((h[0], h[1], h[2], h[3], satir_no(plan, ek_ofs + mm.start())))
    elif ln.strip():
        tablo_ust.append(ln)

def asama_kovasi(a):
    if a.startswith("Ev MVP"):
        return "Ev MVP"
    if a.startswith("İşletme çekirdek"):
        return "İşletme çekirdek"
    if a.startswith("İşletme 2"):
        return "İşletme 2. sürüm"
    if a.startswith("İleride"):
        return "İleride, opsiyonel"
    return "?" + a

# ---------------------------------------------------------------- 1
baslik(1, "Ekran sayıları (plan tablosu <-> paragraf)")
dag = Counter(asama_kovasi(e[3]) for e in ekranlar)
print(f"  Tablo satırı: {len(ekranlar)}; dağılım: {dict(dag)}")
para = " ".join(tablo_ust)
sorun1 = False
m = re.search(r"Konseptte (\d+) ekranın", para)
m_ev = re.search(r"Ev MVP aşamasında (\d+)", para)
m_is = re.search(r"İşletme aşamalarında (\d+)", para)
if not (m and m_ev and m_is):
    hata("Paragraftaki sayılar ayrıştırılamadı (kalıp değişmiş olabilir).")
else:
    top, ev, isl = int(m.group(1)), int(m_ev.group(1)), int(m_is.group(1))
    isl_t = dag["İşletme çekirdek"] + dag["İşletme 2. sürüm"]
    ile_t = dag["İleride, opsiyonel"]
    print(f"  Paragraf: toplam {top}, Ev MVP {ev}, İşletme {isl}, (Watch ayrıca {top-ev-isl})")
    if len(ekranlar) != top:
        hata(f"Tablo satırı {len(ekranlar)} != paragraf toplamı {top}")
    if dag["Ev MVP"] != ev:
        hata(f"Ev MVP: tablo {dag['Ev MVP']} != paragraf {ev}")
    if isl_t != isl:
        hata(f"İşletme (çekirdek + 2. sürüm): tablo {isl_t} != paragraf {isl}")
    if ile_t != top - ev - isl:
        hata(f"İleride/opsiyonel: tablo {ile_t} != paragraf farkı {top-ev-isl}")
bilin = [k for k in dag if k.startswith("?")]
if bilin:
    hata(f"Tanınmayan aşama değerleri: {bilin}")
ad_say = Counter(e[0] for e in ekranlar)
for a, c in ad_say.items():
    if c > 1:
        hata(f"Tabloda yinelenen ekran adı: {a!r}")
if hatalar == 0:
    tamam()

# ---------------------------------------------------------------- konsept sekmeleri
tab_re = re.compile(r'<button role="tab" id="tab-([A-Za-z0-9_-]+)"([^>]*)>(.*?)</button>', re.S)
gruplar = []  # (grup adı, başlangıç pozisyonu)
for g in re.finditer(r'<div class="ng"><b>([^<]+)</b>', kons):
    gruplar.append((g.group(1), g.start()))

def grup_of(pos):
    ad = None
    for a, p in gruplar:
        if p <= pos:
            ad = a
    return ad

sekmeler = []  # dict
for t in tab_re.finditer(kons):
    ad = temiz(re.sub(r'<span class="newdot".*?</span>', "", t.group(3), flags=re.S))
    ac = re.search(r'aria-controls="([^"]*)"', t.group(2))
    sekmeler.append(dict(id=t.group(1), ad=ad, ac=ac.group(1) if ac else None,
                         grup=grup_of(t.start()), satir=satir_no(kons, t.start())))
sek_id = {s["id"]: s for s in sekmeler}
musteri = [s for s in sekmeler if s["grup"] != "İç kullanım"]

# ---------------------------------------------------------------- 2
baslik(2, "Sekmeler <-> paneller")
print(f"  Sekme: {len(sekmeler)} (müşteri: {len(musteri)}, İç kullanım: {len(sekmeler)-len(musteri)}); gruplar: {dict(Counter(s['grup'] for s in sekmeler))}")
n0 = hatalar
ids = Counter(s["id"] for s in sekmeler)
for i, c in ids.items():
    if c > 1:
        hata(f"tab-{i} {c} kez tanımlı")
paneller = {}
for p in re.finditer(r'<div id="pane-([A-Za-z0-9_-]+)"([^>]*)>', kons):
    paneller.setdefault(p.group(1), []).append((p.start(), p.group(2)))
for i, v in paneller.items():
    if len(v) > 1:
        hata(f"pane-{i} {len(v)} kez tanımlı")
for s in sekmeler:
    if f"pane-{s['id']}" not in {f"pane-{k}" for k in paneller}:
        hata(f"tab-{s['id']} ({s['ad']}) satır {s['satir']}: pane-{s['id']} yok")
        continue
    if s["ac"] != f"pane-{s['id']}":
        hata(f"tab-{s['id']} satır {s['satir']}: aria-controls={s['ac']!r}, beklenen pane-{s['id']}")
    pos, attrs = paneller[s["id"]][0]
    if f'aria-labelledby="tab-{s["id"]}"' not in attrs:
        hata(f"pane-{s['id']} satır {satir_no(kons,pos)}: aria-labelledby tab-{s['id']} değil ({attrs.strip()[:60]})")
    if 'role="tabpanel"' not in attrs:
        hata(f"pane-{s['id']} role=tabpanel değil")
for i, v in paneller.items():
    if i not in sek_id:
        incele(f"pane-{i} satır {satir_no(kons, v[0][0])}: sekmesi yok (yetim panel)")
# boş panel: pane başlangıcından sonraki sekme paneline kadar metin
pane_pos = sorted((v[0][0], i) for i, v in paneller.items())
for k, (pos, i) in enumerate(pane_pos):
    son = pane_pos[k + 1][0] if k + 1 < len(pane_pos) else kons.find("<script", pos)
    govde = kons[pos:son]
    govde = re.sub(r"<!--.*?-->", "", govde, flags=re.S)
    govde = re.sub(r"<script.*?</script>", "", govde, flags=re.S)
    metin = temiz(govde)
    if len(metin) < 80:
        hata(f"pane-{i} satır {satir_no(kons,pos)}: boş/çok kısa ({len(metin)} karakter)")
if hatalar == n0:
    tamam()

# ---------------------------------------------------------------- 3
baslik(3, "Konsept sekme sayısı <-> plan ekran sayısı")
print(f"  Plan tablo satırı: {len(ekranlar)}; konsept müşteri sekmesi: {len(musteri)}; fark: {len(ekranlar)-len(musteri)}")
# Adı farklı yazılan ama aynı ekran olan bilinen eşleşmeler (plan satırı başı -> sekme id).
# Bir sekme birden çok plan satırını karşılayabilir (ör. "us").
BILINEN = {
    "İlk açılış": "ob", "Su ve mash pH": "wa", "İçe aktarma eşleştirme": "im",
    "Onay kutusu": "ap", "Kullanıcı davet akışı": "us",
    "Ortak cihazda kullanıcı değiştirme": "us", "Lot izlenebilirlik raporu": "lt",
}
puan = {}
for e in ekranlar:
    en = set(norm(e[0].split(":")[0].split("(")[0]))
    for s in musteri:
        sn = set(norm(s["ad"]))
        # kök (ilk 5 harf) eşleşmesi
        ort = sum(1 for a in en for b in sn if a[:5] == b[:5] and len(a) > 2 and len(b) > 2)
        oran = difflib.SequenceMatcher(None, " ".join(norm(e[0])), " ".join(norm(s["ad"]))).ratio()
        puan[(e[0], s["id"])] = ort / max(1, len(sn)) * 0.7 + oran * 0.3
        if any(e[0].startswith(k) and v == s["id"] for k, v in BILINEN.items()):
            puan[(e[0], s["id"])] = 1.0
plan_en = {}
for e in ekranlar:
    en = max(musteri, key=lambda s: puan[(e[0], s["id"])])
    plan_en[e[0]] = (en, puan[(e[0], en["id"])])
esik = 0.55
eslesen_sek = set()
plan_eslesmeyen = []
for e in ekranlar:
    en, p = plan_en[e[0]]
    if p >= esik:
        eslesen_sek.add(en["id"])
    else:
        plan_eslesmeyen.append((e, en, p))
sek_eslesmeyen = []
for s in musteri:
    best = max(ekranlar, key=lambda e: puan[(e[0], s["id"])])
    if puan[(best[0], s["id"])] < esik:
        sek_eslesmeyen.append((s, best, puan[(best[0], s["id"])]))
if len(ekranlar) != len(musteri) and (plan_eslesmeyen or sek_eslesmeyen):
    incele("Sayılar farklı ve eşleşmeyen ad var; yeni ekran mı, ad farkı mı bakılmalı (gerekirse BILINEN'e ekle).")
elif len(ekranlar) != len(musteri):
    print("  Sayı farkı bilinen çoklu eşleşmelerden (bir sekme birden çok plan satırı).")
if plan_eslesmeyen:
    print(f"  Plan satırı, konseptte yakın adlı sekmesi bulunamayan ({len(plan_eslesmeyen)}):")
    for e, en, p in plan_eslesmeyen:
        incele(f"plan:{e[4]} {e[0][:60]!r} -> en yakın sekme {en['ad']!r} ({p:.2f})")
if sek_eslesmeyen:
    print(f"  Konsept sekmesi, planda yakın adlı satırı bulunamayan ({len(sek_eslesmeyen)}):")
    for s, b, p in sek_eslesmeyen:
        incele(f"konsept:{s['satir']} {s['ad']!r} (tab-{s['id']}) -> en yakın plan satırı {b[0][:50]!r} ({p:.2f})")
if not plan_eslesmeyen and not sek_eslesmeyen and len(ekranlar) == len(musteri):
    tamam()

# ---------------------------------------------------------------- 4
baslik(4, "Mod haritası <-> sekmeler")
mdmap_m = re.search(r'<div class="mdmap" id="mdmap".*?<p style', kons, re.S)
mdc = re.findall(r'class="mdc" data-ev="([A-Z]*)" data-go2="([^"]*)"><span class="nm">([^<]*)</span>', mdmap_m.group(0) if mdmap_m else "")
if not mdc:
    hata("Mod haritası (mdmap / mdc) ayrıştırılamadı")
else:
    print(f"  Haritada {len(mdc)} kutu; ev kodları: {dict(Counter(x[0] for x in mdc))}")
    n0 = hatalar
    harita_id = Counter(x[1] for x in mdc)
    for i, c in harita_id.items():
        if c > 1:
            incele(f"Haritada {i!r} {c} kez")
    for s in musteri:
        if s["id"] not in harita_id:
            hata(f"konsept:{s['satir']} {s['ad']!r} (tab-{s['id']}) mod haritasında yok")
    for _, i, ad in mdc:
        if i not in sek_id:
            hata(f"Haritada {ad!r} (data-go2={i}) için sekme yok")
        elif sek_id[i]["grup"] == "İç kullanım":
            incele(f"Haritada iç kullanım sekmesi var: {ad!r}")
    if hatalar == n0:
        tamam()

# ---------------------------------------------------------------- 5
baslik(5, "Apple karşılıkları tablosu (bilgi)")
ab = re.search(r"<h2>Her ekranın Apple'daki karşılığı</h2>", kons)
ab_son = kons.find("</table>", ab.end())
ab_metin = kons[ab.end():ab_son]
apple_satirlar = [temiz(re.sub(r'<span class="newdot".*?</span>', "", td, flags=re.S))
                  for td in re.findall(r"<tr><td>(.*?)</td>", ab_metin, re.S)]
print(f"  Tabloda {len(apple_satirlar)} satır")
lead = re.search(r'<p class="lead">(\d+) ekranın hepsi', ab_metin)
if lead:
    print(f"  Giriş cümlesi '{lead.group(1)} ekranın hepsi' diyor; müşteri sekmesi {len(musteri)}, tablo satırı {len(apple_satirlar)}")
    if int(lead.group(1)) != len(apple_satirlar):
        incele(f"Giriş cümlesindeki sayı ({lead.group(1)}) tablo satırı ({len(apple_satirlar)}) ile uyuşmuyor")
eksik5 = []
for s in musteri:
    en_iyi = max((difflib.SequenceMatcher(None, " ".join(norm(s["ad"])), " ".join(norm(a))).ratio() for a in apple_satirlar), default=0)
    kok = any(any(x[:5] == y[:5] and len(x) > 3 for x in norm(s["ad"]) for y in norm(a)) for a in apple_satirlar)
    if en_iyi < 0.6 and not kok:
        eksik5.append(s)
if eksik5:
    for s in eksik5:
        bilgi(f"konsept:{s['satir']} {s['ad']!r} (tab-{s['id']}) Apple tablosunda satırı yok görünüyor")
else:
    tamam()

# ---------------------------------------------------------------- 6
baslik(6, "Konsept içi bağlantılar (data-go / data-go2)")
n0 = hatalar
sayac = Counter()
for m in re.finditer(r'data-go2?="([^"]*)"', kons):
    sayac[m.group(1)] += 1
    if m.group(1) not in sek_id:
        hata(f"satır {satir_no(kons, m.start())}: {m.group(0)} -> tab-{m.group(1)} yok")
print(f"  {sum(sayac.values())} bağlantı, {len(sayac)} farklı hedef")
if hatalar == n0:
    tamam()

# ---------------------------------------------------------------- 7
baslik(7, "Formül göndermeleri")
n0 = hatalar
form_dosya = {}
for f in sorted(FORM.glob("[0-9][0-9]-*.md")):
    form_dosya[f.name[:2]] = f
basliklar = {}
for nn, f in form_dosya.items():
    kume = set()
    for ln in f.read_text(encoding="utf-8").split("\n"):
        mm = re.match(r"^##\s+(\d+)\.?(?:\s|$)", ln)
        if mm:
            kume.add(mm.group(1))
        mm = re.match(r"^###\s+(\d+\.\d+)\.?(?:\s|$)", ln)
        if mm:
            kume.add(mm.group(1))
    basliklar[nn] = kume
ref_dosyalar = [PLAN] + sorted(FORM.glob("*.md"))
re_nn = re.compile(r"(?<![\d./-])(\d{2}) ?§ ?(\d+(?:\.\d+)?)")
re_fd = re.compile(r"formuller/(\d{2})(?:-[\w.-]*?\.md)?(?![\d])(?: ?§ ?(\d+(?:\.\d+)?))?")
toplam_ref = 0
for f in ref_dosyalar:
    metin = f.read_text(encoding="utf-8")
    for m in re_nn.finditer(metin):
        toplam_ref += 1
        nn, s = m.group(1), m.group(2)
        yer = f"{f.relative_to(KOK)}:{satir_no(metin, m.start())}"
        if nn not in form_dosya:
            hata(f"{yer}: {m.group(0)!r} -> formuller/{nn}-*.md yok")
        elif s not in basliklar[nn]:
            hata(f"{yer}: {m.group(0)!r} -> {form_dosya[nn].name} içinde '{s}' başlığı yok")
    for m in re_fd.finditer(metin):
        toplam_ref += 1
        nn = m.group(1)
        yer = f"{f.relative_to(KOK)}:{satir_no(metin, m.start())}"
        if nn not in form_dosya:
            hata(f"{yer}: {m.group(0)!r} -> formuller/{nn}-*.md yok")
        else:
            tam = re.match(r"formuller/(\d{2}-[\w.-]+\.md)", m.group(0))
            if tam and not (FORM / tam.group(1).split("/")[-1]).exists():
                hata(f"{yer}: {tam.group(0)!r} dosya adı yanlış")
        if m.group(2) and nn in basliklar and m.group(2) not in basliklar[nn]:
            hata(f"{yer}: {m.group(0)!r} -> '{m.group(2)}' başlığı yok")
print(f"  {toplam_ref} gönderme tarandı ({len(form_dosya)} formül dosyası)")
if hatalar == n0:
    tamam()

# ---------------------------------------------------------------- 8
baslik(8, "Formül test betikleri")
n0 = hatalar
testler = sorted((FORM / "testler").glob("*.py"))
for t in testler:
    try:
        r = subprocess.run([sys.executable, "-I", str(t)], cwd=FORM, capture_output=True,
                           text=True, timeout=60)
        if r.returncode == 0:
            bilgi(f"{t.name}: çıkış 0")
        else:
            cikti = (r.stderr.strip() or r.stdout.strip()).splitlines()[:5]
            hata(f"{t.name}: çıkış kodu {r.returncode}; " + " | ".join(cikti))
    except subprocess.TimeoutExpired:
        hata(f"{t.name}: 60 sn zaman aşımı")
print(f"  {len(testler)} betik çalıştırıldı")
if hatalar == n0:
    tamam()

# ---------------------------------------------------------------- 9
baslik(9, '"doğrulanacak" sayımı')
sayim = Counter()
for i, ln in enumerate(plan_satir):
    pass
bas = "(giriş)"
n_plan = 0
for ln in plan_satir:
    mm = re.match(r"^(#{2,3}) (.*)", ln)
    if mm:
        bas = ("## " if len(mm.group(1)) == 2 else "### ") + mm.group(2).strip()
    c = len(re.findall("doğrulanacak", ln, re.I))
    if c:
        sayim["plan › " + bas] += c
        n_plan += c
n_form = 0
for f in sorted(FORM.glob("*.md")):
    c = len(re.findall("doğrulanacak", f.read_text(encoding="utf-8"), re.I))
    if c:
        sayim["formuller/" + f.name] += c
        n_form += c
print(f"  Toplam {n_plan + n_form} (plan {n_plan}, formuller {n_form})")
for k, v in sayim.most_common(10):
    print(f"   {v:3d}  {k}")

# ---------------------------------------------------------------- özet
print(f"\nÖZET: {hatalar} hata, {incelemeler} inceleme notu. " + ("BAŞARISIZ" if hatalar else "TEMİZ (inceleme notları kesin hata değildir)"))
sys.exit(1 if hatalar else 0)

"""İşletme 2. sürüm hesapları: etiket enerjisi, vergi/beyan, spunding.
Çalıştır: python3 -I testler/09_isletme_2_tv.py   (yalnız standart kütüphane)
Vergi oranları ÖRNEKTİR; kaynak ve tarih spesifikasyonda (09 §2.4). Gerçek oranlar veri tablosundan gelir."""
import math
from decimal import Decimal as D, ROUND_HALF_UP, ROUND_FLOOR

def r(x, n=3): return round(x, n)
def to_plato(sg): return 135.997*sg**3 - 630.272*sg**2 + 1111.14*sg - 616.868      # 01 §1.1
def to_sg(P): return 1 + P/(258.6 - (P/258.6)*227.1)                              # 01 §1.2
def real_extract(OE, AE): return 0.1808*OE + 0.8192*AE                            # 01 §6.1
def abw(OE, RE): return (OE - RE)/(2.0665 - 0.010665*OE)                          # 01 §6.2
def abv_from(og, fg):
    OE, AE = to_plato(og), to_plato(fg); RE = real_extract(OE, AE)
    w = abw(OE, RE); return w*fg/0.79661, w, RE                                    # 01 §6.3

# ---------------------------------------------------------------- 1. Etiket enerjisi (AB 1169/2011 Ek XIV)
KJ = {"alkol": 29, "karbonhidrat": 17, "protein": 17, "yag": 37, "organik_asit": 13, "poliol": 10, "lif": 8}
KCAL = {"alkol": 7, "karbonhidrat": 4, "protein": 4, "yag": 9, "organik_asit": 3, "poliol": 2.4, "lif": 2}
ETOH_DENS = 0.78924          # g/mL, 20 °C [kesin]
WATER_DENS = 0.9982          # g/mL, 20 °C [kesin]; SG × 0.9982 = biranın 20 °C'de g/mL yoğunluğu

def label_energy(og, fg, protein_g100mL=0.40, abv_measured=None):
    abv, w, RE = abv_from(og, fg)
    if abv_measured is not None:
        abv = abv_measured
        alc_g = abv*ETOH_DENS                              # ölçülmüş ABV: hacim yüzdesi × etanol yoğunluğu
    else:
        alc_g = w*fg*WATER_DENS                            # tahmini ABV: doğrudan ABW (g/100 g) × SG × 0.9982 (F1 karar 13)
    re_g = RE*fg                                           # g/100 g × g/mL -> g/100 mL
    carb = max(re_g - protein_g100mL, 0.0)
    kj = KJ["alkol"]*alc_g + KJ["karbonhidrat"]*carb + KJ["protein"]*protein_g100mL
    kcal = KCAL["alkol"]*alc_g + KCAL["karbonhidrat"]*carb + KCAL["protein"]*protein_g100mL
    return {"ABV": r(abv, 2), "alkol_g": r(alc_g, 2), "gercek_ekstrakt_g": r(re_g, 2), "karbonhidrat_g": r(carb, 1),
            "protein_g": r(protein_g100mL, 1), "kJ": round(kj), "kcal": round(kcal)}

# ---------------------------------------------------------------- 2. Vergi motoru (yöntemler veri, oranlar veri)
# Para ve vergi hesabı ondalık tiple (Decimal); bant/eşik karşılaştırmaları da ondalık. (Karşılaştırma ilkesi:
# 00-OKU-BENI.md › Önemli genel notlar.) Girdi (ABV, °P) yuvarlama kuralı ve bant sınırı türü veri tablosundadır.
HL_PER_BBL = D("1.17348")    # 31 US gal [kesin]

def d(x): return x if isinstance(x, D) else D(str(x))

class _Tanimsiz:
    def __repr__(self): return "TANIMSIZ"
TANIMSIZ = _Tanimsiz()      # girdi yuvarlama kuralı satırda yazılmamış (F1 karar 4: hata)

def _round_in(x, dp, mode):
    """Girdi yuvarlaması (veri tablosu): dp = ondalık hane sayısı (None: yuvarlama yok), mode = decimal yuvarlama modu."""
    x = d(x)
    return x if dp is None else x.quantize(D(1).scaleb(-dp), rounding=mode)

def _band(value, bands, inclusive):
    """bands = [(üst, oran)] artan; inclusive: değer == üst ise o banda girer. Bant yoksa hata (sessiz 0 yok)."""
    for upper, rate in bands:
        if upper is None or (value <= d(upper) if inclusive else value < d(upper)): return d(rate)
    raise ValueError("bant bulunamadı: son bant açık uçlu olmalı (üst sınır None)")

def row_status(resmi_kaynak=None, tarih=None):
    """Ülke satırı durumu: yalnız resmi kaynak (HMRC, BierStG, TTB, Resmi Gazete...) ve kaynak tarihi birlikte verilmişse
    'doğrulandı'; aksi hâlde 'doğrulanacak' (F1 karar 4)."""
    return "doğrulandı" if resmi_kaynak and tarih else "doğrulanacak"

def money(x, dp=2, mode=ROUND_HALF_UP):
    """Para yuvarlaması (veri tablosundaki kurala göre; varsayılan: yarım yukarı)."""
    return d(x).quantize(D(1).scaleb(-dp), rounding=mode)

def tax_line(method, V_L, abv=None, plato=None, rate=None, bands=None, tiers=None, ytd=0,
             plato_trunc=True, discount_per_lpa=0, input_dp=TANIMSIZ, input_mode=ROUND_HALF_UP, inclusive=None):
    """Dönüş: Decimal (yuvarlanmamış), her zaman ≥ 0. method:
      'hacim'        : oran × hL (rate = para birimi / hL; litre başına oran × 100 olarak girilir)
      'saf_alkol'    : oran × litre saf alkol (LPA = V × ABV/100); bands=[(abv_üst, oran)] varsa ABV bandından oran;
                       bant sınırı varsayılan DIŞLAYICI (abv < üst)
      'hl_plato'     : oran × hL × °P (°P tam sayıya kesilebilir)
      'plato_bandi'  : bands=[(plato_üst, PB/hL)], bant sınırı varsayılan KAPSAYICI (°P ≤ üst)
      'kademeli_hacim': tiers=[(yıllık_üst_hL, PB/hL)], ytd = bu yıl daha önce vergilenen hL; son kademenin üstü
                       None (açık uçlu) olmalı, aksi hâlde kalan hacim için hata verilir
    input_dp/input_mode: ABV ve °P'nin vergi için kaç haneye, hangi yönde yuvarlanacağı. input_dp=None açıkça
      "yuvarlama yok" demektir (varsayılan davranış, ama satır bunu yazmalı); input_dp verilmezse (TANIMSIZ) ABV ya da
      °P kullanan yöntemlerde (saf_alkol, hl_plato, plato_bandi) hata verilir (F1 karar 4).
    inclusive: bant sınırı türünü ezer."""
    if method in ("saf_alkol", "hl_plato", "plato_bandi") and input_dp is TANIMSIZ:
        raise ValueError("girdi yuvarlama kuralı tanımsız: ülke satırı input_dp yazmalı (None = yuvarlama yok)")
    hl = d(V_L) / 100
    if method == "hacim":
        return max(d(rate) * hl, D(0))
    if method == "saf_alkol":
        a = _round_in(abv, input_dp, input_mode)
        lpa = d(V_L) * a / 100
        rt = _band(a, bands, False if inclusive is None else inclusive) if bands else d(rate)
        return max(lpa * max(rt - d(discount_per_lpa), D(0)), D(0))      # indirim oranı aşamaz, negatif vergi yok
    if method == "hl_plato":
        p = _round_in(plato, input_dp, input_mode)
        if plato_trunc: p = p.to_integral_value(rounding=ROUND_FLOOR)
        return max(d(rate) * hl * p, D(0))
    if method == "plato_bandi":
        p = _round_in(plato, input_dp, input_mode)
        return _band(p, bands, True if inclusive is None else inclusive) * hl
    if method == "kademeli_hacim":
        tax, start, left = D(0), d(ytd), hl
        for upper, rt in tiers:
            if left <= 0: break
            room = left if upper is None else max(d(upper) - start, D(0))
            take = min(room, left)
            tax += take * d(rt); start += take; left -= take
        if left > 0: raise ValueError("son kademenin üst sınırı aşıldı: %s hL kademesiz" % left)
        return tax
    raise ValueError(method)

# ---------------------------------------------------------------- 3. Spunding
CO2_G_PER_VOL = 44.01/22.414  # = 1.9635 g/L, ideal gaz, 0 °C 1 atm [kesin; 10 ve 11 ile tek sabit, F1 karar 9]
BALLING_CO2 = 0.9565/2.0665   # g CO2 / g fermente ekstrakt [Balling]

def residual_co2(T_C):                                     # 04 §3.2
    T = max(T_C*9/5 + 32, 32); return 3.0378 - 0.050062*T + 0.00026555*T*T

def co2_pressure_psi(T_C, vol):                            # 04 §3.5
    T = max(T_C*9/5 + 32, 32)
    return max(-16.6999 - 0.0101059*T + 0.00116512*T*T + 0.173354*T*vol + 4.24267*vol - 0.0684226*vol*vol, 0.0)

def spund(target_vol, T_C, fg_expected):
    need_vol = target_vol - residual_co2(T_C)
    need_g = max(need_vol, 0)*CO2_G_PER_VOL
    sg = fg_expected
    dRE_gL = need_g/BALLING_CO2                           # fermente olması gereken gerçek ekstrakt (g/L)
    dRE_P = dRE_gL/(10*sg)                                 # °P (g/100 g)
    dAE_P = dRE_P/0.8192                                   # görünür ekstrakt düşüşü
    close_P = to_plato(fg_expected) + dAE_P
    psi = co2_pressure_psi(T_C, target_vol)
    return {"kalinti_vol": r(residual_co2(T_C), 2), "gereken_g_L": r(need_g, 2), "kapama_SG": r(to_sg(close_P), 4),
            "puan_once": r((to_sg(close_P) - fg_expected)*1000, 1), "ayar_psi": r(psi, 1), "ayar_bar": r(psi*0.0689476, 2)}

# ---------------------------------------------------------------- 4. Beyan, vergilenecek hacim, spunding uyarıları (T1.14g)
LABEL_NUTRITION_EXEMPT_ABV = D("1.2")      # §1.3 [mevzuat, doğrulanacak]: bu ABV'nin üstünde besin değeri beyanı zorunlu değil

def label_declaration(abv):
    """§1.3 hangi beyan: ABV > %1.2 ise besin değeri beyanı zorunlu değildir, yalnız enerji beyan edilebilir
    ("enerji_istege_bagli"); ABV ≤ %1.2 ise beyan ülke etiket şablonuna göre ("ulke_sablonu"). Karşılaştırma Decimal.
    (Enerji kJ/kcal tam sayıya, besin değerleri 0.1 g'a yuvarlanır: label_energy.)"""
    return "enerji_istege_bagli" if d(abv) > LABEL_NUTRITION_EXEMPT_ABV else "ulke_sablonu"

TAX_TRIGGERS = ("ambalaj", "cikis", "satis")          # §2.1 verginin doğduğu an (veri tablosu seçer)
TAX_OTHER = ("imha", "iade", "lab")                   # ayrı satırlarda raporlanan hacimler

def taxable_volume(events, trigger, start, end, deduct=()):
    """§2.1 dönem vergilenecek hacmi (litre, Decimal). events = [(tarih 'YYYY-MM-DD', tür, litre)]; tür
    TAX_TRIGGERS ya da TAX_OTHER. start ≤ tarih ≤ end (sınırlar dahil, ISO metin karşılaştırması).
    vergilenen = trigger türündeki olayların toplamı (diğer tetikleyici türleri sayılmaz); imha/iade/lab ayrı satır;
    deduct (veri tablosu kuralı) içindeki türler vergilenenden düşülür; sonuç negatif olmaz."""
    if trigger not in TAX_TRIGGERS: raise ValueError("bilinmeyen tetikleyici: %s" % trigger)
    if any(k not in TAX_OTHER for k in deduct): raise ValueError("düşülebilir tür imha/iade/lab olmalı")
    taxed = D(0); other = {k: D(0) for k in TAX_OTHER}
    for date, kind, litres in events:
        if kind not in TAX_TRIGGERS and kind not in TAX_OTHER: raise ValueError("bilinmeyen olay türü: %s" % kind)
        if not (start <= date <= end): continue
        v = d(litres)
        if kind == trigger: taxed += v
        elif kind in other: other[kind] += v
    net = taxed
    for k in TAX_OTHER:
        if k in deduct: net = net - other[k]
    return {"vergilenecek": max(net, D(0)), "vergilenen": taxed,
            "imha": other["imha"], "iade": other["iade"], "lab": other["lab"]}

SPUND_WIDE_U = 200        # [tasarım, doğrulanacak] FG güven aralığı genişliği eşiği: 0.002 SG (200 birim)
def spund_warnings(target_vol, T_C, tank_max_psi=None, fg_range=None, wide_u=SPUND_WIDE_U):
    """§3.4 uyarı kodları. tank_max_psi: tank kaydındaki çalışma basıncı (None: denetim yok); ayar basıncı onu aşarsa
    "tank_basinc_asildi" ve engellenir. fg_range = (alt, üst) FG güven aralığı (07 §1); genişlik > wide_u birim ise
    "erken_kapama_riski" (1–2 puan erken kapatma). Dönüş: {"uyari": [...], "engellendi": bool}."""
    out = []
    psi = co2_pressure_psi(T_C, target_vol)
    blocked = tank_max_psi is not None and psi > tank_max_psi
    if blocked: out.append("tank_basinc_asildi")
    if fg_range is not None and _u_sg(fg_range[1]) - _u_sg(fg_range[0]) > wide_u: out.append("erken_kapama_riski")
    return {"uyari": out, "engellendi": blocked}

def _u_sg(sg): return round(sg * 100000)


# ================================================================ TEST VEKTÖRLERİ
if __name__ == "__main__":
    print("== 1 Etiket enerjisi")
    print("TV1.1 Pilsner OG 1.048 FG 1.009:", label_energy(1.048, 1.009))
    print("TV1.2 Citra PA OG 1.052 FG 1.011:", label_energy(1.052, 1.011))
    print("TV1.3 ölçülen ABV %5.0 ile Pilsner:", label_energy(1.048, 1.009, abv_measured=5.0))

    print("== 2 Vergi (örnek oranlar, 20 hL, %5.0, 12 °P; Decimal)")
    V, A, P = 2000.0, 5.0, 12.0
    def pr(x): return float(money(x))
    print("TV2.1 saf alkol, 21.78 PB/LPA (örnek):", pr(tax_line("saf_alkol", V, abv=A, rate=21.78, input_dp=None)))
    print("TV2.2 saf alkol, ABV bandı [(3.5, 9.27), (8.5, 21.78), (None, 29.54)] + indirim 5.0/LPA:",
          pr(tax_line("saf_alkol", V, abv=A, bands=[(3.5, 9.27), (8.5, 21.78), (None, 29.54)], discount_per_lpa=5.0, input_dp=None)))
    print("TV2.3 hL·°P, 0.787 PB (12.6 °P kesilir -> 12):", pr(tax_line("hl_plato", V, plato=12.6, rate=0.787, input_dp=None)))
    b4 = [(10.0, 16.88), (14.0, 25.32), (None, 33.76)]
    print("TV2.4 Plato bandı [(10.0,16.88),(14.0,25.32),(None,33.76)] PB/hL:", pr(tax_line("plato_bandi", V, plato=P, bands=b4, input_dp=None)))
    tiers = [(60000*HL_PER_BBL, D("3.50")/HL_PER_BBL), (None, D("16.0")/HL_PER_BBL)]
    print("TV2.5 kademeli (60 000 bbl @3.50, sonra @16 /bbl), 20 hL, ytd 0:", pr(tax_line("kademeli_hacim", V, tiers=tiers)))
    print("TV2.6 aynı, eşiği aşan: ytd = 60000 bbl − 10 hL:", pr(tax_line("kademeli_hacim", V, tiers=tiers, ytd=60000*HL_PER_BBL - 10)))
    print("TV2.7 hacim, 12.49 PB/L = 1249 PB/hL (örnek):", pr(tax_line("hacim", V, rate=1249.0)))
    print("TV2.8 bant sınırı: ≤10.0 °P kapsayıcı; plato 10.0 ve 10.05 (input_dp yok) ve 10.05 (input_dp=1, yarım yukarı):",
          [pr(tax_line("plato_bandi", V, plato=x, bands=b4, input_dp=dp)) for x, dp in ((10.0, None), (10.05, None), (10.05, 1))])
    print("TV2.9 ABV 3.5 sınırı dışlayıcı (3.5 → ikinci bant), LPA 70:", pr(tax_line("saf_alkol", V, abv=3.5, bands=[(3.5, 9.27), (8.5, 21.78), (None, 29.54)], input_dp=None)))
    print("TV2.10 indirim > oran → vergi 0 (negatif değil):", pr(tax_line("saf_alkol", V, abv=A, rate=4.0, discount_per_lpa=5.0, input_dp=None)))
    try: tax_line("kademeli_hacim", V, tiers=[(100.0, 3.0)], ytd=95.0)
    except ValueError as e: print("TV2.11 son kademe aşıldı → hata:", e)
    print("TV2.12 float tuzağı: 2.675 girdisi (input_dp=2, yarım yukarı) →", _round_in(2.675, 2, ROUND_HALF_UP))
    try: tax_line("saf_alkol", V, abv=A, rate=21.78)
    except ValueError as e: print("TV2.13 girdi yuvarlama kuralı tanımsız → hata:", e)
    print("TV2.13b aynı, kural açıkça 'yuvarlama yok' (input_dp=None):", pr(tax_line("saf_alkol", V, abv=A, rate=21.78, input_dp=None)))
    print("TV2.14 satır durumu: kaynaksız | resmi kaynak ve tarihli:", row_status(), "|", row_status("gov.uk SPR kılavuzu", "2026-10-31"))

    print("== 3 Spunding")
    print("TV3.1 hedef 2.5 vol, 12 °C, FG 1.010:", spund(2.5, 12.0, 1.010))
    print("TV3.2 hedef 2.6 vol, 18 °C, FG 1.012:", spund(2.6, 18.0, 1.012))

"""İşletme çekirdek hesapları: referans uygulama ve test vektörleri.
Çalıştır: python3 -I testler/08_isletme_cekirdek_tv.py   (yalnız standart kütüphane)
Birimler: L, hL, kg, °P, SG; para birimi soyut ("PB")."""
import math
from decimal import Decimal, ROUND_HALF_UP

def r(x, n=3): return round(x, n)

def money(x):
    """Para kalemi: 2 ondalık, yarım yukarı (T1.12 C6)."""
    return x.quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)

def plato_to_sg(P):  # 01 §1.2 ile aynı polinom
    return 1 + P / (258.6 - (P / 258.6) * 227.1)

# ---------------------------------------------------------------- 1. Maliyet (FIFO)
# Stok miktarı ve para Decimal (T1.12 C2, C6): miktar yuvarlanmaz; her maliyet satırı 2 ondalık ROUND_HALF_UP,
# toplam = yuvarlanmış satırların toplamı; litre = toplam / ambalajlanan L, 3 ondalık ROUND_HALF_UP.
def fifo_consume(lots, qty):
    """lots: [(lot, kalan_miktar, birim_fiyat)] Decimal, kabul sırasında. -> (satırlar, maliyet, eksik)"""
    rows, cost, need = [], Decimal(0), qty
    for lot, left, price in lots:
        if need <= 0: break
        take = min(left, need)
        if take > 0:
            line = money(take*price)
            rows.append((lot, take, price, line)); cost += line; need -= take
    return rows, cost, max(need, Decimal(0))

def landed_price(invoice_price, qty, shipping_share):
    """Kargo/gümrük payı lot birim fiyatına eklenir (yuvarlanmaz). qty <= 0 hata."""
    if qty <= 0: raise ValueError("miktar > 0 olmalı")
    return (invoice_price*qty + shipping_share)/qty

def energy_cost_electric(kWh, price_kWh): return kWh*price_kWh
def kwh_from_heat(kJ, eff): return kJ/eff/3600.0
def gas_m3_from_heat(kJ, eff, Hu_kJ_m3=34500.0): return kJ/(eff*Hu_kJ_m3)

def batch_cost(ingredient_cost, water_L, water_price_m3, energy_cost, pack_units, pack_unit_cost, packaged_L):
    if packaged_L <= 0: raise ValueError("ambalajlanan hacim > 0 olmalı")
    water = water_L/Decimal(1000)*water_price_m3
    pack = pack_units*pack_unit_cost
    lines = [money(ingredient_cost), money(water), money(energy_cost), money(pack)]
    total = sum(lines, Decimal(0))
    return {"malzeme": lines[0], "su": lines[1], "enerji": lines[2], "ambalaj": lines[3],
            "toplam": total, "litre": (total/packaged_L).quantize(Decimal("0.001"), rounding=ROUND_HALF_UP)}

# ---------------------------------------------------------------- 2. Harmanlama
def blend(parts):
    """parts: [dict(V, ABV, IBU, SRM, CO2, P)] -> karışım. V litre. P gerçek ekstrakt °P (veya FG °P)."""
    V = sum(p["V"] for p in parts)
    out = {"V": V}
    for k in ("ABV", "IBU", "SRM", "CO2"):
        out[k] = sum(p["V"]*p[k] for p in parts)/V          # hacim ağırlıklı
    m = [p["V"]*plato_to_sg(p["P"]) for p in parts]         # kütle (kg)
    out["P"] = sum(mi*p["P"] for mi, p in zip(m, parts))/sum(m)  # kütle ağırlıklı
    return out                                              # ham değer (T1.12 C9)

def blend_volume_for_target(V1, x1, x2, xt):
    """İki bileşen: hedef değere (ör. ABV) ulaşmak için 2. bileşenden gereken hacim."""
    if not (min(x1, x2) + 1e-9 < xt < max(x1, x2) - 1e-9): raise ValueError("hedef iki bileşen arasında olmalı")   # kesin ve toleranslı (T1.12 C7); Karar (T1.16): eskiden None
    return V1*(x1 - xt)/(xt - x2)

# ---------------------------------------------------------------- 3. Maya tekrar kullanımı
def cells_needed_billion(pitch_M_mL_P, V_L, P):          # 04 §1.8
    return pitch_M_mL_P*V_L*P

def hemocytometer(total_counted, squares_counted, dilution):
    """Neubauer: 25 küçük kare = 1 mm² × 0.1 mm = 1e-4 mL. -> hücre/mL"""
    return total_counted/squares_counted*25*1e4*dilution

def viability_stain(unstained, stained): return unstained/(unstained + stained)

def slurry_volume_mL(cells_billion, density_cells_mL=None, viability=1.0, default_density=1.0e9):
    """Ölçülmüş yoğunluk varsa onu kullan; yoksa varsayılan etkin yoğunluk (maya dışı tortu dahil) [doğrulanacak]."""
    d = density_cells_mL if density_cells_mL else default_density
    return cells_billion*1e9/(d*viability)

# ---------------------------------------------------------------- 4. Profesyonel verim
CONTRACTION = 0.96   # sıcak (≈100 °C) hacim -> 20 °C hacim

def extract_kg(V_L, P, hot=True):
    v = V_L*(CONTRACTION if hot else 1.0)
    return v*plato_to_sg(P)*P/100.0

def brewhouse_yield(V_L, P, malt_kg, hot=True):
    """Sudhausausbeute: kazandan çıkan şırada ekstrakt / malt kütlesi (%)."""
    return 100*extract_kg(V_L, P, hot)/malt_kg

def lab_extract_as_is(fgdb, moisture, fc_diff=0.0):
    """Malt analizi -> olduğu gibi (as-is) ekstrakt %. fc_diff verilirse kaba öğütme."""
    return (fgdb - fc_diff)*(1 - moisture/100.0)

def first_wort_max_P(grain_kg, extract_as_is_pct, moisture_pct, mash_water_L):
    """Braukaiser: tam dönüşümde ilk şıranın ulaşabileceği °P."""
    e = grain_kg*extract_as_is_pct/100.0
    w = mash_water_L + grain_kg*moisture_pct/100.0
    return 100*e/(e + w)

def conversion_eff(fw_measured_P, fw_max_P): return fw_measured_P/fw_max_P

def lab_ratio(brewhouse_pct, lab_extract_pct):
    """§4.2 laboratuvara oran = brewhouse verimi / laboratuvar ekstraktı (kesir; % için ×100). Bölen 0: ValueError."""
    if lab_extract_pct == 0: raise ValueError("laboratuvar ekstraktı 0 olamaz")
    return brewhouse_pct/lab_extract_pct

def lauter_eff(brewhouse_pct, lab_extract_pct, conversion):
    """§4.3 süzme verimi = (brewhouse verimi / laboratuvar ekstraktı) / dönüşüm verimi (kesir). Bölenlerden biri 0:
    ValueError (T1.17). BIAB ve sparge'sız mash'te anlamsız: istemci gösterimi gizler (motor hesaplar)."""
    if conversion == 0: raise ValueError("dönüşüm verimi 0 olamaz")
    return lab_ratio(brewhouse_pct, lab_extract_pct)/conversion

# ---------------------------------------------------------------- 5. Malt analizinden potansiyel
def malt_from_analysis(fgdb, moisture, fc_diff):
    fgai = fgdb*(1 - moisture/100.0)
    cgdb = fgdb - fc_diff
    cgai = cgdb*(1 - moisture/100.0)
    yield_pct = cgai                                  # tarif hesaplarında kullanılan "yield" (01 §1.6)
    pot = 1 + yield_pct*0.04621/100.0                 # sakaroz = 46.21 ppg (kesin değer)
    pot_bs = 1 + yield_pct*0.046/100.0                # BeerSmith yuvarlaması (01 §1.6)
    return {"FGAI": fgai, "CGDB": cgdb, "CGAI_yield": cgai,         # ham değer (T1.12 C9)
            "potansiyel": pot, "potansiyel_BS": pot_bs, "PPG": yield_pct*0.4621}

def ebc_from_lab(ebc): return {"EBC": ebc, "SRM": ebc/1.97, "°L": (ebc/1.97 + 0.76)/1.3546}  # 1.3546: plan kararı (01 §1.5 notu)

# ---------------------------------------------------------------- 6. Uyarı kuralları (T1.14g)
def yeast_reuse_warnings(generation, days_since_harvest, viability=None, max_generation=8, history_differs=False):
    """§3.4 uyarı tablosu, tablo sırasıyla kod listesi. viability: kesir (0-1) ya da None (ölçülmemiş: canlılık
    uyarısı verilmez). history_differs: farklı stil ya da maya suşu geçmişi (bilgi)."""
    out = []
    if generation > max_generation: out.append("nesil_siniri")                 # "yeni kültür önerilir"
    if days_since_harvest > 14 + 1e-9: out.append("hasat_14_gun")              # "canlılık düşük olabilir, sayım önerilir"
    if viability is not None and viability < 0.80 - 1e-9: out.append("canlilik_dusuk")  # "starter veya taze maya"
    if history_differs: out.append("bilgi_gecmis")                             # bilgi: nesil zinciri gösterilir
    return out

DIASTATIC_MIN_L = 35.0     # [doğrulanacak eşik] tahıl faturası ağırlıklı ortalaması bunun altındaysa uyarı
def diastatic_check(grains, threshold=DIASTATIC_MIN_L):
    """§5.4 kendi kendini dönüştürür mü: grains = [(kg, diastatik_güç_°L)] (tahıl faturasındaki bütün tahıllar;
    diastatik gücü olmayan tahıl 0 ile girilir). Ağırlıklı ortalama < eşik ise uyarı. Boş/sıfır ağırlık: uyarı yok."""
    w = sum(kg for kg, _ in grains)
    if w <= 0: return {"ortalama": 0.0, "uyari": []}
    avg = sum(kg*dp for kg, dp in grains)/w
    return {"ortalama": avg, "uyari": ["dusuk_diastatik_guc"] if avg < threshold - 1e-9 else []}


# ================================================================ TEST VEKTÖRLERİ
if __name__ == "__main__":
    print("== 1 Maliyet")
    lots = [("L-A", Decimal("3.0"), Decimal("1.10")), ("L-B", Decimal("25.0"), Decimal("1.25"))]
    rows, c, miss = fifo_consume(lots, Decimal("4.5"))
    print("TV1.1 Pale 4.5 kg FIFO:", rows, "maliyet", c, "eksik", miss)
    rows, c, miss = fifo_consume([("L-2411", Decimal("0.20"), Decimal("52.0"))], Decimal("0.30"))
    print("TV1.2 Crystal 0.30 kg, stok 0.20:", "maliyet", c, "eksik", miss)
    print("TV1.3 kargo payı: 25 kg × 1.20 + 4.00 kargo ->", r(float(landed_price(Decimal("1.20"), Decimal(25), Decimal("4.0"))), 3), "PB/kg")
    kJ_strike = (15*4.186 + 8*0.5)*(72 - 15)          # 07 §3.1
    kJ_boil_up = (28*4.186 + 8*0.5)*(100 - 76)
    kwh = kwh_from_heat(kJ_strike + kJ_boil_up, 0.90) + 2.5*70/60
    print("TV1.4 enerji: strike + kaynamaya ısınma + 70 dk 2.5 kW kaynatma ->", r(kwh, 2), "kWh")
    print("TV1.5 gaz: yalnız ısınma (strike + kaynamaya), η 0.40:", r(gas_m3_from_heat(kJ_strike + kJ_boil_up, 0.40), 3), "m³")
    en = energy_cost_electric(Decimal(repr(round(kwh, 2))), Decimal("2.6"))
    print("TV1.6 parti:", batch_cost(Decimal("18.40"), Decimal(35), Decimal(30), en, Decimal(60), Decimal("0.35"), Decimal(20)))

    print("== 2 Harmanlama")
    a = dict(V=960, ABV=5.4, IBU=38, SRM=7.0, CO2=1.0, P=2.8)
    b = dict(V=400, ABV=4.6, IBU=22, SRM=4.0, CO2=1.0, P=2.4)
    print("TV2.1 960 L + 400 L:", {k: r(v, 3) for k, v in blend([a, b]).items()})
    print("TV2.2 %5.4 olan 960 L'yi %5.2'ye: %4.6'dan", r(blend_volume_for_target(960, 5.4, 4.6, 5.2), 1), "L")
    print("TV2.3 suyla %5.4 -> %5.0:", r(blend_volume_for_target(960, 5.4, 0.0, 5.0), 1), "L su")

    print("== 3 Maya tekrar kullanımı")
    n = cells_needed_billion(0.75, 1000, 12.0)
    print("TV3.1 10 hL 12 °P ale 0.75:", r(n, 0), "milyar")
    d = hemocytometer(230, 5, 100); v = viability_stain(212, 18)
    print("TV3.2 sayım 5 karede 230 hücre, 1:100 ->", "%.3e" % d, "hücre/mL; canlılık", r(v, 3))
    print("TV3.3 gereken slurry:", r(slurry_volume_mL(n, d, v)/1000, 2), "L")
    print("TV3.4 ölçümsüz tahmin (1.0e9 hücre/mL, canlılık .90):", r(slurry_volume_mL(n, None, 0.90)/1000, 2), "L")

    print("== 4 Profesyonel verim")
    sg = plato_to_sg(12.0)
    print("TV4.1 12 °P SG", r(sg, 4), "; 1040 L sıcak -> ekstrakt", r(extract_kg(1040, 12.0), 2), "kg")
    sha = brewhouse_yield(1040, 12.0, 170)
    lab = lab_extract_as_is(81.5, 4.3)
    print("TV4.2 brewhouse verimi (170 kg malt):", r(sha, 2), "%; lab as-is", r(lab, 2), "%; oran", r(lab_ratio(sha, lab)*100, 1), "%")
    fwmax = first_wort_max_P(170, lab, 4.3, 520)
    ce = conversion_eff(19.6, fwmax)
    print("TV4.3 ilk şıra maks", r(fwmax, 2), "°P; ölçülen 19.6 -> dönüşüm", r(ce*100, 1), "%; süzme", r(lauter_eff(sha, lab, ce)*100, 1), "%")

    print("== 5 Malt analizi")
    print("TV5.1 FGDB 81.5, nem 4.3, ince/kaba 1.5:", {k: r(v, 4 if k.startswith("pot") else 2) for k, v in malt_from_analysis(81.5, 4.3, 1.5).items()})
    print("TV5.2 renk 3.5 EBC:", {k: r(v, 2) for k, v in ebc_from_lab(3.5).items()})

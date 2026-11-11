"""Bölüm 05 (mash, hacimler) parite vektörleri. Referans: formuller/testler/05_mash_tv.py (ADR-0003)."""
from ortak import Vektor, yukle

# Tun: (kütle kg, özgül ısı cal/g°C, hacim L). Sıfır = tun ısısı yok sayılır.
TUN_KUCUK = (4.0, 0.12, 40.0)
TUN_VARSAYILAN = (160 * 0.4535924 / 16, 0.3, 640 * 29.57352956 / 1000.0)  # 160 oz, 0.3, 640 fl oz
TUN_DOLU = (4.0, 0.12, 12.0)   # mash tun'un %80 kapasitesini aşar -> düzeltme yok


def _tun(t):
    return {"m_tun": t[0], "c_tun": t[1], "v_tun": t[2]}


def uret():
    ref = yukle("05_mash_tv.py")
    v = Vektor("05", "formuller/testler/05_mash_tv.py")

    # Sabitler (iç birimler; L/kg ve cal/g°C)
    for ad in ("FLOZ_L", "LB_KG", "OZ_KG", "GRAIN_VOL", "C_MALT", "ABS_STD", "ABS_BIAB"):
        v.ekle("constant", {"name": ad}, getattr(ref, ad))

    # tun_eff (§2.0)
    for m, vm, vt in ((4.0, 10.0, 40.0), (4.0, 32.0, 40.0), (4.0, 40.0, 40.0), (4.0, 5.0, 0.0),
                      (0.0, 10.0, 40.0), (4.536, 20.0, 18.927), (4.0, 31.9999, 40.0), (3.0, 0.0, 20.0)):
        v.ekle("tun_eff", {"m_tun": m, "v_mash": vm, "v_tun": vt}, ref.tun_eff(m, vm, vt))

    # strike_temp (§2.1); t_tun verilmezse tahıl sıcaklığı; vw = 0 -> None
    strike = [
        dict(vw=15, t_step=66, w=5, t_grain=20),
        dict(vw=15, t_step=66, w=5, t_grain=20, **_tun(TUN_KUCUK), t_tun=20),
        dict(vw=15, t_step=66, w=5, t_grain=20, **_tun(TUN_VARSAYILAN), t_tun=20),
        dict(vw=15, t_step=66, w=5, t_grain=20, **_tun(TUN_KUCUK)),
        dict(vw=15, t_step=66, w=5, t_grain=20, **_tun(TUN_DOLU), t_tun=25),
        dict(vw=0, t_step=66, w=5, t_grain=20),
        dict(vw=0, t_step=66, w=5, t_grain=20, **_tun(TUN_KUCUK), t_tun=20),
        dict(vw=12.5, t_step=67.5, w=4.2, t_grain=18.5, **_tun(TUN_VARSAYILAN), t_tun=22.0),
        dict(vw=30, t_step=64, w=8, t_grain=5, **_tun(TUN_KUCUK), t_tun=5),
        dict(vw=0.5, t_step=70, w=0, t_grain=20),
        dict(vw=20, t_step=66, w=5, t_grain=20, m_tun=3.0, c_tun=0.0, v_tun=0.0),
        dict(vw=20, t_step=66, w=5, t_grain=20, m_tun=3.0, c_tun=0.12, v_tun=0.0, t_tun=10),
        dict(vw=10, t_step=55, w=3, t_grain=25, **_tun(TUN_KUCUK), t_tun=40),
    ]
    for g in strike:
        v.ekle("strike_temp", g, ref.strike_temp(g["vw"], g["t_step"], g["w"], g["t_grain"], g.get("m_tun", 0),
                                                 g.get("c_tun", 0), g.get("v_tun", 0), g.get("t_tun")))

    # infusion_temp (§2.2); va <= 0 -> None
    inf = [
        dict(va=4, t2=72, vw=15, t1=66, w=5),
        dict(va=4, t2=72, vw=15, t1=66, w=5, **_tun(TUN_KUCUK)),
        dict(va=4, t2=72, vw=15, t1=66, w=5, **_tun(TUN_VARSAYILAN)),
        dict(va=0, t2=72, vw=15, t1=66, w=5),
        dict(va=-1, t2=72, vw=15, t1=66, w=5),
        dict(va=2.5, t2=78, vw=10, t1=63, w=4.2, **_tun(TUN_DOLU)),
        dict(va=8, t2=66, vw=12, t1=50, w=5, **_tun(TUN_KUCUK)),
        dict(va=1, t2=76, vw=20, t1=66, w=5),
        dict(va=3, t2=66, vw=0, t1=66, w=5),
        dict(va=3, t2=60, vw=15, t1=66, w=5),
    ]
    for g in inf:
        v.ekle("infusion_temp", g, ref.infusion_temp(g["va"], g["t2"], g["vw"], g["t1"], g["w"],
                                                     g.get("m_tun", 0), g.get("c_tun", 0), g.get("v_tun", 0)))

    # infusion_amount (§2.3)
    amt = [
        dict(tinf=100, t2=72, vw=15, t1=66, w=5),
        dict(tinf=100, t2=72, vw=15, t1=66, w=5, **_tun(TUN_KUCUK)),
        dict(tinf=95, t2=72, vw=15, t1=66, w=5, **_tun(TUN_VARSAYILAN)),
        dict(tinf=100, t2=100, vw=15, t1=66, w=5),
        dict(tinf=100, t2=66, vw=15, t1=66, w=5),
        dict(tinf=100, t2=60, vw=15, t1=66, w=5),
        dict(tinf=100, t2=105, vw=15, t1=66, w=5),
        dict(tinf=40, t2=50, vw=10, t1=66, w=5),
        dict(tinf=40, t2=55, vw=10, t1=66, w=5),
        dict(tinf=98.5, t2=68.5, vw=7.25, t1=63.0, w=3.3, **_tun(TUN_DOLU)),
    ]
    for g in amt:
        v.ekle("infusion_amount", g, ref.infusion_amount(g["tinf"], g["t2"], g["vw"], g["t1"], g["w"],
                                                         g.get("m_tun", 0), g.get("c_tun", 0), g.get("v_tun", 0)))

    # decoct_vol (§2.4); tb varsayılan 100
    dec = [
        dict(t2=72, vw=15, t1=66, w=5),
        dict(t2=66, vw=15, t1=50, w=5),
        dict(t2=72, vw=15, t1=66, w=5, **_tun(TUN_KUCUK)),
        dict(t2=72, vw=15, t1=66, w=5, **_tun(TUN_VARSAYILAN), tb=98.5),
        dict(t2=95, vw=15, t1=50, w=5),
        dict(t2=72, vw=15, t1=100, w=5),
        dict(t2=72, vw=15, t1=105, w=5),
        dict(t2=72, vw=0, t1=66, w=0),
        dict(t2=72, vw=10, t1=66, w=0),
        dict(t2=72, vw=0, t1=66, w=5),
        dict(t2=78, vw=12.5, t1=62, w=4.2, **_tun(TUN_DOLU), tb=99.0),
        dict(t2=66, vw=15, t1=66, w=5),
    ]
    for g in dec:
        v.ekle("decoct_vol", g, ref.decoct_vol(g["t2"], g["vw"], g["t1"], g["w"], g.get("m_tun", 0),
                                               g.get("c_tun", 0), g.get("v_tun", 0), g.get("tb", 100.0)))

    # boil_temp_c (§6.1)
    for ft in (0.0, 1000 / 0.3048, 500.0, 5280.0, 10000.0, -100.0):
        v.ekle("boil_temp_c", {"alt_ft": ft}, ref.boil_temp_C(ft))

    # equip_boil_vol (§5.2)
    for batch, topup, trub, cool, bo in ((23, 0, 1, 4, 4.0), (23, 3, 1, 4, 5.5), (20, 0, 0, 0, 0.0),
                                         (19, 1, 2, 99, 3.0), (19, 1, 2, 120, 3.0), (40, 0, 1.5, 4, 8.0),
                                         (25, 0, 1, 3.5, 4.25)):
        v.ekle("equip_boil_vol", {"batch": batch, "topup": topup, "trub": trub, "cool_pct": cool,
                                  "boil_off_total": bo}, ref.equip_boil_vol(batch, topup, trub, cool, bo))

    # batch_sparge ve sparge_plan (§4.2)
    bv = ref.equip_boil_vol(23, 0, 1, 4, 4.0)
    absorb = ref.ABS_STD * 5
    mash = 16
    sparge = bv + absorb + 0 + 1 - 0 - mash
    temel = dict(S=sparge, absorb=absorb, W=5, tun_vol=40, pct=95, mash_water=mash, even=True, drain=False)
    senaryolar = [
        temel,
        dict(temel, even=False),
        dict(temel, drain=True),
        dict(temel, drain=True, even=False),
        dict(temel, tun_vol=25),
        dict(temel, tun_vol=25, even=False),
        dict(temel, tun_vol=25, drain=True),
        dict(temel, tun_vol=15),
        dict(temel, tun_vol=15, even=False),
        dict(temel, S=-3.0),
        dict(temel, S=0.0, even=False),
        dict(temel, mash_water=8, tun_vol=60),
        dict(temel, mash_water=8, tun_vol=60, even=False),
        dict(temel, mash_water=30, tun_vol=40),
        dict(temel, mash_water=30, tun_vol=40, even=False),
        dict(temel, S=60.0, tun_vol=20, even=True),
        dict(temel, S=60.0, tun_vol=20, even=False),
        dict(temel, S=60.0, tun_vol=20, drain=True),
        dict(temel, S=12.7, tun_vol=0.0, pct=0.0),
        dict(temel, S=12.7, tun_vol=0.0, pct=0.0, drain=True),
        dict(temel, pct=100, W=8, absorb=ref.ABS_STD * 8, S=20.0, mash_water=24.0, tun_vol=30),
        dict(temel, pct=100, W=8, absorb=ref.ABS_STD * 8, S=20.0, mash_water=24.0, tun_vol=30, even=False),
    ]
    for s in senaryolar:
        for i in (0, 1, 2, 3, 4, 11):
            vol, bayrak = ref.batch_sparge(i, **s)
            v.ekle("batch_sparge", dict(s, i=i), [vol, bayrak])
        v.ekle("sparge_plan", s, [list(x) for x in ref.sparge_plan(**s)])

    # parti (§7)
    for og, vol, f1 in ((1.060, 40, 60), (1.080, 50, 50), (1.050, 20, 75), (1.045, 30, 0), (1.045, 30, 100),
                        (1.100, 25, 33.3), (1.0, 40, 60)):
        v.ekle("parti", {"og": og, "vol": vol, "f1": f1}, ref.parti(og, vol, f1))

    return v.yaz()

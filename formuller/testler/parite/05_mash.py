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

    # --- T1.14b: sonradan eklenen referans fonksiyonları (her biri girdi adlarıyla çağrılır) ---
    def k(fonk, **g):
        v.ekle(fonk, g, getattr(ref, fonk)(**g))

    for w in (0.0, 5.0, 4.2, 12.345):
        k("grain_vol", W=w)
    for vsu, w in ((15.0, 5.0), (0.0, 0.0), (22.5, 7.3), (0.0, 4.0)):
        k("mash_vol", Vsu=vsu, W=w)
    for w in (0.0, 5.0, 3.3):
        k("grain_absorb", W_mash=w)
        k("grain_absorb", W_mash=w, biab=True)
        k("sparge_absorb", W_sparge=w)
        k("sparge_absorb", W_sparge=w, biab=True)

    gr = [[0, 0, 4.0], [0, 3, 0.5], [0, 1, 9.0], [0, 2, 9.0], [1, 0, 7.0], [2, 0, 1.0], [8, 0, 1.5], [8, 3, 0.25],
          [8, 4, 3.0], [-1, 0, 0.75], [4, 3, 0.2]]
    k("mash_grain_weight", grains=gr)
    k("mash_grain_weight", grains=[])
    k("mash_grain_weight", grains=[[8, 0, 2.0]])
    k("sparge_grain_weight", grains=gr)
    k("sparge_grain_weight", grains=[])
    k("sparge_grain_weight", grains=[[0, 0, 2.0]])

    for mtw, wm, ws in ((16.0, 5.0, 0.0), (35.0, 8.0, 1.5), (0.0, 0.0, 0.0)):
        k("tun_vol_needed", mash_tot_water=mtw, W_mash=wm)
        k("tun_vol_needed_recipe", mash_tot_water=mtw, W_mash=wm, W_sparge=ws)
    for i, a in ((15.0, 0.0), (15.0, 1.0), (0.0, 0.0), (4.5, 0.5)):
        k("step_tot_infusion", infusion=i, tun_addition=a)
    k("mash_tot_water", steps=[[0, 15.0, 1.0], [0, 4.0, 0.0], [0, 3.0, 0.0]])
    k("mash_tot_water", steps=[[0, 12.0, 0.0], [1, 5.0, 0.0], [2, 2.0, 0.0], [0, 1.0, 0.5]])
    k("mash_tot_water", steps=[[1, 5.0, 0.0]])
    k("mash_tot_water", steps=[])
    for tw, a, d in ((16.0, 5.007, 1.0), (35.0, 8.0, 0.0), (3.0, 5.0, 1.0)):
        k("mash_avail_water", tot_water=tw, absorb=a, tun_deadspace=d)

    # §2.2 kaynama üstü bayrağı
    for g in (dict(Va=4, T2=72, Vw=15, T1=66, W=5),
              dict(Va=1, T2=76, Vw=20, T1=66, W=5),
              dict(Va=0, T2=72, Vw=15, T1=66, W=5),
              dict(Va=-1, T2=72, Vw=15, T1=66, W=5),
              dict(Va=2.5, T2=78, Vw=10, T1=63, W=4.2, m_tun=4.0, c_tun=0.12, V_tun=12.0),
              dict(Va=8, T2=66, Vw=12, T1=50, W=5, m_tun=4.0, c_tun=0.12, V_tun=40.0),
              dict(Va=3, T2=60, Vw=15, T1=66, W=5)):
        k("infusion_temp_checked", **g)

    # §2.5 tek adım güncelleme
    for g in (dict(kind=0, infusion=15.0, tun_addition=0.0, t_step=66.0, start_temp=0.0, start_vol=0.0, W=5.0, T_grain=20.0),
              dict(kind=0, infusion=15.0, tun_addition=1.0, t_step=66.0, start_temp=0.0, start_vol=0.0, W=5.0, T_grain=20.0,
                   m_tun=4.0, c_tun=0.12, V_tun=40.0, T_tun=20.0),
              dict(kind=0, infusion=4.0, tun_addition=0.0, t_step=72.0, start_temp=66.0, start_vol=15.0, W=5.0, T_grain=20.0),
              dict(kind=0, infusion=0.0, tun_addition=0.0, t_step=72.0, start_temp=66.0, start_vol=15.0, W=5.0, T_grain=20.0),
              dict(kind=1, infusion=0.0, tun_addition=0.0, t_step=72.0, start_temp=66.0, start_vol=15.0, W=5.0, T_grain=20.0),
              dict(kind=2, infusion=3.0, tun_addition=0.0, t_step=70.0, start_temp=66.0, start_vol=15.0, W=5.0, T_grain=20.0)):
        k("update_infusion_temp", **g)

    # §2.5 toplu yeniden hesap (RecalculateMash)
    S3 = [dict(type=0, infusion=15.0, t_step=66.0), dict(type=0, infusion=4.0, t_step=72.0),
          dict(type=0, infusion=3.0, t_step=76.0)]
    SDEC = [dict(type=0, infusion=15.0, t_step=50.0), dict(type=1, infusion=0.0, t_step=66.0),
            dict(type=0, infusion=3.0, t_step=72.0), dict(type=2, infusion=2.0, t_step=78.0)]
    SDEC0 = [dict(type=1, infusion=12.0, t_step=64.0), dict(type=0, infusion=5.0, t_step=72.0)]
    SHOT = [dict(type=0, infusion=15.0, t_step=66.0), dict(type=0, infusion=1.0, t_step=78.0)]
    k("recalc_mash", steps=S3, W=5.0, T_grain=20.0)
    k("recalc_mash", steps=S3, W=5.0, T_grain=20.0, deadspace=1.0)
    k("recalc_mash", steps=S3, W=5.0, T_grain=20.0, m_tun=4.0, c_tun=0.12, V_tun=40.0, T_tun=20.0, deadspace=1.0)
    k("recalc_mash", steps=S3, W=5.0, T_grain=20.0, m_tun=TUN_VARSAYILAN[0], c_tun=0.3, V_tun=TUN_VARSAYILAN[2], T_tun=22.0)
    k("recalc_mash", steps=SDEC, W=5.0, T_grain=20.0, m_tun=4.0, c_tun=0.12, V_tun=40.0, Tb=99.0)
    k("recalc_mash", steps=SDEC, W=5.0, T_grain=20.0)
    k("recalc_mash", steps=SDEC0, W=4.0, T_grain=18.0)
    k("recalc_mash", steps=SHOT, W=5.0, T_grain=20.0)
    k("recalc_mash", steps=[dict(type=0, infusion=0.0, t_step=66.0)], W=5.0, T_grain=20.0)
    k("recalc_mash", steps=[], W=5.0, T_grain=20.0)
    # T1.12 B15: mash'te su yokken dekoksiyon (no_water bayrağı)
    SNOW = [dict(type=2, infusion=0.0, t_step=55.0), dict(type=1, infusion=0.0, t_step=66.0),
            dict(type=0, infusion=10.0, t_step=66.0), dict(type=1, infusion=0.0, t_step=72.0)]
    k("recalc_mash", steps=SNOW, W=5.0, T_grain=20.0)
    k("recalc_biab", steps=SNOW, W=5.0, T_grain=20.0, biab_vol=20.0)

    # §3.2 BIAB adım dağıtımı
    k("recalc_biab", steps=[dict(type=0, infusion=0.0, t_step=66.0)], W=5.0, T_grain=20.0, biab_vol=33.057)
    k("recalc_biab", steps=S3, W=5.0, T_grain=20.0, biab_vol=33.057)
    k("recalc_biab", steps=S3, W=5.0, T_grain=20.0, biab_vol=33.057, m_tun=4.0, c_tun=0.12, V_tun=40.0, T_tun=20.0)
    k("recalc_biab", steps=SDEC, W=5.0, T_grain=20.0, biab_vol=30.0, Tb=99.0)
    k("recalc_biab", steps=SDEC0, W=4.0, T_grain=18.0, biab_vol=25.0)
    k("recalc_biab", steps=SHOT, W=5.0, T_grain=20.0, biab_vol=20.0)
    k("recalc_biab", steps=[dict(type=2, infusion=0.0, t_step=66.0), dict(type=0, infusion=2.0, t_step=70.0)],
      W=5.0, T_grain=20.0, biab_vol=18.0)
    k("recalc_biab", steps=S3, W=5.0, T_grain=20.0, biab_vol=5.0)

    # §2.6
    for a, adj, d in ((0.0, True, 1.0), (0.5, True, 1.0), (0.5, False, 1.0), (0.0, False, 0.0)):
        k("tun_deadspace_profile", tun_addition=a, adj_deadspace=adj, tun_deadspace=d)
    for g in (dict(infusions=[15.0, 4.0, 0.0, 3.0], W_old=5.0, W_new=6.0),
              dict(infusions=[15.0, 4.0], W_old=5.0, W_new=5.0),
              dict(infusions=[15.0, 4.0], W_old=0.0, W_new=5.0),
              dict(infusions=[15.0, 4.0], W_old=5.0, W_new=0.0),
              dict(infusions=[15.0, 4.0], W_old=5.0, W_new=8.0, biab=True),
              dict(infusions=[15.0, -2.0, 0.0], W_old=5.0, W_new=2.5),
              dict(infusions=[], W_old=5.0, W_new=6.0)):
        k("scale_from_grain_weight", **g)

    # §2.7
    for sv, inf, w, dec in ((0.0, 15.0, 5.0, False), (15.0, 4.0, 5.0, False), (15.0, 4.0, 5.0, True),
                            (15.0, 4.0, 0.0, False), (0.0, 0.0, 5.0, True)):
        k("step_water_grain", start_vol=sv, infusion=inf, W=w, decoction=dec)
    for g in (dict(r=3.0, W=5.0, start_vol=0.0, infusion_old=15.0),
              dict(r=3.5, W=5.0, start_vol=0.0, infusion_old=15.0),
              dict(r=3.0, W=5.0, start_vol=15.0, infusion_old=4.0),
              dict(r=3.006, W=5.0, start_vol=0.0, infusion_old=15.0),
              dict(r=3.0, W=5.0, start_vol=0.0, infusion_old=15.02),
              dict(r=2.5, W=5.0, start_vol=15.0, infusion_old=4.0)):
        k("scale_water_grain", **g)

    # §2.8: step = {"w","t_step","start_vol","tot_infusion","decoction"}
    def ms(t, sv, ti, dec=False, w=5.0):
        return dict(w=w, t_step=t, start_vol=sv, tot_infusion=ti, decoction=dec)
    k("main_mash_thickness", steps=[ms(66.0, 0.0, 15.0), ms(72.0, 15.0, 4.0)], tot_water=19.0, W_total=5.0)
    k("main_mash_thickness", steps=[ms(50.0, 0.0, 12.0), ms(67.0, 12.0, 3.0), ms(78.0, 15.0, 2.0)], tot_water=17.0, W_total=5.0)
    k("main_mash_thickness", steps=[ms(67.5, 0.0, 12.0), ms(66.0, 12.0, 6.0)], tot_water=18.0, W_total=5.0)
    k("main_mash_thickness", steps=[ms(64.0, 0.0, 10.0), ms(71.0, 10.0, 5.0)], tot_water=15.0, W_total=5.0)
    k("main_mash_thickness", steps=[ms(66.0, 12.0, 0.0, True), ms(72.0, 12.0, 3.0)], tot_water=15.0, W_total=5.0)
    k("main_mash_thickness", steps=[ms(20.0, 0.0, 15.0), ms(100.0, 15.0, 4.0)], tot_water=19.0, W_total=5.0)
    k("main_mash_thickness", steps=[ms(66.0, 0.0, 15.0, w=0.0)], tot_water=15.0, W_total=5.0)
    k("main_mash_thickness", steps=[], tot_water=15.0, W_total=0.0)
    k("main_mash_thickness", steps=[], tot_water=0.0, W_total=0.0)

    # §3.1, §4.1
    for pb, d, tk, w in ((29.0, 1.0, 0.0, 5.0), (29.0, 0.0, 2.0, 5.0), (25.5, 1.5, 1.0, 0.0)):
        k("biab_mash_vol", v_preboil=pb, tun_deadspace=d, topup_kettle=tk, W_mash=w)
    for g in (dict(v_preboil=29.0, absorb=5.007, sparge_absorb_=0.0, tun_deadspace=1.0, topup_kettle=0.0, mash_tot_water_=16.0),
              dict(v_preboil=29.0, absorb=3.057, sparge_absorb_=1.0, tun_deadspace=0.0, topup_kettle=2.0, mash_tot_water_=33.057),
              dict(v_preboil=20.0, absorb=8.0, sparge_absorb_=0.0, tun_deadspace=0.5, topup_kettle=0.0, mash_tot_water_=40.0)):
        k("sparge_vol", **g)

    # §5.1-5.8
    for bo, t, flag in ((4.0, 60.0, True), (4.0, 90.0, True), (4.0, 60.0, False), (5.5, 45.0, False), (0.0, 60.0, True)):
        k("tot_boil_off", boil_off=bo, boil_time_min=t, rate_flag=flag)
    for pb, bo in ((29.0, 4.0), (20.0, 0.0), (10.0, 12.0)):
        k("post_boil_vol", v_preboil=pb, boil_off_total=bo)
    for c, pv in ((4.0, 25.0), (0.0, 25.0), (3.5, 18.2), (120.0, 10.0), (4.0, 0.0)):
        k("cool_loss", cool_pct=c, v_postboil=pv)
    for b, t in ((23.0, 1.0), (19.0, 0.0), (0.0, 0.0)):
        k("pre_trub_vol", batch=b, trub=t)
    for bo, pb, t in ((4.0, 29.0, 60.0), (4.0, 29.0, 90.0), (0.0, 29.0, 60.0), (4.0, 0.0, 60.0), (4.0, -3.0, 60.0),
                      (4.0, 29.0, 0.0), (4.0, 29.0, -5.0), (2.5, 25.0, 60.0)):
        k("evap_rate", boil_off_total=bo, v_preboil=pb, boil_time_min=t)
    for b, fl in ((23.0, 0.0), (23.0, 1.5), (19.0, 2.0)):
        k("bottle_vol_equip", batch=b, fermenter_loss=fl)
    for g in (dict(batch=23.0, fermenter_loss=1.0, include_starter=False, starter=2.0, use_second=True, second_starter=1.0),
              dict(batch=23.0, fermenter_loss=1.0, include_starter=True, starter=2.0, use_second=False, second_starter=1.0),
              dict(batch=23.0, fermenter_loss=1.0, include_starter=True, starter=2.0, use_second=True, second_starter=1.5),
              dict(batch=19.0, fermenter_loss=0.0, include_starter=True, starter=0.0, use_second=True, second_starter=0.0)):
        k("bottle_vol_recipe", **g)
    base = dict(has_boil_ing=True, mash_tot_water_=16.0, sparge=19.007, topup_kettle=1.0, topup_ferm=2.0, batch=23.0,
                v_preboil=29.0)
    for rt in (0, 1, 2, 3, 7):
        for hb in (False, True):
            k("total_water", recipe_type=rt, **dict(base, has_boil_ing=hb))
    k("total_water", recipe_type=2, **dict(base, mash_tot_water_=0.0, sparge=0.0, topup_kettle=0.0, topup_ferm=0.0))

    # §5.9
    for items in ([[2, 1.0], [2, 0.5], [1, 3.0], [4, 2.0], [5, 1.0]], []):
        for use in (1, 2, 4, 5):
            k("total_grain_use", items=items, use=use)
    for pb, bg, ws in ((29.0, 0.0, 1.0), (29.0, 0.0, 5.0), (29.0, 1.5, 3.0), (10.0, 2.0, 5.0), (29.0, 0.0, 0.0)):
        k("steep_vol", v_preboil=pb, v_boilgrains=bg, W_steep=ws)
    for pb, sv, bg in ((29.0, 8.345, 0.0), (29.0, 5.0, 1.5), (10.0, 12.0, 0.0)):
        k("boil_water_add", v_preboil=pb, v_steep=sv, v_boilgrains=bg)
    for rt in (0, 1, 2, 3, 5):
        for hb in (False, True):
            k("primary_water_add", recipe_type=rt, has_boil_ing=hb, batch=23.0, v_postboil=25.0, v_primarygrains=1.2,
              topup_ferm=2.0)

    # §5.10
    for og, b, t, pb in ((1.050, 23.0, 1.0, 29.0), (1.080, 20.0, 0.0, 26.5), (1.050, 23.0, 1.0, 0.0), (1.050, 23.0, 1.0, -2.0),
                         (1.0, 23.0, 1.0, 29.0)):
        k("preboil_sg", og=og, batch=b, trub=t, v_preboil=pb)

    # §5.11 zincir (bileşik)
    for g in (dict(batch=23.0, topup_ferm=0.0, trub=1.0, cool_pct=4.0, boil_off=4.0, boil_time_min=60.0, rate_flag=False,
                   fermenter_loss=0.0),
              dict(batch=23.0, topup_ferm=3.0, trub=1.0, cool_pct=4.0, boil_off=5.5, boil_time_min=90.0, rate_flag=True,
                   fermenter_loss=1.0),
              dict(batch=40.0, topup_ferm=0.0, trub=1.5, cool_pct=99.0, boil_off=8.0, boil_time_min=60.0, rate_flag=True,
                   fermenter_loss=0.5),
              dict(batch=19.0, topup_ferm=1.0, trub=2.0, cool_pct=0.0, boil_off=0.0, boil_time_min=0.0, rate_flag=False,
                   fermenter_loss=0.0)):
        k("volume_chain", **g)

    # §5.12 Water Needed
    for vf, tu, c in ((23.0, 0.0, 4.0), (23.0, 3.0, 4.0), (20.0, 0.0, 0.0), (25.0, 2.0, 3.5)):
        k("wn_cool_loss", v_ferm=vf, topup=tu, cool_pct=c)
    for vf, tu, tr, vc, bo in ((23.0, 0.0, 1.0, 0.958, 4.0), (23.0, 3.0, 1.0, 0.8, 5.0), (20.0, 0.0, 0.0, 0.0, 0.0)):
        k("wn_boil_vol", v_ferm=vf, topup=tu, trub=tr, v_cool=vc, boil_off=bo)
    for bo, vb in ((4.0, 28.958), (0.0, 28.958), (4.0, 0.0), (4.0, 10.0)):
        k("wn_boil_pct", boil_off=bo, v_boil=vb)
    for w in (0.0, 5.0, 3.3):
        k("wn_grain_absorb", W=w)
    for g in (dict(v_boil=28.958, absorb=5.007, topup_kettle=0.0, lauter_deadspace=1.0, v_mash=15.0),
              dict(v_boil=28.958, absorb=5.007, topup_kettle=2.0, lauter_deadspace=0.0, v_mash=35.0)):
        k("wn_sparge_water", **g)
    for bt in (0, 1, 2):
        k("wn_total_water", brew_type=bt, v_boil=28.958, topup=0.0, v_sparge=19.966, v_mash=15.0, topup_kettle=0.0)
        k("wn_total_water", brew_type=bt, v_boil=28.958, topup=2.0, v_sparge=19.966, v_mash=15.0, topup_kettle=1.0)
    for vf, vl in ((23.0, 0.0), (23.0, 1.0), (19.0, 2.5)):
        k("wn_bottle_vol", v_ferm=vf, v_loss_ferm=vl)

    # §6.2
    for g in (dict(v0=25.0, evap_pct=10.0, boil_time_min=60.0, cool_pct=4.0, sg0=1.050),
              dict(v0=25.0, evap_pct=10.0, boil_time_min=90.0, cool_pct=0.0, sg0=1.040),
              dict(v0=30.0, evap_pct=8.5, boil_time_min=75.0, cool_pct=3.5, sg0=1.062),
              dict(v0=25.0, evap_pct=0.0, boil_time_min=60.0, cool_pct=4.0, sg0=1.050),
              dict(v0=25.0, evap_pct=100.0, boil_time_min=60.0, cool_pct=0.0, sg0=1.050),
              dict(v0=0.0, evap_pct=10.0, boil_time_min=60.0, cool_pct=4.0, sg0=1.050)):
        k("boil_off_tool", **g)

    # §8
    for g in (dict(sg0=1.060, v0=20.0, v_add=5.0, wort=False),
              dict(sg0=1.060, v0=20.0, v_add=5.0, wort=True, sg_add=1.040),
              dict(sg0=1.060, v0=20.0, v_add=-2.0, wort=False),
              dict(sg0=1.060, v0=20.0, v_add=-2.0, wort=True, sg_add=1.040),
              dict(sg0=1.060, v0=20.0, v_add=0.0, wort=False),
              dict(sg0=1.050, v0=10.0, v_add=10.0, wort=True, sg_add=1.080)):
        k("dilution", **g)
    for vv, sg, tg in ((20.0, 1.060, 1.050), (20.0, 1.040, 1.050), (20.0, 1.050, 1.050), (25.0, 1.075, 1.045)):
        k("gravity_adj_water_vol", v=vv, sg=sg, sg_target=tg)

    # §9
    for u in range(0, 13):
        k("misc_to_fluid_oz", amount=2.5, units=u)
    k("misc_to_fluid_oz", amount=0.0, units=1)
    k("misc_to_fluid_oz", amount=3.0, units=13)
    k("misc_to_fluid_oz", amount=3.0, units=-1)

    return v.yaz()

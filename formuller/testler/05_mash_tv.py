FLOZ_L = 29.57352956/1000.0
LB_KG  = 0.4535924
OZ_KG  = LB_KG/16
GRAIN_VOL = 0.652          # O_GRAIN_VOL  L/kg
C_MALT    = 0.380          # O_MALT_SPECIFIC_HEAT cal/g°C
ABS_STD   = 0.96*FLOZ_L/OZ_KG   # O_ADV_GRAIN_ABSORB -> L/kg
ABS_BIAB  = 0.586*FLOZ_L/OZ_KG  # O_ADV_BIAB_ABSORB  -> L/kg

def tun_eff(m_tun, v_mash, v_tun):
    cap = 0.8*v_tun
    return m_tun*v_mash/cap if (cap > 0 and v_mash < cap) else m_tun

def strike_temp(Vw, T_step, W, T_grain, m_tun=0, c_tun=0, V_tun=0, T_tun=None):
    if Vw == 0: return None
    T_tun = T_grain if T_tun is None else T_tun
    me = tun_eff(m_tun, GRAIN_VOL*W + Vw, V_tun)
    return T_step + ((T_step-T_tun)*c_tun*me + C_MALT*W*(T_step-T_grain))/Vw

def infusion_temp(Va, T2, Vw, T1, W, m_tun=0, c_tun=0, V_tun=0):
    if Va <= 0: return None
    me = tun_eff(m_tun, GRAIN_VOL*W + Va + Vw, V_tun)
    return T2 + (T2-T1)*(c_tun*me + C_MALT*W + Vw)/Va

def infusion_amount(Tinf, T2, Vw, T1, W, m_tun=0, c_tun=0, V_tun=0):
    if not (min(T1,Tinf) < T2 < max(T1,Tinf)): return 0.0
    me = tun_eff(m_tun, GRAIN_VOL*W + Vw, V_tun)
    return (T2-T1)*(c_tun*me + C_MALT*W + Vw)/(Tinf-T2)

def decoct_vol(T2, Vw, T1, W, m_tun=0, c_tun=0, V_tun=0, Tb=100.0):
    if not T1 < Tb: return 0.0
    if W == 0 and Vw == 0: return 0.0
    Vm = GRAIN_VOL*W + Vw
    me = tun_eff(m_tun, Vm, V_tun)
    f = (T2-T1)/(Tb-T1) * (c_tun*me + C_MALT*W + Vw)/(C_MALT*W + Vw)
    return Vm*min(f, 1.0)

def boil_temp_C(alt_ft): return 100.0 - alt_ft*0.3048*0.00324

def equip_boil_vol(batch, topup, trub, cool_pct, boil_off_total):
    c = min(cool_pct, 99.0)
    return boil_off_total + (batch - topup + trub)/(1 - c/100)

def batch_sparge(i, S, absorb, W, tun_vol, pct, mash_water, even, drain):
    """GetBatchSpargeSize; returns (vol, drain_flag)"""
    S = max(S, 0.0)
    Vg = GRAIN_VOL*W
    cap = pct*tun_vol/100 - Vg - absorb
    if cap < 0: cap = pct*tun_vol/100
    M = mash_water - absorb
    R = cap - M
    if not (i != 0 or (R > 0 and not drain)):
        return M, True
    if even and not drain:
        n = 2.0
        size = (S+M)/2 if (R >= 0 or i < 1) else S
        while cap < size and n < 10:
            n += 1
            size = (S+M)/n if (R >= 0 or i < 1) else S/(n-1)
        if i == 0:
            return (size-M, False) if size-M >= 0 else (M, True)
        return (0.0 if int(n+0.3) <= i else size), False
    R = max(R, 0.0)
    if i == 0: return min(S, R), False
    if not drain and not even: S -= R
    if not even:
        if (i-1)*cap <= S: return min(cap, S-(i-1)*cap), False
        return 0.0, False
    if cap <= 0: cap = 1.0
    n = int(S/cap + 0.99)
    return (0.0 if n < i else S/n), False

def sparge_plan(**kw):
    out=[]
    for i in range(50):
        v,f = batch_sparge(i, **kw)
        if not f and v <= 0: break
        out.append((i, round(v,3), 'DRAIN(ilk süzme)' if f else 'EKLE'))
    return out

def parti(OG, vol, f1):
    k = 0.5**(3*f1/100)
    return dict(vol1=vol*f1/100, vol2=vol*(100-f1)/100,
                og1=1+(OG-1)*(1-0.5*k), og2=1+(OG-1)*0.5*k)

r=lambda x: round(x,3)
print("Sabitler: ABS_STD L/kg", r(ABS_STD), " ABS_BIAB L/kg", r(ABS_BIAB), " floz/oz->L/kg", 1/(OZ_KG/FLOZ_L))
print("TV1 strike, tun yok:", r(strike_temp(15,66,5,20)))
print("TV2 strike, tun 4kg c=0.12 V=40L Ttun=20:", r(strike_temp(15,66,5,20,4,0.12,40,20)))
print("TV2b strike, tun varsayilan 4.536kg c=0.3 V=18.93L:", r(strike_temp(15,66,5,20,160*OZ_KG,0.3,640*FLOZ_L,20)))
a=infusion_amount(100,72,15,66,5); print("TV3 infuzyon 66->72 kaynar su (L):", r(a), " kontrol T:", r(infusion_temp(a,72,15,66,5)))
print("TV3b infuzyon 4 L ile 66->72 gereken T:", r(infusion_temp(4,72,15,66,5)))
print("TV4 dekoksiyon 66->72 (L mash):", r(decoct_vol(72,15,66,5)), " 50->66:", r(decoct_vol(66,15,50,5)))
print("TV5 kaynama 1000 m:", r(boil_temp_C(1000/0.3048)))
bv=equip_boil_vol(23,0,1,4,4.0); print("TV6 pre-boil:",r(bv)," post-boil:",r(bv-4)," cool loss:",r(0.04*(bv-4)))
absorb=ABS_STD*5; mash=15+1  # deadspace adj ile ilk adim 15+1
sparge = bv + absorb + 0 + 1 - 0 - mash
print("TV7 absorb",r(absorb)," mash",mash," sparge",r(sparge)," toplam su", r(mash+sparge+0+0), " mash tun hacmi", r(mash+GRAIN_VOL*5))
biab = bv + 1 - 0 + ABS_BIAB*5
print("TV8 BIAB toplam su",r(biab)," strike",r(strike_temp(biab,66,5,20)))
print("TV9 batch even:", sparge_plan(S=sparge,absorb=absorb,W=5,tun_vol=40,pct=95,mash_water=mash,even=1,drain=0))
print("TV9 batch fill:", sparge_plan(S=sparge,absorb=absorb,W=5,tun_vol=40,pct=95,mash_water=mash,even=0,drain=0))
print("TV9 drain+even:", sparge_plan(S=sparge,absorb=absorb,W=5,tun_vol=40,pct=95,mash_water=mash,even=1,drain=1))
print("TV9 kucuk tun 25L even:", sparge_plan(S=sparge,absorb=absorb,W=5,tun_vol=25,pct=95,mash_water=mash,even=1,drain=0))
print("TV9 kucuk tun 25L fill:", sparge_plan(S=sparge,absorb=absorb,W=5,tun_vol=25,pct=95,mash_water=mash,even=0,drain=0))
print("TV10 parti 60/40:", {k:r(v) for k,v in parti(1.060,40,60).items()})
print("TV11 dilution:", r(1+(1.060-1)*20/25), r(1+(0.060*20+0.040*5)/25))
print("TV12 preboil SG:", round(1+50*(23+1)/bv/1000,4))
print("TV13 boil tool 25L 10%/h 60dk 4% :", "off",r(60*10*25/6000),"cool",r(0.04*(25-2.5)),"end",r(25-2.5-0.04*22.5),"SG",round(1+0.050*25/(25-2.5-0.9),4))
wn_cool = (23-0)*0.04/0.96; wn_boil = 23-0+1+wn_cool+4
print("TV14 WN cool",r(wn_cool),"boil",r(wn_boil), "sparge", r(wn_boil+ABS_STD*5-0+1-15), "total", r(wn_boil+ABS_STD*5+1-15+15))
print("TV15 steep 1 kg 4 qt/lb ->L:", r(4*(1/LB_KG)*0.946352946))

# ---------------------------------------------------------------------------------------------
# T1.14b: spesifikasyonda (formuller/05-mash-hacimler.md) olup yukarıda referansı bulunmayan
# hesaplar. Metrik birimler (L, kg, °C); her fonksiyonun başında bölüm numarası yazılıdır.
# Yukarıdaki fonksiyonlar ve çıktılar değişmedi.
# ---------------------------------------------------------------------------------------------
ABOVE_BOILING_C = 100.0                 # §2.2: infüzyon suyu bu sıcaklığın üstündeyse hata
THICKNESS_CONST = 1.043192807606429     # §2.8 kod sabiti (fl oz/oz -> L/kg)
MASH_FG_ADJ_TEMP_F = 153.5              # §0.2 O_MASH_FG_ADJ_TEMP (°F)
STEEP_QT_PER_LB = 4.0                   # §0.2 O_ADV_STEEP_WATER_GRAIN
MASH_USE, SPARGE_USE = 0, 8             # §0.4 F_G_USE (mash: use < 1; sparge: koddan görülen 8)
MASHABLE_TYPES = (0, 3)                 # §0.4 F_G_TYPE: Grain, Adjunct
DECOCTION = 1                           # §0.4 F_MS_TYPE

# --- §1 tahıl hacmi ve emilimi ---
def grain_vol(W):                       # §1.1
    return GRAIN_VOL*W

def mash_vol(Vsu, W):                   # §1.1 MashVol
    return Vsu + GRAIN_VOL*W

def grain_absorb(W_mash, biab=False):   # §1.2
    return (ABS_BIAB if biab else ABS_STD)*W_mash

def sparge_absorb(W_sparge, biab=False):    # §1.3 (W_sparge = toplam sparge tahılı)
    return (ABS_BIAB if biab else ABS_STD)*W_sparge

def _kod(ad, deger, gecerli):
    """Karar (T1.16): listede olmayan kod ValueError."""
    if deger not in gecerli: raise ValueError("%s kodu geçersiz: %r" % (ad, deger))


def mash_grain_weight(grains):          # §2.6; grains = [(use, type, amount_kg)]
    for u, t, a in grains: _kod("kullanım", u, range(9)); _kod("tip", t, range(9))
    return sum(a for u, t, a in grains if u < MASH_USE+1 and t in MASHABLE_TYPES)

def sparge_grain_weight(grains):        # §1.3 (use == 8 ve type in {0, 3})
    for u, t, a in grains: _kod("kullanım", u, range(9)); _kod("tip", t, range(9))
    return sum(a for u, t, a in grains if u == SPARGE_USE and t in MASHABLE_TYPES)

def tun_vol_needed(mash_tot_water, W_mash):                 # §1.4
    return mash_tot_water + GRAIN_VOL*W_mash

def tun_vol_needed_recipe(mash_tot_water, W_mash, W_sparge):    # §1.4 reçete düzeyi
    return tun_vol_needed(mash_tot_water, W_mash) + GRAIN_VOL*W_sparge

def step_tot_infusion(infusion, tun_addition):              # §1.5
    return infusion + tun_addition

def mash_tot_water(steps):              # §1.5; steps = [(type, infusion, tun_addition)], dekoksiyon hariç
    return sum(step_tot_infusion(i, a) for t, i, a in steps if t != DECOCTION)

def mash_avail_water(tot_water, absorb, tun_deadspace):     # §1.6
    return tot_water - absorb - tun_deadspace

# --- §2 ısı dengesi eklentileri ---
def infusion_temp_checked(Va, T2, Vw, T1, W, m_tun=0, c_tun=0, V_tun=0):
    """§2.2: (T_inf, kaynama_ustu) ya da None (Va <= 0). kaynama_ustu = T_inf > 100 °C."""
    t = infusion_temp(Va, T2, Vw, T1, W, m_tun, c_tun, V_tun)
    return None if t is None else (t, t > ABOVE_BOILING_C)

def update_infusion_temp(kind, infusion, tun_addition, t_step, start_temp, start_vol, W, T_grain,
                         m_tun=0, c_tun=0, V_tun=0, T_tun=None):
    """§2.5 UpdateInfusionTemp (tek adım). start_temp == 0 -> ilk adım."""
    if kind == 0:
        if start_temp == 0:
            return strike_temp(infusion+tun_addition, t_step, W, T_grain, m_tun, c_tun, V_tun, T_tun)
        return infusion_temp(infusion, t_step, start_vol, start_temp, W, m_tun, c_tun, V_tun)
    return t_step

def recalc_mash(steps, W, T_grain, m_tun=0, c_tun=0, V_tun=0, T_tun=None, deadspace=0.0, Tb=100.0):
    """§2.5 RecalculateMash (BIAB değil). steps = [{"type","infusion","t_step"}]. Adım başına sözlük."""
    out = []; v_cum = 0.0; t_prev = None
    for i, s in enumerate(steps):
        kind, t_step = s["type"], s["t_step"]
        v_add = s["infusion"]; tun_add = 0.0
        if i == 0 and deadspace != 0:
            v_add += deadspace; tun_add = deadspace
        itemp = None; dec = 0.0
        if kind == 0 or (kind == DECOCTION and i == 0):
            if i == 0:
                itemp = strike_temp(v_add, t_step, W, T_grain, m_tun, c_tun, V_tun, T_tun)
            else:
                itemp = infusion_temp(v_add, t_step, v_cum, t_prev, W, m_tun, c_tun, V_tun)
        elif kind == DECOCTION:
            dec = decoct_vol(t_step, v_cum, t_prev, W, m_tun, c_tun, V_tun, Tb)
            itemp = 0.0
        out.append(dict(infusion=s["infusion"], tun_addition=tun_add, tot_infusion=v_add,
                        start_vol=v_cum, start_temp=0.0 if i == 0 else t_prev,
                        infusion_temp=itemp, decoction_amt=dec,
                        above_boiling=(kind == 0 and itemp is not None and itemp > ABOVE_BOILING_C),
                        no_water=(kind == DECOCTION and v_cum <= 0.0)))   # §2.4: mash'te su yok -> dekoksiyon hatası
        if kind != DECOCTION: v_cum += v_add
        t_prev = t_step
    return out

def recalc_biab(steps, W, T_grain, biab_vol, m_tun=0, c_tun=0, V_tun=0, T_tun=None, Tb=100.0):
    """§3.2 RecalculateBIAB. Adımlar sondan başa; sıcaklıklar §2.5 ile aynı."""
    n = len(steps); infusion = [s["infusion"] for s in steps]; start = [0.0]*n
    v_rem = biab_vol
    for k in range(n-1, -1, -1):
        if k > 0 and steps[k]["type"] != DECOCTION: v_rem -= infusion[k]
        if k == 0: infusion[0] = v_rem; start[0] = 0.0
        else: start[k] = v_rem
    out = []
    for k, s in enumerate(steps):
        kind, t_step = s["type"], s["t_step"]; t_prev = steps[k-1]["t_step"] if k > 0 else None
        itemp = None; dec = 0.0
        if k == 0:
            if kind in (0, DECOCTION):
                itemp = strike_temp(infusion[0], t_step, W, T_grain, m_tun, c_tun, V_tun, T_tun)
        elif kind == 0:
            itemp = infusion_temp(infusion[k], t_step, start[k], t_prev, W, m_tun, c_tun, V_tun)
        elif kind == DECOCTION:
            dec = decoct_vol(t_step, start[k], t_prev, W, m_tun, c_tun, V_tun, Tb); itemp = 0.0
        out.append(dict(infusion=infusion[k], tun_addition=0.0, tot_infusion=infusion[k],
                        start_vol=start[k], start_temp=0.0 if k == 0 else t_prev,
                        infusion_temp=itemp, decoction_amt=dec,
                        above_boiling=(kind == 0 and itemp is not None and itemp > ABOVE_BOILING_C),
                        no_water=(kind == DECOCTION and start[k] <= 0.0)))   # §2.4
    return out

def tun_deadspace_profile(tun_addition, adj_deadspace, tun_deadspace):  # §2.6 F_MH_TUN_DEADSPACE
    return tun_addition + (tun_deadspace if adj_deadspace else 0.0)

def scale_from_grain_weight(infusions, W_old, W_new, biab=False):   # §2.6 ScaleFromGrainWeight
    if not biab and W_old > 0 and W_new > 0 and W_old != W_new:
        return [W_new*i/W_old if i > 0 else i for i in infusions]
    return list(infusions)

def step_water_grain(start_vol, infusion, W, decoction):    # §2.7 (L/kg, W = 0 -> 0: belirtilmemiş, seçim)
    if W == 0: return 0.0
    return (start_vol/W) if decoction else ((start_vol+infusion)/W)

def scale_water_grain(r, W, start_vol, infusion_old):       # §2.7: yalnız |değişim| > 1 fl oz ise yazılır
    new = r*W - start_vol
    return new if abs(new-infusion_old) > FLOZ_L else infusion_old

def main_mash_thickness(steps, tot_water, W_total):
    """§2.8; steps = [{"w","t_step","start_vol","tot_infusion","decoction"}] (L, kg, °C).
    Sonuç L/kg, ama kod sabitiyle: r (fl oz/oz) x 1.043192807606429."""
    best = None; best_diff = 40.0
    for s in steps:
        if not s["w"] > 0: continue
        d = abs(s["t_step"]*9/5 + 32 - MASH_FG_ADJ_TEMP_F)
        if d < best_diff: best, best_diff = s, d
    if best is not None:
        v = best["start_vol"] if best["decoction"] else best["start_vol"] + best["tot_infusion"]
        r = (v/FLOZ_L)/(best["w"]/OZ_KG)
    else:
        r = (tot_water/FLOZ_L)/(W_total/OZ_KG) if W_total != 0 else 0.0
    return r*THICKNESS_CONST

# --- §3.1 BIAB ---
def biab_mash_vol(v_preboil, tun_deadspace, topup_kettle, W_mash):
    return v_preboil + tun_deadspace - topup_kettle + ABS_BIAB*W_mash

# --- §4.1 sparge suyu ---
def sparge_vol(v_preboil, absorb, sparge_absorb_, tun_deadspace, topup_kettle, mash_tot_water_):
    return v_preboil + absorb + sparge_absorb_ + tun_deadspace - topup_kettle - mash_tot_water_

# --- §5 kaynatma, soğuma, hacim zinciri ---
def tot_boil_off(boil_off, boil_time_min, rate_flag):       # §5.1
    return boil_off*(boil_time_min/60) if rate_flag else boil_off

def post_boil_vol(v_preboil, boil_off_total):               # §5.3
    return v_preboil - boil_off_total

def cool_loss(cool_pct, v_postboil):                        # §5.4 (c burada sınırlanmaz)
    return (cool_pct/100)*v_postboil

def pre_trub_vol(batch, trub):                              # §5.5
    return batch + trub

def evap_rate(boil_off_total, v_preboil, boil_time_min):    # §5.6 (%/saat; süre <= 0 -> 0: seçim)
    if boil_time_min <= 0: return 0.0
    vp = v_preboil if v_preboil > 0 else 1.0
    return boil_off_total*100/(vp*boil_time_min/60)

def bottle_vol_equip(batch, fermenter_loss):                # §5.7
    return batch - fermenter_loss

def bottle_vol_recipe(batch, fermenter_loss, include_starter, starter, use_second, second_starter):
    if include_starter:
        return batch + starter + (second_starter if use_second else 0.0) - fermenter_loss
    return batch - fermenter_loss

def total_water(recipe_type, has_boil_ing, mash_tot_water_, sparge, topup_kettle, topup_ferm, batch, v_preboil):
    """§5.8. Tip 1/2: toplam; diğerleri Extract sayılır (tip > 2 belirtilmemiş, seçim). Tarif tipi 0-6 (T1.16)."""
    _kod("tarif tipi", recipe_type, range(7))
    if recipe_type in (1, 2):
        return mash_tot_water_ + sparge + topup_kettle + topup_ferm
    if not has_boil_ing:
        return batch
    return v_preboil + topup_ferm

def total_grain_use(items, use):                            # §5.9 TotalGrainUseField; items = [(use, amount)]
    _kod("kullanım", use, range(9))
    for u, a in items: _kod("kullanım", u, range(9))
    return sum(a for u, a in items if u == use)

def steep_vol(v_preboil, v_boilgrains, W_steep):            # §5.9; W_steep kg
    steep_max = STEEP_QT_PER_LB*(W_steep/OZ_KG/16)*32*FLOZ_L
    return min(v_preboil - v_boilgrains, steep_max)

def boil_water_add(v_preboil, v_steep, v_boilgrains):       # §5.9
    return v_preboil - v_steep - v_boilgrains

def primary_water_add(recipe_type, has_boil_ing, batch, v_postboil, v_primarygrains, topup_ferm):
    _kod("tarif tipi", recipe_type, range(7))
    post = v_postboil if (recipe_type <= 2 or has_boil_ing) else 0.0
    return batch - post - v_primarygrains - topup_ferm

def preboil_sg(og, batch, trub, v_preboil):                 # §5.10 (v_preboil <= 0 -> ValueError, T1.16)
    if v_preboil <= 0: raise ValueError("ön-kaynatma hacmi > 0 olmalı")  # Karar (T1.16): eskiden None
    return 1 + (og-1)*(batch+trub)/v_preboil

def volume_chain(batch, topup_ferm, trub, cool_pct, boil_off, boil_time_min, rate_flag, fermenter_loss):
    """§5.11 zincir (ekipman): boil-off -> pre-boil -> post-boil -> soğuma -> soğuk wort -> şişe."""
    bo = tot_boil_off(boil_off, boil_time_min, rate_flag)
    pre = equip_boil_vol(batch, topup_ferm, trub, cool_pct, bo)
    post = post_boil_vol(pre, bo)
    cool = cool_loss(cool_pct, post)
    return dict(boil_off=bo, preboil=pre, postboil=post, cool_loss=cool, cold_wort=post-cool,
                pretrub=pre_trub_vol(batch, trub), evap_pct=evap_rate(bo, pre, boil_time_min),
                bottle=bottle_vol_equip(batch, fermenter_loss))

# --- §5.12 Water Needed aracı ---
def wn_cool_loss(v_ferm, topup, cool_pct):
    return (v_ferm-topup)*(cool_pct/100)/(1-cool_pct/100)

def wn_boil_vol(v_ferm, topup, trub, v_cool, boil_off):
    return (v_ferm-topup) + trub + v_cool + boil_off

def wn_boil_pct(boil_off, v_boil):                          # v_boil == 0 -> 0 (seçim)
    return boil_off*100/v_boil if v_boil != 0 else 0.0

def wn_grain_absorb(W):
    return ABS_STD*W

def wn_sparge_water(v_boil, absorb, topup_kettle, lauter_deadspace, v_mash):
    return v_boil + absorb - topup_kettle + lauter_deadspace - v_mash

def wn_total_water(brew_type, v_boil, topup, v_sparge, v_mash, topup_kettle):
    _kod("tarif tipi", brew_type, range(7))
    if brew_type == 0: return v_boil + topup
    return v_sparge + v_mash + topup_kettle + topup

def wn_bottle_vol(v_ferm, v_loss_ferm):
    return v_ferm - v_loss_ferm

# --- §6.2 boil-off aracı ---
def boil_off_tool(v0, evap_pct, boil_time_min, cool_pct, sg0):
    rate = evap_pct*v0/100
    bo = boil_time_min*evap_pct*v0/6000
    cool = (cool_pct/100)*(v0-bo)
    v_end = v0 - bo - cool
    sg_end = 1.0 if v_end == 0 else 1 + (sg0-1)*v0/v_end
    return dict(rate=rate, boil_off=bo, cool_loss=cool, v_end=v_end, sg_end=sg_end)

# --- §8 seyreltme ---
def dilution(sg0, v0, v_add, wort, sg_add=1.0):             # wort=False su, True wort/bira (§8.1)
    v_tot = v0 + v_add
    if v_add < 0: return dict(v_tot=v_tot, sg=sg0)
    if not wort: return dict(v_tot=v_tot, sg=1 + (sg0-1)*v0/v_tot)
    return dict(v_tot=v_tot, sg=1 + ((sg0-1)*v0 + (sg_add-1)*v_add)/v_tot)

def gravity_adj_water_vol(v, sg, sg_target):                # §8.2
    return max(v*(sg-1)/(sg_target-1) - v, 0.0)

# --- §9 misc hacmi (fl oz'a; girdi misc birim kodu) ---
def misc_to_fluid_oz(amount, units):
    ml = {0: amount/1000.0, 1: amount, 3: amount*LB_KG*1000.0, 4: amount*1000.0, 5: amount,
          6: amount*4.93, 7: amount*14.78, 11: amount*1000.0}
    if units in ml: return ml[units]/(FLOZ_L*1000.0)
    _kod("misc birimi", units, (2, 8, 9, 10, 12))  # Karar (T1.16): eskiden None
    return {2: amount, 8: amount*8.0, 9: amount*16.0, 10: amount*32.0, 12: amount*128.0}[units]

print("T1.14b grain_vol(5):", r(grain_vol(5)), " grain_absorb(5):", r(grain_absorb(5)), " BIAB:", r(grain_absorb(5, True)))
print("T1.14b biab_mash_vol (TV8):", r(biab_mash_vol(bv, 1, 0, 5)), " sparge_vol (TV7):", r(sparge_vol(bv, absorb, 0, 1, 0, 16)))
print("T1.14b volume_chain (TV6):", {k: r(x) for k, x in volume_chain(23, 0, 1, 4, 4.0, 60, False, 0).items()})
print("T1.14b boil_off_tool (TV13):", {k: r(x) for k, x in boil_off_tool(25, 10, 60, 4, 1.050).items()})

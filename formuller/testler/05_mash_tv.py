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

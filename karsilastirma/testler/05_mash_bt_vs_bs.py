#!/usr/bin/env python3
"""05 - Mash / hacim: Brewtarget formullerinin Python portu ve BeerSmith 4 test vektorleriyle cross-check.
BeerSmith referansi: /home/user/formuller/05-mash-hacimler.md (s.10) + testler/05_mash_tv.py
Brewtarget kaynaklari (dosya:satir) her fonksiyonun yaninda.  Birimler: degC, L, kg.
"""
# ---------------- BeerSmith 4 (spesifikasyon s.10'dan aynen) ----------------
FLOZ_L = 29.57352956/1000.0; LB_KG = 0.4535924; OZ_KG = LB_KG/16
BS_GRAIN_VOL = 0.652; BS_C_MALT = 0.380
BS_ABS_STD = 0.96*FLOZ_L/OZ_KG; BS_ABS_BIAB = 0.586*FLOZ_L/OZ_KG

def bs_tun_eff(m, vm, vt):
    cap = 0.8*vt
    return m*vm/cap if (cap > 0 and vm < cap) else m
def bs_strike(Vw, Ts, W, Tg, m=0, c=0, Vt=0, Tt=None):
    Tt = Tg if Tt is None else Tt
    me = bs_tun_eff(m, BS_GRAIN_VOL*W+Vw, Vt)
    return Ts + ((Ts-Tt)*c*me + BS_C_MALT*W*(Ts-Tg))/Vw
def bs_inf_temp(Va, T2, Vw, T1, W, m=0, c=0, Vt=0):
    me = bs_tun_eff(m, BS_GRAIN_VOL*W+Va+Vw, Vt)
    return T2 + (T2-T1)*(c*me + BS_C_MALT*W + Vw)/Va
def bs_inf_amt(Ti, T2, Vw, T1, W, m=0, c=0, Vt=0):
    me = bs_tun_eff(m, BS_GRAIN_VOL*W+Vw, Vt)
    return (T2-T1)*(c*me + BS_C_MALT*W + Vw)/(Ti-T2)
def bs_decoct(T2, Vw, T1, W, m=0, c=0, Vt=0, Tb=100.0):
    Vm = BS_GRAIN_VOL*W+Vw; me = bs_tun_eff(m, Vm, Vt)
    f = (T2-T1)/(Tb-T1)*(c*me+BS_C_MALT*W+Vw)/(BS_C_MALT*W+Vw)
    return Vm*min(f, 1.0)
def bs_preboil(batch, topup, trub, cool, bo): return bo + (batch-topup+trub)/(1-min(cool,99)/100)

# ---------------- Brewtarget (port) ----------------
BT_C_GRAIN = 0.4            # src/measurement/PhysicalConstants.h:141 grainSpecificHeat_calGC
BT_C_GRAIN_DLG = 0.41       # src/tools/StrikeWaterDialog.cpp:34 specificHeatBarley
BT_GRAIN_DENS = 0.963       # PhysicalConstants.h:105 grainDensity_kgL -> 1/0.963 = 1.0384 L/kg
BT_ABS_CONST = 1.085        # PhysicalConstants.h:113 (ekipman yoksa)
BT_ABS_EQUIP = 1.086        # src/model/Equipment.h:254 default_grainAbsorption_LKg (ekipmanda deger yoksa)
BT_EVAP_DEF = 4.0           # Equipment.h:253 default_kettleEvaporationPerHour_l

def bt_strike_wizard(Vw, Ts, W, Tg, tunHC=0.0, Tt=None):
    """MashWizard.cpp:241-319 enerji dengesi (Kelvin). tunHC = kg*cal/gC, kismi doluluk yok."""
    Tt = Tg if Tt is None else Tt
    K = 273.15; hw = Vw*1.0; hg = BT_C_GRAIN*W
    E = (Ts+K)*(hw+hg+tunHC) - (Tg+K)*hg - (Tt+K)*tunHC
    return E/hw - K
def bt_strike_dialog(Vw, Ts, W, Tg):
    """StrikeWaterDialog.cpp:39-44 Palmer: (0.41/r)(T2-T1)+T2, tun yok."""
    return (BT_C_GRAIN_DLG/(Vw/W))*(Ts-Tg) + Ts
def bt_inf_amt_wizard(T2, Vw, T1, W, tunHC=0.0, Tb=100.0):
    """MashWizard.cpp:355,433-441: sicak su daima kaynama noktasinda; HC = tahil+tun+onceki su."""
    return (BT_C_GRAIN*W + tunHC + Vw)*(T2-T1)/(Tb-T2)
def bt_inf_amt_dialog(Ti, T2, Vw, T1, W):
    """StrikeWaterDialog.cpp:49-59 (0.41)."""
    return (T2-T1)*(BT_C_GRAIN_DLG*W + Vw)/(Ti-T2)
def bt_inf_temp_designer(Va, T2, Vw, T1, W, tunHC=0.0):
    """MashDesigner.cpp:281-324 tempFromVolume_c (ilk adim degil; m_MC = tahil+su+tun, nextStep 138-146)."""
    mMC = BT_C_GRAIN*W + Vw + tunHC
    return mMC*(T2-T1)/Va + T2
def bt_decoct(T2, Vw, T1, W, tunHC=0.0, Tb=100.0):
    """MashWizard.cpp:392-422 (MashDesigner.cpp:668-691 ayni oran)."""
    wg = Vw + BT_C_GRAIN*W
    ratio = (wg+tunHC)*(T2-T1)/(wg*(Tb-T1))
    return ratio*(Vw + W/BT_GRAIN_DENS), ratio
def bt_preboil(batch, trub, topup_fv, evap_lh, boil_min=60):
    """Equipment.cpp:792-798 calculatedPreBoilSize_l (sogume buzulmesi YOK)."""
    return batch + trub - topup_fv + boil_min/60*evap_lh
def bt_end_of_boil(V, evap_lh, boil_min=60):
    """Equipment.cpp:818-822"""
    return V - boil_min/60*evap_lh
def bt_total_mash_vol(preboil, topup_kettle, absorb, W, lauter_loss, post_mash_add=0.0):
    """Recipe.cpp:3344-3401 targetCollectedWortVol_l + targetTotalMashVol_l"""
    return max(preboil - topup_kettle - post_mash_add, 0.0) + absorb*W + lauter_loss
def bt_strike_water(thick, W, foundation, mashtun_loss):
    """Recipe.cpp:3403-3406 + Equipment.cpp:800-802"""
    return thick*W + foundation + mashtun_loss
def bt_batch_sparge(sparge, n): return [sparge/n]*n            # MashWizard.cpp:552-559
def bt_sparge_temp(sparge, W, absorb, tunHC, T_sparge, T_last):
    """MashWizard.cpp:531-543 (son adim -10 C sezgisi, tum sparge hacmine bolunur)."""
    hc = W*BT_C_GRAIN + absorb*W + tunHC
    return hc/sparge*(T_sparge-(T_last-10)) + T_sparge

# ---------------- karsilastirma ----------------
rows = []
def add(tv, ad, bs, bt, note=""):
    d = (bt-bs)/bs*100 if (isinstance(bs,(int,float)) and isinstance(bt,(int,float)) and bs) else None
    rows.append((tv, ad, bs, bt, d, note))
f = lambda x: f"{x:.3f}" if isinstance(x,(int,float)) else str(x)

W, Vw, Tg, Ts = 5.0, 15.0, 20.0, 66.0
add("TV1","Strike, tun yok", bs_strike(Vw,Ts,W,Tg), bt_strike_wizard(Vw,Ts,W,Tg), "BT MashWizard c=0.40")
add("TV1","Strike, tun yok", bs_strike(Vw,Ts,W,Tg), bt_strike_dialog(Vw,Ts,W,Tg), "BT StrikeWaterDialog c=0.41")
add("TV2","Strike, tun 4kg c=0.12 V=40L", bs_strike(Vw,Ts,W,Tg,4,0.12,40,20), bt_strike_wizard(Vw,Ts,W,Tg,4*0.12,20), "BT kismi doluluk yok (tam kutle)")
add("TV2b","Strike, varsayilan tun 4.536kg c=0.3", bs_strike(Vw,Ts,W,Tg,160*OZ_KG,0.3,640*FLOZ_L,20), bt_strike_wizard(Vw,Ts,W,Tg,160*OZ_KG*0.3,20), "")
add("TV2c","Strike, tun BIAB/AIO (kazanda mash)", bs_strike(Vw,Ts,W,Tg), bt_strike_wizard(Vw,Ts,W,Tg,0.0), "BT mashesInKettle -> tunHC=0")
add("TV3","Ara infuzyon 66->72, 100C su (L)", bs_inf_amt(100,72,Vw,66,W), bt_inf_amt_wizard(72,Vw,66,W), "BT MashWizard")
add("TV3","Ara infuzyon 66->72, 100C su (L)", bs_inf_amt(100,72,Vw,66,W), bt_inf_amt_dialog(100,72,Vw,66,W), "BT StrikeWaterDialog")
add("TV3b","4 L ile 66->72 su sicakligi", bs_inf_temp(4,72,Vw,66,W), bt_inf_temp_designer(4,72,Vw,66,W), "BT MashDesigner")
d1, r1 = bt_decoct(72,Vw,66,W); d2, r2 = bt_decoct(66,Vw,50,W)
add("TV4","Dekoksiyon 66->72 (L mash)", bs_decoct(72,Vw,66,W), d1, f"oran ayni ({r1:.4f}); fark tahil hacmi 0.652 vs 1.038 L/kg")
add("TV4","Dekoksiyon 50->66 (L mash)", bs_decoct(66,Vw,50,W), d2, f"oran {r2:.4f}")
add("TV5","Kaynama sicakligi 1000 m", 100-1000*0.00324, "yok (elle, vars. 100)", "BT rakim modeli yok")
pb_bs = bs_preboil(23,0,1,4,4.0); pb_bt = bt_preboil(23,1,0,4.0)
add("TV6","Pre-boil (23L,trub1,%4,4L/h)", pb_bs, pb_bt, "BT sogume buzulmesi yok")
add("TV6","Post-boil", pb_bs-4, bt_end_of_boil(pb_bt,4.0), "")
add("TV6","Fermentore giren (BT zinciri, ayni pre-boil=29)", 23.0, bt_end_of_boil(29,4.0)-1, "BT 29 L ile 24 L verir (fazla)")
# TV7
A_bs = BS_ABS_STD*W; sp_bs = pb_bs + A_bs + 1 - 16; tot_bs = 16 + sp_bs
tot_bt_same = bt_total_mash_vol(29,0,BS_ABS_STD,W,1)
strike_bt = bt_strike_water(3.0,W,0,1)
add("TV7","Toplam su - ayni girdiler (29L, 1.0014 L/kg)", tot_bs, tot_bt_same, "yapi ozdes")
add("TV7","Sparge - ayni girdiler", sp_bs, tot_bt_same-strike_bt, "")
tot_bt_def = bt_total_mash_vol(pb_bt,0,BT_ABS_EQUIP,W,1)
add("TV7","Toplam su - her biri kendi varsayilaniyla", tot_bs, tot_bt_def, "BT: pre-boil 28, emilim 1.086")
add("TV7","Sparge - kendi varsayilanlari", sp_bs, tot_bt_def-strike_bt, "")
add("TV7","Tun hacim ihtiyaci (16 L su + 5 kg)", 16+BS_GRAIN_VOL*W, 16+W/BT_GRAIN_DENS, "MashDesigner.cpp:222-224")
# TV8 BIAB
biab_bs = pb_bs + 1 + BS_ABS_BIAB*W
biab_bt = bt_total_mash_vol(29,0,BT_ABS_EQUIP,W,0)   # kazanda mash: mashTunLoss yok sayilir
biab_bt_gf = bt_total_mash_vol(29,0,0.8,W,0)
add("TV8","BIAB toplam su (pre-boil 29)", biab_bs, biab_bt, "BT ayri BIAB emilimi yok (1.086)")
add("TV8","BIAB toplam su, BT Grainfather 0.8 L/kg", biab_bs, biab_bt_gf, "data/DefaultContent005 ekipmanlari")
add("TV8","BIAB strike (kendi hacimleri)", bs_strike(biab_bs,66,W,20), bt_strike_wizard(biab_bt,66,W,20), "")
add("TV8","BIAB strike (ayni hacim 33.057)", bs_strike(biab_bs,66,W,20), bt_strike_wizard(biab_bs,66,W,20), "yalniz c_malt farki")
# TV9
add("TV9a","Batch sparge esit, 1. parti", "4.007 ekle + 15.000", f"{bt_batch_sparge(sp_bs,2)[0]:.3f} x2", "BT tun kapasitesi/ilk suzme dengelemesi yok")
add("TV9","Sparge suyu sicakligi (75.6C hedef)", "kullanici girdisi", bt_sparge_temp(sp_bs,W,BT_ABS_EQUIP,0,75.6,66), "BT -10C sezgisi")
add("TV10","Parti-gyle", "var", "yok", "")
add("TV11","Seyreltme araci", 1.048, "yok", "")
add("TV13","Boil araci son hacim 25L,2.5L/h", 21.6, bt_end_of_boil(25,2.5), "BT sogume yok")
add("TV13","Boil araci SG (nokta korunumu)", 1.0579, 1+0.050*25/bt_end_of_boil(25,2.5), "BrewLog.cpp:489-499 benzeri")
add("--","Emilim sabiti L/kg", BS_ABS_STD, BT_ABS_EQUIP, "BT PhysicalConstants 1.085")
add("--","BIAB emilimi L/kg", BS_ABS_BIAB, BT_ABS_EQUIP, "BT ayrim yok")
add("--","Tahil hacmi L/kg", BS_GRAIN_VOL, 1/BT_GRAIN_DENS, "")
add("--","Malt ozgul isisi", BS_C_MALT, BT_C_GRAIN, "BT dialog 0.41")

print(f"{'TV':5} | {'Hesap':48} | {'BeerSmith':>14} | {'Brewtarget':>22} | {'Fark %':>7} | Not")
for tv, ad, bs, bt, d, note in rows:
    print(f"{tv:5} | {ad:48} | {f(bs):>14} | {f(bt):>22} | {('%+.2f'%d) if d is not None else '-':>7} | {note}")

# kendi kendini dogrulama: BS portu spesifikasyon test vektorlerini veriyor mu?
assert abs(bs_strike(15,66,5,20)-71.827) < 1e-3
assert abs(bs_strike(15,66,5,20,4,0.12,40,20)-72.667) < 1e-3
assert abs(bs_inf_amt(100,72,15,66,5)-3.621) < 1e-3
assert abs(bs_decoct(72,15,66,5)-3.222) < 1e-3
assert abs(pb_bs-29.0) < 1e-9 and abs(sp_bs-19.007) < 1e-3 and abs(biab_bs-33.057) < 1e-3
print("\nBeerSmith portu spesifikasyon test vektorleriyle uyumlu (assert OK).")

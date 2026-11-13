import math
GAL_L = 128*29.57352956/1000.0
# a_matrix: columns = salts, rows = Ca,Mg,Na,SO4,Cl,HCO3  (ppm per g/US gal)
A = {
 'gypsum':[61.478,0,0,147.359,0,0],
 'nacl':[0,0,103.891,0,160.212,0],
 'epsom':[0,26.043,0,102.933,0,0],
 'cacl2':[95.371,0,0,0,168.731,0],
 'soda':[0,0,72.275,0,0,191.827],
 'chalk':[105.754,0,0,0,0,322.016],
 'mgcl2':[0,31.573,0,0,92.111,0],
 'lime':[142.856,0,0,0,0,434.991],
}
ions=['Ca','Mg','Na','SO4','Cl','HCO3']
def salt_ppm(grams, vol_L):
    gal = vol_L/GAL_L
    out=[0]*6
    for s,g in grams.items():
        for i in range(6): out[i]+= g/gal*A[s][i]
    return out
print("GAL_L",GAL_L)
p = salt_ppm({'gypsum':5,'cacl2':3},20)
print("T1", [round(x,2) for x in p])
def alk(hco3): return hco3*50/61
def hard(ca,mg): return ca/1.4+mg/1.7
def RA(ca,mg,hco3): return alk(hco3)-hard(ca,mg)
base=dict(Ca=50,Mg=10,Na=15,SO4=40,Cl=30,HCO3=150)
print("T2 alk",alk(150),"hard",hard(50,10),"RA",RA(50,10,150))
def gfun(c, bizim=False):
    """Grist asitlik fonksiyonu. BeerSmith: c<14 -> c-1.9, 14<=c<200 -> 0.453c+5 (14 SRM'de süreksiz).
    Bizim model (F1 karar 12a): iki doğrunun küçüğü, c<200 için sürekli (kesişim 6.9/0.547 = 12.614 SRM)."""
    if c>=200: return 42.0
    if bizim: return min(c-1.9, 0.453*c+5.0)
    return c-1.9 if c<14 else 0.453*c+5.0
def gristph(bill, bizim=False):
    W=sum(w for w,c in bill)
    s=sum(w/W*gfun(c,bizim) for w,c in bill)
    return 5.72-0.0337*s, s
GRIST_TYPES = (0, 3)                    # §5.1 F_G_TYPE: Grain, Adjunct
def gristph_typed(bill, bizim=False):
    """§5.1 tam hali: bill = [(kg, SRM, F_G_TYPE)] (mash tahılları). Payda bütün mash tahıllarıdır;
    yalnız tip 0 ve 3 toplama girer (şeker vb. S'ye 0 katar). W <= 0 ise (7.0, 0.0)."""
    W = sum(w for w, c, ty in bill)
    if W <= 0: return 7.0, 0.0
    s = sum(w/W*gfun(c, bizim) for w, c, ty in bill if ty in GRIST_TYPES)
    return 5.72-0.0337*s, s

def slope(model,t):
    return t*0.47930613*0.17/1.5 if model==1 else 0.037+0.014*t
def pks(a):
    if a not in range(9): raise ValueError("asit kodu 0-8 olmalı")  # Karar (T1.16): eskiden laktik
    return {2:(2.12,7.2,12.44),4:(3.14,4.77,6.39),5:(4.75,20,20),6:(-7,20,20),7:(-1,1.92,20),8:(2.98,4.34,20)}.get(a,(3.86,20,20))
def acidfrac(a,ph):
    k1,k2,k3=pks(a); r1=10**(ph-k1); r2=10**(ph-k2); r3=10**(ph-k3)
    f0=1/(1+r1+r1*r2+r1*r2*r3)
    return r1*f0+2*r1*r2*f0+3*r1*r2*r3*f0
def fdiss(acid,model,ph_t,bizim):
    """Ayrışma kesri. BeerSmith: yalnız model 1'de ve sabit pH 5.4'te. Bizim model (F1 karar 12b):
    iki modelde de hedef pH'ta."""
    if bizim: return acidfrac(acid,ph_t)
    return acidfrac(acid,5.4) if model==1 else 1.0
def acid_per_L(meq,acid,conc,model,ph_t=5.4,bizim=False):
    if acid not in (0,1,2,3): raise ValueError("mash asit miktarı yalnız 0-3 kodları için tanımlı")  # Karar (T1.16): 4-8 eskiden fosforik sabitleri
    if acid==0: return 0
    if conc<=0: raise ValueError("derişim > 0 olmalı")  # Karar (T1.16)
    if acid in (1,3): mw=90.09; rho=1+conc*0.237/100
    else: mw=98.0; rho=1+0.49*conc/100+0.375*(conc/100)**2
    f=fdiss(acid,model,ph_t,bizim)
    if acid==3: return meq*100*mw/(conc*f*1000)
    return meq*100*mw/(conc*f*rho*1000)
bill=[(4.5,2.0),(0.3,60.0),(0.1,350.0)]
W=sum(w for w,c in bill); t=3.0
gp,s=gristph(bill); print("T3 grist sum",s,"gristpH",gp)
# mash water: base + 5 g gypsum + 3 g CaCl2 in 20 L mash water
mash=[base[k]+p[i] for i,k in enumerate(ions)]
ra=RA(mash[0],mash[1],mash[5]); print("T3 mash profile",[round(x,2) for x in mash],"RA",ra)
print("acidfrac lactic 5.4",acidfrac(1,5.4),"phos",acidfrac(2,5.4))
for bizim in (False,True):
  print("-- T3", "BİZİM MODEL (hedef pH'ta ayrışma)" if bizim else "BeerSmith uyumu (ayrışma pH 5.4 sabit, yalnız BW)")
  for model in (0,1):
    sl=slope(model,t); est=gp+ra/50*sl
    dph=est-5.4
    meq=dph/sl
    perL=acid_per_L(meq,1,88,model,5.4,bizim)
    mL=perL*W*t
    perLm=acid_per_L(meq,3,3.0,model,5.4,bizim)  # acid malt 3%
    print(f"T3 model{model} slope={sl:.5f} estpH={est:.4f} meq/L={meq:.4f} lactic88 mL/L={perL:.5f} total mL={mL:.3f} acidmalt(3%) g={perLm*W*t:.1f}")
    # verify via MiscAcidRA
    p_=0.88; rho=1+0.237*p_; f=fdiss(1,model,5.4,bizim)
    VL=mL/1000
    acidra=-(p_*1000*rho*VL*f)/(0.09009*W*t)
    print("   adjusted pH check", gp+sl*(ra/50+acidra))
# sparge
def sparge_mpH(hco3,phw,pht):
    c=hco3/61016.0
    K1=4.4435e-07; K1K2=2.07378145e-17; K2=4.667e-11
    xw=10**phw; xt=10**pht
    co3=xw*c*K2
    Ct=(K1K2*xw*xw+K1*xw+1)*c/(K1*xw)
    return (10**-pht + (10**(phw-14)+2*co3+c) - 10**-phw) - 10**(pht-14) - (2*K1K2*xt*xt+K1*xt)*Ct/(K1K2*xt*xt+K1*xt+1)
def sparge_bnw(hco3,phw,pht,acid):
    def a(ph):
        r1=10**(ph-6.38); r2=10**(ph-10.373); D=1+r1+r1*r2
        return 1/D, r1*r2/D
    a0w,a2w=a(phw); a0e,a2e=a(4.3); a0t,a2t=a(pht)
    Ct=(hco3*50/61/50)/((a0e-a0w)+(a2w-a2e))
    return Ct*((a2w-a2t)+(a0t-a0w))/acidfrac(acid,pht)/1000
def sparge_mpH_bizim(hco3,phw,pht,acid=1):
    """Bizim model (F1 karar 12b): model 0 sparge asidi de hedef pH'taki ayrışma kesrine bölünür."""
    return sparge_mpH(hco3,phw,pht)/acidfrac(acid,pht)
def lactic_mL(molL,V,conc):
    c=max(conc,1.0)/100
    return (1/c)*0.09009/(1+0.237*c)*molL*V*1000
hco3=150; phw=7.5; pht=5.4
Vtot=30.0; Vsp=Vtot - W*t
m1=sparge_mpH(hco3,phw,pht); m2=sparge_bnw(hco3,phw,pht,1)
print("T4 BeerSmith uyumu: Vsp",Vsp,"MPH mol/L",m1,"mL",lactic_mL(m1,Vsp,88),"BNW mol/L",m2,"mL",lactic_mL(m2,Vsp,88))
# bizim model: ayrı sparge hedefi (F1 karar 12c), varsayılan 6.0 [doğrulanacak]
for pts in (5.4,6.0):
    mm=sparge_mpH_bizim(hco3,phw,pts); mb=sparge_bnw(hco3,phw,pts,1)
    print(f"T4b bizim model sparge hedefi {pts}: MPH mol/L {mm:.8f} mL {lactic_mL(mm,Vsp,88):.3f} ; BW mol/L {mb:.8f} mL {lactic_mL(mb,Vsp,88):.3f}")
print("T5 SO4/Cl", mash[3]/mash[4])
print("T6 color range", 5.2+0.082*ra, (ra+122.4)/12.2)
# boil scale
Vb,Vp,cool,trub,top=28.0,24.0,1.0,1.0,0.0
a_=Vp-cool; b_=a_-trub
print("T7 boilscale",((Vb/a_)*b_+top)/(b_+top))
def adj_ph(bill_, acidmalt_g, pct, model, ra, t, bizim=False, target=5.4):
    """§5.5 (asit malt). bill_ = [(kg, SRM, F_G_TYPE)] (T1.17: tip süzgeci, §5.1). Eklenen asit malt tip 0, 3 SRM.
    target: hedef pH (§6.4); ayrışma kesri yalnız bizim modelde ona bağlıdır."""
    b=bill_+([(acidmalt_g/1000,3.0,0)] if acidmalt_g>0 else [])
    W=sum(w for w,c,ty in b)
    if W<=0 or t<=0: raise ValueError("tahıl ağırlığı ve kalınlık > 0 olmalı")  # Karar (T1.16)
    gp,_=gristph_typed(b,bizim); sl=slope(model,t)
    f=fdiss(3,model,target,bizim)
    acidra=-((pct/100)*1000*(acidmalt_g/1000)*f)/(W*t*0.09009)
    return gp+sl*(ra/50+acidra), W
for bizim in (False,True):
  print("-- asit malt yinelemesi", "BİZİM MODEL" if bizim else "BeerSmith uyumu")
  for model in (0,1):
    tot=0.0
    tbill=[(w,c,0) for w,c in bill]
    for it in range(8):
        ph,W=adj_ph(tbill,tot,3.0,model,ra,t,bizim)
        sl=slope(model,t)
        amt=acid_per_L((ph-5.4)/sl,3,3.0,model,5.4,bizim)*W*t if ph-5.4>=0 else 0
        tot+=amt
        ph2,_=adj_ph(tbill,tot,3.0,model,ra,t,bizim)
        print(f"  model{model} it{it} add={amt:.2f} g total={tot:.2f} g adjpH={ph2:.4f}")
        if ph2-5.4<0.005: break
try:
    import numpy as np
    from scipy.optimize import nnls
    names=list(A.keys())
    M=np.array([A[n] for n in names]).T
    tgt=np.array([100,15,20,250,60,150.0]); b=np.array([base[k] for k in ions])
    x,r=nnls(M,tgt-b)
    print("T8 g/gal",dict(zip(names,np.round(x,4))),"rnorm",r)
    print("T8 grams for 20 L",dict(zip(names,np.round(x*20/GAL_L,2))))
    print("T8 result profile",np.round(b+M@x,1))
    # exclude chalk, lime, mgcl2
    keep=['gypsum','nacl','epsom','cacl2','soda']
    M2=np.array([A[n] for n in keep]).T; x2,r2=nnls(M2,tgt-b)
    print("T8b g/gal",dict(zip(keep,np.round(x2,4))),"rnorm",r2,"grams 20L",np.round(x2*20/GAL_L,2),"profile",np.round(b+M2@x2,1))
except Exception as e: print("no scipy",e)

# ---- F1 kararları: varsayılan model, grist sürekliliği, tebeşir uyarısı
DEFAULT_MODEL = 0          # karar 3: MPH varsayılan; BW (1) seçenek ve "doğrulanacak"
def uyarilar(model, salt_grams):
    u_=[]
    if model==1: u_.append("BW modeli doğrulanacak: MPH'ye göre yaklaşık yarı asit önerir (ölçülmüş mash pH ile kalibre edilene kadar)")
    if salt_grams.get('chalk',0)>0: u_.append("çözünürlük düşük, asitli mash olmadan etkisi abartılı olabilir (tebeşir)")
    return u_
print("T9 grist g(c) BeerSmith 13.99/14:", round(gfun(13.99),3), round(gfun(14),3), " bizim:", round(gfun(13.99,True),3), round(gfun(14,True),3), " 12.614:", round(gfun(6.9/0.547,True),3))
print("T9b grist pH, tek malt 13 SRM: bizim", round(gristph([(1.0,13.0)],True)[0],4), " BeerSmith", round(gristph([(1.0,13.0)])[0],4))
print("T10 varsayılan model:", DEFAULT_MODEL, uyarilar(DEFAULT_MODEL,{'gypsum':5}))
print("T10b BW seçili + 2 g tebeşir:", uyarilar(1,{'chalk':2.0}))

# ---- T1.14c: spesifikasyonda olup yukarıda fonksiyon olarak bulunmayan hesaplar (referans, Rust paritesi için)
# Yukarıdaki fonksiyonlar değişmez; korumalar (guard) ayrı sarmalayıcılardır ve spesifikasyon metnine dayanır.
def salt_ppm_guarded(grams, vol_L):
    """§2: V_gal = 0 ise 0 (hacim <= 0 için de 0)."""
    if vol_L <= 0: return [0.0]*6
    return salt_ppm(grams, vol_L)
def gristph_guarded(bill, bizim=False):
    """§5.1: W <= 0 (boş fatura dahil) ise pH_grist = 7.0 (S = 0)."""
    if sum(w for w,c in bill) <= 0: return 7.0, 0.0
    return gristph(bill, bizim)
def so4cl_ratio(so4, cl):
    """§4.4: SO4/Cl; Cl = 0 koruması 0 döndürür (spesifikasyon önerisi [belirsiz])."""
    return 0.0 if cl == 0 else so4/cl
def so4cl_label(r):
    """§4.4 sınıf etiketi (SulfateBalFunc eşikleri)."""
    if r < 0 or r > 18: return "Out of Range"
    if r < 0.4: return "Extremely Malty"
    if r < 0.6: return "Very Malty"
    if r < 0.8: return "Malty"
    if r < 1.5: return "Balanced"
    if r < 2.01: return "Slightly Bitter"
    if r < 6.01: return "Bitter"
    if r < 8.01: return "Very Bitter"
    return "Extremely Bitter"
def color_range(ra):
    """§4.5: (SRM_min, SRM_max), 0'a tabanlı."""
    return max(0.0, 5.2+0.082*ra), max(0.0, (ra+122.4)/12.2)
def boil_scale(vb, vp, cool, trub, top, has_boil_ing=True):
    """§3.4: S_boil; Vb > 0, Vp > 0 ve F_R_HAS_BOIL_ING > 0 değilse 1."""
    if not (vb > 0 and vp > 0 and has_boil_ing): return 1.0
    a_ = vp-cool; b_ = a_-trub
    return ((vb/a_)*b_+top)/(b_+top)
def slope_checked(model, t, w_kg):
    """§5.2 kenar durum: t <= 0 veya W <= 0 ise hata (ValueError, T1.16); aksi halde eğim."""
    if t <= 0 or w_kg <= 0: raise ValueError("mash kalınlığı ve tahıl ağırlığı > 0 olmalı")
    return slope(model, t)
def misc_acid_ra(acid, conc, vol_L, f, w_kg, t):
    """§5.4(a): sıvı asidin RA eşdeğeri (mEq/L, negatif). acid 1 = laktik, 2 = fosforik (ρ'da 0.365).
    Süzgeçteki dışı asit kodları (FilterMashAgents yalnız WA 10/11) 0 katkı verir."""
    p_ = conc/100.0
    if acid == 1: mw = 0.09009; rho = 1+0.237*p_
    elif acid == 2: mw = 0.098; rho = 1+0.49*p_+0.365*p_*p_
    else: return 0.0
    return -(p_*1000*rho*vol_L*f)/(mw*w_kg*t)
def acid_malt_ra(pct, malt_kg, f, w_kg, t):
    """§5.4(b): asit maltın RA eşdeğeri (mEq/L, negatif); w_kg mash tahılının toplamı."""
    return -((pct/100.0)*1000*malt_kg*f)/(w_kg*t*0.09009)
def sparge_bnw_guarded(hco3, phw, pht, acid):
    """§6.5 kenar durum (model 1): pH_w <= 4.3 ise hesap yapılmaz, 0."""
    if phw <= 4.3: return 0.0
    return sparge_bnw(hco3, phw, pht, acid)
def sparge_acid_ml(mol_L, v_L, conc, acid):
    """§6.5 hacme çevirme: laktik (1) ve fosforik (2, ρ'da 0.365); asit malt ve diğerleri 0; sonuç <= 0 ise 0."""
    c = max(conc, 1.0)/100
    if acid == 1: ml = 0.09009/c/(1+0.237*c)*mol_L*v_L*1000
    elif acid == 2: ml = 0.098/c/(1+0.49*c+0.365*c*c)*mol_L*v_L*1000
    else: return 0.0
    return ml if ml > 0 else 0.0
def sparge_acid_amount(model, bizim, hco3, phw, pht, acid, conc, mash_dph, v_sp):
    """§6.5 bütün: mL. acid 0/3/diğer ya da mash ΔpH < 0 ise 0; model 1 BW (korumalı); model 0 MPH
    (bizim modelde AcidFraction'a bölünür, §6.8)."""
    if acid not in (1, 2) or mash_dph < 0: return 0.0
    if model == 1: mol = sparge_bnw_guarded(hco3, phw, pht, acid)
    elif bizim: mol = sparge_mpH_bizim(hco3, phw, pht, acid)
    else: mol = sparge_mpH(hco3, phw, pht)
    return sparge_acid_ml(mol, v_sp, conc, acid)
# §3 profiller
def base_profile(waters):
    """§3.1: waters = [(miktar, [6 iyon])]; miktar-ağırlıklı ortalama, toplam 0 ise sıfır profil."""
    tot = sum(a for a,_ in waters)
    if tot <= 0: return [0.0]*6
    return [sum(ion[i]*a for a,ion in waters)/tot for i in range(6)]
def water_ph_mix(waters):
    """§3.1: waters = [(miktar, pH)]; ağırlıklı ortalama, su yoksa (toplam 0) 7.0."""
    tot = sum(a for a,_ in waters)
    if tot <= 0: return 7.0
    return sum(ph*a for a,ph in waters)/tot
def agents_ppm(grams, vol_L, ignore):
    """§2: F_R_IGNORE_WATER_AGENTS = 1 ise tüm tuz katkıları 0."""
    return [0.0]*6 if ignore else salt_ppm_guarded(grams, vol_L)
def agent_volume(rtype, use, mash_tot_water, sparge_vol, boil_vol, volume, final_vol):
    """§2 referans hacim F_M_TEMP_VOL. rtype: 0 extract, 1 partial mash, 2 all grain. use: 0 boil, 1 mash, 2 primary, 5 sparge."""
    if rtype in (1, 2):
        return {1: mash_tot_water, 5: sparge_vol, 0: boil_vol, 2: volume}.get(use, final_vol)
    if use in (0, 1): return boil_vol
    if use == 2: return volume
    return final_vol
def total_water_ion(mash_i, sparge_i, vm, vs, has_mash):
    """§3.3 mash + sparge karışımı (iyon listeleri)."""
    if has_mash and vs > 0:
        return [(vm*m+vs*s)/(vm+vs) for m,s in zip(mash_i, sparge_i)]
    return list(mash_i)
def top_up_factor(v_avail, vs, v_topk):
    """§3.3 k_top."""
    return (v_avail+vs)/(v_avail+vs+v_topk) if v_topk != 0 else 1.0
def boil_ion(tot_i, k_top, adj0_i):
    """§3.3 Boil_i = Tot_i·k_top + Adj(use=0, base yok)."""
    return [t*k_top+a for t,a in zip(tot_i, adj0_i)]
def beer_ion(boil_i, s_boil, adj2_i):
    """§3.3 Beer_i = Boil_i·S_boil + Adj(use=2, base yok)."""
    return [b*s_boil+a for b,a in zip(boil_i, adj2_i)]
# §7 eşleştirme çevresi
def match_scale(mash_recipe, has_boil_ing, s_boil_val):
    """§7.1: mash'li reçetede 1; extract reçetede ve kaynatma malzemesi varsa S_boil; aksi halde 1."""
    return 1.0 if mash_recipe or not has_boil_ing else s_boil_val
def salt_grams(x, v_mash_gal, v_sp_gal, combine):
    """§7.2: [gram_mash, gram_sparge]; birleştirme açık ve sparge > 0 ise hacimler toplanır, sparge 0."""
    vm = v_mash_gal
    if combine and v_sp_gal > 0: vm += v_sp_gal
    sp = v_sp_gal if (v_sp_gal > 0 and not combine) else 0.0
    return [[xk*vm for xk in x], [xk*sp for xk in x]]
def profile_mix(vb, b, vd, d):
    """§7.4: Mix_i = (Vb·B_i + Vd·D_i)/(Vb+Vd); toplam hacim 0 ise sıfır profil (spesifikasyonda tanımsız, seçim)."""
    if vb+vd <= 0: return [0.0]*6
    return [(vb*bi+vd*di)/(vb+vd) for bi,di in zip(b,d)]
def wp_grams(x, vb_gal):
    """§7.4 tuhaflığı: gramlar yalnız base hacmiyle (x·Vb)."""
    return [xk*vb_gal for xk in x]
print("T1.14c so4/cl etiket (1.426, 0.3, 19):", so4cl_label(1.426), so4cl_label(0.3), so4cl_label(19))


# ---- T1.15: parite betiğinde duran hesaplar referansa taşındı (ADR-0003 madde 1; T1.12 B4, B7, B17, B18)
# Aşağıdaki fonksiyonlar yukarıdakileri değiştirmez; parite betiği yalnız çağırır.
LB_G = 453.5924                         # §0.1 lb -> g (05 ile aynı LB_KG)
MASH_ACIDS_DEFINED = (0, 1, 2, 3)       # §6.2: yalnız bu kodların MW ve yoğunluğu tanımlı (T1.15 B13 kararı)

def to_grams(amount, units):
    """§0.1 ToGrams: tanımsız birim kodu ValueError (T1.16). mL (5) değeri olduğu gibi gram sayar [belirsiz]."""
    if units not in range(8): raise ValueError("birim kodu 0-7 olmalı")
    return {0: amount/1000.0, 1: amount, 2: amount*LB_G/16.0, 3: amount*LB_G, 4: amount*1000.0,
            5: amount, 6: amount*0.8*4.93, 7: amount*0.8*14.78}[units]

def est_mash_ph(grist_ph, ra, sl):
    """§5.3 (T3 ifadesi: gp + ra/50*sl)."""
    return grist_ph + ra/50*sl

def mash_acid_amount(est, target, model, t, w, acid, conc, bizim):
    """§6.3 (T3: dph/sl -> acid_per_L * W * t); acid 0 veya dph < 0 ise 0."""
    dph = est - target
    if t <= 0 or w <= 0: raise ValueError("mash kalınlığı ve tahıl ağırlığı > 0 olmalı")  # Karar (T1.16)
    if acid not in (0,1,2,3): raise ValueError("mash asit miktarı yalnız 0-3 kodları için tanımlı")  # Karar (T1.16)
    if acid == 0 or dph < 0: return 0.0
    sl = slope(model, t)
    return acid_per_L(dph/sl, acid, conc, model, target, bizim)*w*t

def adj_ph_multi(grist_ph, ra, acid_ras, sl):
    """§5.5 çok asitli ayarlanmış pH (T1.17): gp + m*(RA/50 + Σ acid_ra) = est_mash_ph(gp, RA + 50*Σ acid_ra, m).
    acid_ras: sıvı asitlerin (misc_acid_ra) ve asit maltların (acid_malt_ra) RA eşdeğerleri, mEq/L, verilen sırayla toplanır."""
    tot = 0.0
    for a in acid_ras: tot += a
    return est_mash_ph(grist_ph, ra + 50.0*tot, sl)

def lactic_acid_ra(conc, vol_l, f, w_kg, t):
    """§5.4(a) laktik (T3 kontrolü: acidra = -(p*1000*rho*VL*f)/(0.09009*W*t))."""
    p_ = conc/100
    rho = 1+0.237*p_
    return -(p_*1000*rho*vol_l*f)/(0.09009*w_kg*t)

def adj_ph_liquid(grist_ph, ra, t, model, w_kg, acid, conc, vol_l, target, bizim=False):
    """§5.5 sıvı asitli ayarlanmış pH: gp + m*(RA/50 + MiscAcidRA). f: §5.4(a) (model 1: 5.4'te, bizim: hedefte)."""
    f = fdiss(acid, model, target, bizim) if acid in (1, 2) else 1.0
    return grist_ph + slope(model, t)*(ra/50 + misc_acid_ra(acid, conc, vol_l, f, w_kg, t))

def acid_malt_amount(bill, pct, model, ra, t, target, bizim):
    """§6.4: en fazla 8 tur. bill = [(kg, SRM, F_G_TYPE)] (T1.17). Döner (toplam g, son pH, yakınsadı). yakınsadı = pH_adj - hedef < 0.005 ile durdu."""
    if pct <= 0: raise ValueError("asit malt yüzdesi > 0 olmalı")  # Karar (T1.16)
    tot = 0.0
    ph2 = 0.0
    ok = False
    for _ in range(8):
        ph, w = adj_ph(bill, tot, pct, model, ra, t, bizim, target)
        sl = slope(model, t)
        amt = acid_per_L((ph-target)/sl, 3, pct, model, target, bizim)*w*t if ph-target >= 0 else 0
        tot += amt
        ph2, _ = adj_ph(bill, tot, pct, model, ra, t, bizim, target)
        if ph2-target < 0.005:
            ok = True
            break
    return tot, ph2, ok

def sparge_volume_l(total_l, w_kg, t):
    """§6.5: sparge suyu = toplam su - mash suyu (W*t)."""
    return total_l - w_kg*t

# §7 NNLS: saf Python Lawson-Hanson (ADR-0003 madde 8); Rust `water::lstsq`/`nnls` ile aynı işlem sırası.
_EPS = 2.220446049250313e-16
def _lstsq(cols, b):
    m = len(b); k = len(cols)
    a = [list(c) for c in cols]; qtb = list(b)
    for j in range(k):
        norm = math.sqrt(sum(a[j][i]*a[j][i] for i in range(j, m)))
        if norm == 0.0: continue
        alpha = -norm if a[j][j] > 0.0 else norm
        v = list(a[j][j:]); v[0] -= alpha
        vn2 = sum(x*x for x in v)
        if vn2 == 0.0: continue
        for c in range(j, k):
            dot = sum(v[i]*a[c][j+i] for i in range(len(v)))
            f = 2.0*dot/vn2
            for i in range(len(v)): a[c][j+i] -= f*v[i]
        dot = sum(v[i]*qtb[j+i] for i in range(len(v)))
        f = 2.0*dot/vn2
        for i in range(len(v)): qtb[j+i] -= f*v[i]
    x = [0.0]*k
    for i in range(k-1, -1, -1):
        s = qtb[i]
        for c in range(i+1, k): s -= a[c][i]*x[c]
        d = a[i][i]
        x[i] = s/d if abs(d) > 1e-300 else 0.0
    return x

def nnls_lh(cols, b):
    """Lawson-Hanson NNLS: min ||A x - b||, x >= 0; cols = sütunlar. Döner (x, rnorm)."""
    n = len(cols); m = len(b)
    x = [0.0]*n; passive = [False]*n
    anorm = max((sum(abs(v) for v in c) for c in cols), default=0.0)
    tol = 10.0*max(m, n)*_EPS*anorm
    max_iter = 3*n; it = 0
    while True:
        r = [b[i]-sum(cols[j][i]*x[j] for j in range(n)) for i in range(m)]
        w = [sum(cols[j][i]*r[i] for i in range(m)) for j in range(n)]
        best = -1
        for j in range(n):
            if not passive[j] and w[j] > tol and (best < 0 or w[j] > w[best]): best = j
        if best < 0: break
        passive[best] = True
        while True:
            idx = [j for j in range(n) if passive[j]]
            s_p = _lstsq([cols[j] for j in idx], b)
            if all(v > 0.0 for v in s_p):
                for j in range(n): x[j] = 0.0
                for j, v in zip(idx, s_p): x[j] = v
                break
            alpha = min(x[j]/(x[j]-s) for j, s in zip(idx, s_p) if s <= 0.0)
            for j, s in zip(idx, s_p): x[j] += alpha*(s-x[j])
            for j in idx:
                if x[j] <= tol:
                    x[j] = 0.0; passive[j] = False
            it += 1
            if it > max_iter: break
        it += 1
        if it > max_iter: break
    r = [b[i]-sum(cols[j][i]*x[j] for j in range(n)) for i in range(m)]
    return x, math.sqrt(sum(v*v for v in r))

def nnls_kkt_ok(cols, b, x, tol=1e-7):
    """KKT denetimi (T1.12 B18): x >= 0; gradyan w = A^T (b - A x): x_j > 0 ise ~0, x_j = 0 ise <= tol."""
    n = len(cols); m = len(b)
    r = [b[i]-sum(cols[j][i]*x[j] for j in range(n)) for i in range(m)]
    scale = max(1.0, max(abs(v) for v in b))*max(1.0, max(max(abs(v) for v in c) for c in cols))
    for j in range(n):
        if x[j] < 0.0: return False
        w = sum(cols[j][i]*r[i] for i in range(m))
        if x[j] > 0.0 and abs(w) > tol*scale: return False
        if x[j] == 0.0 and w > tol*scale: return False
    return True

def water_match_ref(base, tgt, s, haric):
    """§7.1: hedef - s*base üzerinde NNLS; beklenen profil = s*base + A x. Döner dict(profile, rnorm, x_salts)."""
    aktif = [k for k in A if k not in haric]
    cols = [A[k] for k in aktif]
    b = [tgt[i]-s*base[i] for i in range(6)]
    x, rn = nnls_lh(cols, b)
    prof = [s*base[i]+sum(x[j]*cols[j][i] for j in range(len(cols))) for i in range(6)]
    return dict(profile=prof, rnorm=rn, x=[x[aktif.index(k)] if k in aktif else 0.0 for k in A], cols=cols, b=b, xa=x)

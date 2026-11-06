"""Hop/IBU cross-check: BeerSmith 4 spesifikasyonu (02-hop-ibu.md) vs Brewtarget (src/measurement/IbuMethods.cpp,
src/model/Recipe.cpp::ibuFromHopAddition).  Python 3, bagimlilik yok."""
import math

# ---------------- Ortak sabitler ----------------
OZ_G   = 28.349523125
GAL_L  = 3.785411784
K_BS   = 100/1.34            # BeerSmith IBU sabiti (74.6269)

# ---------------- BeerSmith (spesifikasyondan) ----------------
def bs_tin(sg, t):  return 1.65*0.000125**(sg-1)*(1-math.exp(-0.04*t))/4.15
def bs_ftemp(Tc):   return 1.0 if Tc >= 100 else 2.39e11*math.exp(-9773/(Tc+273.15))
RT=[(5,.05),(10,.06),(15,.08),(20,.101),(25,.121),(30,.153),(35,.188),(40,.228),(45,.269),(50,.281),(55,.291)]
def bs_rager(sg, t):
    n=int(t+0.5)
    if n==0: r=0.0
    else:
        r=0.30
        for lim,v in RT:
            if n<=lim: r=v; break
    ga=(sg-1.05)/0.2 if sg>1.05 else 0.0
    return r/(1+ga)
GT=[0,0,2,5,8,11,14,16,18,19,20,21,22,23,23]
def bs_garetz_ibu(w,a,t,sg,v,bv):
    if t<=0 or w<=0 or a<0: return 0.0
    idx=min(int((int(t)-1)/5),14); r=v/bv; GF=((sg-1)*r+1-1.05)*5+1
    prev,cur=1.0,50.0
    while abs(prev-cur)>0.05:
        prev=cur; cur=GT[idx]*a*w*0.749/(v*GF*(r*cur/260+1)*1.02)
    return cur
def bs_postboil(sg,t0,dur):
    s=0.0; t=t0
    while t<t0+dur:
        f=2.39e11*math.exp(-9773/(((t-t0)*-1.344+210.64+459.67)*(5/9)))
        if t<5.0: f=1.0
        s+=1.65*0.000125**(sg-1)*0.04*math.exp(-0.04*t)/4.15*f*0.1
        t+=0.1
    return s
def bs_combine(segs):
    p=1.0
    for s in segs: p*=1-s
    return 1-p
def bs_ibu(U, oz, a_pct, v_gal, adj=1.0): return adj*U*a_pct*oz*K_BS/v_gal
def bs_boiltemp(alt_ft): return 100-alt_ft*0.3048*0.00324

# ---------------- Brewtarget (C++'tan birebir tasima) ----------------
# IbuMethods.cpp:45-94 calculateDecimalAlphaAcidUtilization
def bt_tin_util(sg,t): return 1.65*pow(0.000125,sg-1.0)*(1.0-math.exp(-0.04*t))/4.15
# IbuMethods.cpp:99-150
def bt_tinseth(aa,g,vl,sg,t,**k): return bt_tin_util(sg,t)*(aa*g*1000)/vl
# IbuMethods.cpp:152-158 (Rager tanh egrisi)
def bt_rager_util(sg,t):
    u=(18.11+13.86*math.tanh((t-31.32)/18.17))/100.0
    gf=(sg-1.050)/0.2 if sg>1.050 else 0.0
    return u/(1+gf)
def bt_rager(aa,g,vl,sg,t,**k): return g*bt_rager_util(sg,t)*aa*1000/vl
# IbuMethods.cpp:163-189 Noonan
NP=[0.7000029428,-0.08868853463,0.02720809386,-0.002340415323,0.00009925450081,-0.000002102006144,0.00000002132644293,-0.00000000008229488217]
def bt_noonan(aa,g,vl,sg,t,**k):
    vf=(5*GAL_L)/vl; hf=g/(OZ_G/1000*1000)
    p=sum(c*t**i for i,c in enumerate(NP))
    uf=1 if sg<=1.050 else 0.9286 if sg<=1.065 else 0.8571 if sg<=1.085 else 0.75
    return vf*(hf*(100*aa)*p)*uf
# IbuMethods.cpp:197-225 computePostBoilUtilization
def bt_postboil(tb,sg,vl,cool,dint,dopen,delay,dt=0.001):
    sa=math.pi*(dint/2)**2; oa=math.pi*(dopen/2)**2; ea=math.sqrt(sa*oa)
    b=0.0002925*ea/vl+0.00538
    s=0.0; t=tb
    n=int(round(cool/dt))
    for i in range(n):           # ayni Riemann toplami (C++ for dongusu, 0.001 adim)
        t=tb+i*dt
        dU=1.65*0.000125**(sg-1)*0.04*math.exp(-0.04*t)/4.15
        Tk=53.70*math.exp(-b*(t-tb+delay))+319.55
        f=1.0 if t<5.0 else 2.39e11*math.exp(-9773.0/Tk)
        s+=dU*f*dt
    return s
def bt_mibu(aa,g,vl,sg,t,cool=0.0,dint=45.0,dopen=45.0,delay=0.0):
    u=bt_tin_util(sg,t)+bt_postboil(t,sg,vl,cool,dint,dopen,delay)
    return u*aa*g*1000/vl, u
def bt_temp_c(tmin,vl,dint=45.0,dopen=45.0):
    ea=math.pi*(dint/2)**2*0+math.sqrt(math.pi*(dint/2)**2*math.pi*(dopen/2)**2)
    b=0.0002925*ea/vl+0.00538
    return 53.70*math.exp(-b*tmin)+319.55-273.15, b
# Recipe.cpp:3040-3176 form/ekipman duzeltmeleri
def bt_form(form): return {'pellet':1.10,'plug':1.02}.get(form,1.0)

# ---------------- Senaryolar ----------------
oz,a,sg,vg=1.0,5.0,1.050,5.0
g,aa,vl=oz*OZ_G,a/100,vg*GAL_L
rows=[]
def row(name,bs,bt,note=""):
    d = "" if (bs is None or bt is None or bs==0) else "%+.2f%%"%((bt-bs)/bs*100)
    f=lambda x:"—" if x is None else "%.4f"%x
    rows.append((name,f(bs),f(bt),d,note))

row("1 Tinseth 60 dk (BS K=74.63 / BT 74.89)", bs_ibu(bs_tin(sg,60),oz,a,vg), bt_tinseth(aa,g,vl,sg,60))
row("2 Rager 60 dk (BS tablo / BT tanh)", bs_ibu(bs_rager(sg,60),oz,a,vg), bt_rager(aa,g,vl,sg,60))
row("3 Rager 60 dk SG 1.070", bs_ibu(bs_rager(1.07,60),oz,a,vg), bt_rager(aa,g,vl,1.07,60))
row("3b Rager 20 dk", bs_ibu(bs_rager(sg,20),oz,a,vg), bt_rager(aa,g,vl,sg,20))
row("4 Garetz 60 dk (arac yolu)", bs_garetz_ibu(oz,a,60,sg,vg,vg), None, "BT'de Garetz yok (IbuMethods.h TODO)")
row("5 Noonan 60 dk", None, bt_noonan(aa,g,vl,sg,60), "BS'de Noonan yok")
row("6 mIBU 60 dk, sogutma 0 dk (=Tinseth)", bs_ibu(bs_tin(sg,60),oz,a,vg), bt_mibu(aa,g,vl,sg,60,cool=0)[0])
bs10=bs_ibu(bs_combine([bs_tin(sg,60),bs_postboil(sg,60,20)]),oz,a,vg)
row("7 60 dk + 20 dk sicak bekleme (BS carry / BT mIBU cool=20)", bs10, bt_mibu(aa,g,vl,sg,60,cool=20)[0])
row("7b 60 dk, BT mIBU varsayilan cool=30 (Boil::default_coolTime_mins)", bs_ibu(bs_tin(sg,60),oz,a,vg), bt_mibu(aa,g,vl,sg,60,cool=30)[0], "BS carry kapali ile kiyas")
row("8 0 dk flameout + 20 dk sogutma (BS PostBoilUtil / BT mIBU)", bs_ibu(bs_postboil(sg,0,20),oz,a,vg), bt_mibu(aa,g,vl,sg,0,cool=20)[0])
row("8b 0 dk flameout, Tinseth (sogutma modeli yok)", 0.0, bt_tinseth(aa,g,vl,sg,0), "her ikisi 0")
# whirlpool 80 C 20 dk
bs_wp=bs_ibu(bs_tin(sg,20)*bs_ftemp(80),oz,a,vg)
# BT: 80 C'ye inmek icin gereken bekleme (kazan 45 cm varsayilan)
_,b=bt_temp_c(0,vl); delay80=-math.log((80+273.15-319.55)/53.70)/b
row("9 Whirlpool 80 C 20 dk, BT Tinseth/Rager/Noonan", bs_wp, 0.0, "BT post-boil eklemeye IBU vermez (RecipeCalcTests.cpp:336)")
row("9b Whirlpool 80 C 20 dk, BT mIBU gecikme=%.1f dk (80 C'ye inene kadar)"%delay80, bs_wp, bt_mibu(aa,g,vl,sg,0,cool=20,delay=delay80)[0], "tau<5 istisnasi ilk 5 dk f=1 veriyor")
row("9c Whirlpool hop flameout'ta, 20 dk hop stand, BT mIBU gecikme=0", bs_wp, bt_mibu(aa,g,vl,sg,0,cool=20,delay=0)[0], "BS'de sicaklik sabit 80 C")
# tau<5 istisnasi olmasaydi (ogretici)
def bt_postboil_no5(tb,sg,vl,cool,delay,dt=0.001):
    ea=math.pi*22.5**2; b=0.0002925*ea/vl+0.00538; s=0
    for i in range(int(round(cool/dt))):
        t=tb+i*dt; dU=1.65*0.000125**(sg-1)*0.04*math.exp(-0.04*t)/4.15
        Tk=53.70*math.exp(-b*(t-tb+delay))+319.55; s+=dU*2.39e11*math.exp(-9773/Tk)*dt
    return s
row("9d (9b) tau<5 istisnasi kaldirilmis mIBU", bs_wp, bt_postboil_no5(0,sg,vl,20,delay80)*aa*g*1000/vl, "duzeltilmis model")
row("8c (8) tau<5 istisnasi kaldirilmis mIBU", bs_ibu(bs_postboil(sg,0,20),oz,a,vg), bt_postboil_no5(0,sg,vl,20,0)*aa*g*1000/vl)
Tb=bs_boiltemp(5000)
row("10 Rakim 5000 ft, Tinseth 60", bs_ibu(bs_tin(sg,60)*bs_ftemp(Tb),oz,a,vg), bt_tinseth(aa,g,vl,sg,60), "BT rakim/kaynama noktasini IBU'da kullanmaz")
row("11 Pellet Tinseth 60", bs_ibu(bs_tin(sg,60),oz,a,vg,1.10), bt_tinseth(aa,g,vl,sg,60)*bt_form('pellet'))
row("12 FWH pellet 60", bs_ibu(bs_tin(sg,60),oz,a,vg,1.21), bt_tinseth(aa,g,vl,sg,60)*1.1*1.10)
row("13 Mash hop pellet 60 (varsayilanlar)", bs_ibu(bs_tin(sg,60),oz,a,vg,1.10*0.20), 0.0, "BT mashHopAdjustment varsayilan 0")
# hacim: 5 gal fermentor + 0.5 gal trub
row("14 Tinseth 60, 5 gal parti + 0.5 gal trub", bs_ibu(bs_tin(sg,60),oz,a,5.5), bt_tinseth(aa,g,vl,sg,60), "BS V+trub, BT batchSize")
# gec ekstrakt: BS segment, BT pre-boil SG late ekstrakt haric -> 1.040
segs=[bs_tin(1.040,60)-bs_tin(1.040,15), bs_tin(1.050,15)]
row("15 Gec ekstrakt (1.040 -> 15 dk'da 1.050)", bs_ibu(bs_combine(segs),oz,a,vg), bt_tinseth(aa,g,vl,1.040,60), "BT gec eklemeyi boilGrav'dan cikarir, segment yok")
# izomerize ekstrakt 0.1 fl oz %50
ml=0.1*29.5735295625
row("16 Izomerize ekstrakt 0.1 fl oz %50 (BT: hacim agirlik sayilir, 60 dk)", 0.1*50*K_BS/5, bt_tinseth(0.5,ml,vl,sg,60), "BT'de izomerize ekstrakt ozel durumu yok")
# hopped extract (fermentable) 1 lb, 10 IBU*gal/lb, 5 gal
kg=0.45359237
row("17 Hopped extract 1 lb, 10 IBU.gal/lb, 5 gal", 1*10/5, 10*(kg/vl)/8.34538, "BT Recipe.cpp:1271 bolme yerine carpma olmali")
row("17b (17) BT carpma ile duzeltilmis", 1*10/5, 10*(kg/vl)*8.34538)

print("| Senaryo | BeerSmith IBU | Brewtarget IBU | Fark (BT-BS)/BS | Not |")
print("|---|---|---|---|---|")
for r in rows: print("| "+" | ".join(r)+" |")
print()
print("Kullanim orani tablosu (SG 1.050):")
print("| dk | Tinseth (ikisi ayni) | BS Rager (tablo) | BT Rager (tanh) | BS Garetz | BT Noonan (U esdegeri) |")
print("|---|---|---|---|---|---|")
for m in (5,10,15,20,30,45,60,90):
    nu=bt_noonan(aa,g,vl,sg,m)/(aa*g*1000/vl)
    print("| %d | %.4f | %.4f | %.4f | %.4f | %.4f |"%(m,bs_tin(sg,m),bs_rager(sg,m),bt_rager_util(sg,m),bs_garetz_ibu(1,1,m,sg,1,1)/74.9,nu))
print()
print("BT mIBU sicaklik egrisi (45 cm kazan, 18.93 L): b=%.5f/dk"%b)
for tm in (0,5,10,15,20,30): print("  t=%2d dk: BT %.2f C | BS dogrusal %.2f C"%(tm,bt_temp_c(tm,vl)[0],(210.64-1.344*tm-32)*5/9))
print("80 C'ye inme suresi: BT %.2f dk, BS %.2f dk"%(delay80,(210.64-176)/1.344))

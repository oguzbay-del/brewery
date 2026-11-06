from math import log, exp, sqrt, floor
FLOZ_ML=29.57352956; OZ_G=0.4535924/16*1000
def L_to_floz(v): return v*1000/FLOZ_ML
def floz_to_L(v): return v*FLOZ_ML/1000
def gal_from_L(v): return v*1000/FLOZ_ML/128
def ToPlato(sg): return 135.997*sg**3 - 630.272*sg**2 + 1111.14*sg - 616.868
def ToSG(p): return p/(258.6-(p/258.6)*227.1)+1
def BrixToSG(b): return b/(258.6-(b/258.2)*227.1)+1
def F(c): return c*9/5+32
STIR=45.0; GROWTH_LIMIT=4.0
def viability(v0, rate, days, floor_=5.0):
    if days<=0: return v0
    v=v0*((100-rate)/100)**(days/30.0); return max(v,floor_)
def InnocRate(g): return exp(log(g)*-2.1791038+5.512221)
def GrowthRate(I,stir):
    g=exp((log(I)-5.512221)/-2.1781038)
    if stir: g*=1+STIR/100
    return min(max(g,1.0),6.0) if True else g
def cells_needed(rate, V_L, sg): return rate*V_L*ToPlato(sg)
def new_starter_L(need, avail, stir):
    if not need>avail: return 0.0
    r=need/avail
    if stir: r/=1+STIR/100
    if r<=1: return 0.0
    return avail/InnocRate(r)
def two_stage_tool(need, avail, stir):
    avail=max(avail,0.1)
    if not avail<=need: return 0,0
    r=need/avail; s=1.0
    if stir: s=1+STIR/100; r/=s*s
    g=sqrt(r)
    v1=avail/InnocRate(g); v2=avail*g*s/InnocRate(g)
    return v1,v2
def dme_oz(vol_floz, og): return (og-1)*1000*vol_floz/128/45*16
def sugar_g(vols,T_F,gal,dme=False):
    T=max(T_F,32.0); g=((T*0.050062+(vols-3.0378))-T*0.00026555*T)*gal*15.195
    return g*1.538 if dme else g
def residual(T_F): T=max(T_F,32.0); return 3.0378-0.050062*T+0.00026555*T*T
def co2_psi(vols,T_F):
    T=max(T_F,32.0); p=vols*4.24267+(-16.6999-T*0.0101059)+T*0.00116512*T+T*0.173354*vols-vols*0.0684226*vols
    return max(p,0.0)
def so2_rec(pH): return exp(pH*2.21828-4.27809)
def kmeta_g(V_gal,tgt,free): return V_gal*3.785*(tgt-free)/576.0
def sorbate_g(V_gal): return V_gal*1.25
def mead_pitch(sg):
    p=ToPlato(sg)
    if p<=24: return 0.25
    if p<28: return (p-24)*0.0625+0.25
    return 0.5
def tosna(sg, V_gal, pct, fac, method):
    k={0:0.75,1:0.9,2:1.25}[fac]
    base=ToPlato(sg)*10*(pct/100)*k*V_gal
    return base/50 if method==0 else base/100
def yan(method, grams, V_L):
    r={0:40.0,1:100.0,2:212.3}.get(method,100.0)
    return r*grams/V_L if V_L!=0 else r
P=print
P("== Canlilik"); 
for nm,v0,r in [("Sivi (genel, 10%/ay)",97,10),("Sivi White Labs (5%/ay)",97,5),("Kuru (1.66%/ay)",98,1.66),("Slurry (25%/ay)",96,25)]:
    v=viability(v0,r,90); P(f"{nm}: 90 gun -> {v:.2f} %  ; 100 milyar hucreli pakette canli = {100*v/100:.2f} milyar")
P("== 20 L 1.060 ale")
p=ToPlato(1.060); need=cells_needed(0.75,20,1.060)
P(f"Plato={p:.4f}  gerekli hucre={need:.2f} milyar  (min {0.45*20*p:.1f}, max {1.05*20*p:.1f})  EstCellsML={need*1000/20000:.3f} M/ml")
avail=100*viability(97,10,90)/100
for stir in (False,True):
    V=new_starter_L(need,avail,stir); P(f"stir={stir}: avail={avail:.2f} -> starter {V:.4f} L = {L_to_floz(V):.2f} fl oz; DME(1.036)={dme_oz(L_to_floz(V),1.036):.3f} oz = {dme_oz(L_to_floz(V),1.036)*OZ_G:.1f} g")
    I=avail/V; g=GrowthRate(I,stir); P(f"   kontrol: I={I:.2f} M/ml -> buyume {g:.4f} -> {avail*g:.2f} milyar")
    v1,v2=two_stage_tool(need,avail,stir); P(f"   iki kademe: V1={v1:.4f} L, V2={v2:.4f} L")
fresh=100*0.97
for stir in (False,True):
    V=new_starter_L(need,fresh,stir); P(f"taze paket (97 milyar) stir={stir}: {V:.4f} L")
P("== Kuru maya 20 L, 75 g/hL ->", 20*75/100, "g ; hucre(arac)=", 20*7.5, "milyar ; 11.5g paket=", (20*75/100)/11.5, "; hidrasyon ml=",20*75/100*25, "; GoFerm g=",20*75/100*25*0.05)
P("== Karbonasyon 2.4 vol, 20C, 19 L")
gal=gal_from_L(19); T=F(20)
g=sugar_g(2.4,T,gal); P(f"gal={gal:.4f}, T={T}F, residual={residual(T):.4f} vol; misir sekeri={g:.2f} g ({g/OZ_G:.3f} oz); sofra={g/1.099:.2f} g; DME={g*1.538:.2f} g")
P("== Basinc 4C 2.5 vol:", f"{co2_psi(2.5,F(4)):.3f} psi = {co2_psi(2.5,F(4))*6.894757:.2f} kPa (T={F(4)}F)")
P("   kontrol 38F 2.5vol:", f"{co2_psi(2.5,38):.3f}", " 68F 2.5:",f"{co2_psi(2.5,68):.3f}")
P("== SO2 pH 3.5:", f"{so2_rec(3.5):.3f} ppm; 19 L icin K-meta (0 serbestten) = {kmeta_g(gal,so2_rec(3.5),0):.4f} g ; sorbat = {sorbate_g(gal):.3f} g")
for ph in (3.0,3.2,3.4,3.6,3.8,4.0): P(f"   pH {ph}: {so2_rec(ph):.1f} ppm")
P("== Mead 1.100, 19 L, %100 bal, orta N")
gal=gal_from_L(19)
fo=tosna(1.100,gal,100,1,0); fk=tosna(1.100,gal,100,1,1)
P(f"Plato={ToPlato(1.100):.3f}; Fermaid-O toplam={fo:.3f} g (4x {fo/4:.3f}); Fermaid-K toplam={fk:.3f} g (4x {fk/4:.3f})")
P(f"YAN(O)={yan(0,fo,19):.2f} ppm, esdeger={4*yan(0,fo,19):.2f}; YAN(K)={yan(1,fk,19):.2f} ppm")
P(f"Mead maya: {mead_pitch(1.100)} g/L -> {19*mead_pitch(1.100):.2f} g; 1.120 icin {mead_pitch(1.120):.4f} g/L (P={ToPlato(1.120):.3f})")
P(f"GoFerm (5 g maya) = {5*25*0.05} g, su = {5*25} ml")
P("== Backsweeten: 19 L 0.998 -> 1.010, bal 82.5 Bx")
sgA=ToSG(82.5); Va=19*(0.998-1.010)/(1.010-sgA)
P(f"Plato->SG(82.5)={sgA:.5f}; bal hacmi={Va:.4f} L; agirlik(Brix->SG)={BrixToSG(82.5)*Va:.4f} kg  (BrixToSG={BrixToSG(82.5):.5f})")
P("== Birim: 2.4 vol -> g/L", 2.4*1.9642, " ; 4.71408 g/L -> vol", 4.71408/1.96042)

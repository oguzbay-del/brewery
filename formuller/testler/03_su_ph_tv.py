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
def gristph(bill):
    W=sum(w for w,c in bill)
    s=0
    for w,c in bill:
        f=w/W
        if c<14: g=c-1.9
        elif c<200: g=0.453*c+5.0
        else: g=42.0
        s+=f*g
    return 5.72-0.0337*s, s
def slope(model,t):
    return t*0.47930613*0.17/1.5 if model==1 else 0.037+0.014*t
def pks(a):
    return {2:(2.12,7.2,12.44),4:(3.14,4.77,6.39),5:(4.75,20,20),6:(-7,20,20),7:(-1,1.92,20),8:(2.98,4.34,20)}.get(a,(3.86,20,20))
def acidfrac(a,ph):
    k1,k2,k3=pks(a); r1=10**(ph-k1); r2=10**(ph-k2); r3=10**(ph-k3)
    f0=1/(1+r1+r1*r2+r1*r2*r3)
    return r1*f0+2*r1*r2*f0+3*r1*r2*r3*f0
def acid_per_L(meq,acid,conc,model):
    if acid==0: return 0
    if acid in (1,3): mw=90.09; rho=1+conc*0.237/100
    else: mw=98.0; rho=1+0.49*conc/100+0.375*(conc/100)**2
    f=acidfrac(acid,5.4) if model==1 else 1.0
    if acid==3: return meq*100*mw/(conc*f*1000)
    return meq*100*mw/(conc*f*rho*1000)
bill=[(4.5,2.0),(0.3,60.0),(0.1,350.0)]
W=sum(w for w,c in bill); t=3.0
gp,s=gristph(bill); print("T3 grist sum",s,"gristpH",gp)
# mash water: base + 5 g gypsum + 3 g CaCl2 in 20 L mash water
mash=[base[k]+p[i] for i,k in enumerate(ions)]
ra=RA(mash[0],mash[1],mash[5]); print("T3 mash profile",[round(x,2) for x in mash],"RA",ra)
print("acidfrac lactic 5.4",acidfrac(1,5.4),"phos",acidfrac(2,5.4))
for model in (0,1):
    sl=slope(model,t); est=gp+ra/50*sl
    dph=est-5.4
    meq=dph/sl
    perL=acid_per_L(meq,1,88,model)
    mL=perL*W*t
    perLm=acid_per_L(meq,3,3.0,model)  # acid malt 3%
    print(f"T3 model{model} slope={sl:.5f} estpH={est:.4f} meq/L={meq:.4f} lactic88 mL/L={perL:.5f} total mL={mL:.3f} acidmalt(3%) g={perLm*W*t:.1f}")
    # verify via MiscAcidRA
    p_=0.88; rho=1+0.237*p_; f=acidfrac(1,5.4) if model==1 else 1
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
def lactic_mL(molL,V,conc):
    c=max(conc,1.0)/100
    return (1/c)*0.09009/(1+0.237*c)*molL*V*1000
hco3=150; phw=7.5; pht=5.4
Vtot=30.0; Vsp=Vtot - W*t
m1=sparge_mpH(hco3,phw,pht); m2=sparge_bnw(hco3,phw,pht,1)
print("T4 Vsp",Vsp,"MPH mol/L",m1,"mL",lactic_mL(m1,Vsp,88),"BNW mol/L",m2,"mL",lactic_mL(m2,Vsp,88))
print("T5 SO4/Cl", mash[3]/mash[4])
print("T6 color range", 5.2+0.082*ra, (ra+122.4)/12.2)
# boil scale
Vb,Vp,cool,trub,top=28.0,24.0,1.0,1.0,0.0
a_=Vp-cool; b_=a_-trub
print("T7 boilscale",((Vb/a_)*b_+top)/(b_+top))
def adj_ph(bill_, acidmalt_g, pct, model, ra, t):
    b=bill_+([(acidmalt_g/1000,3.0)] if acidmalt_g>0 else [])
    W=sum(w for w,c in b); gp,_=gristph(b); sl=slope(model,t)
    f=acidfrac(3,5.4) if model==1 else 1
    acidra=-((pct/100)*1000*(acidmalt_g/1000)*f)/(W*t*0.09009)
    return gp+sl*(ra/50+acidra), W
for model in (0,1):
    tot=0.0
    for it in range(8):
        ph,W=adj_ph(bill,tot,3.0,model,ra,t)
        sl=slope(model,t)
        amt=acid_per_L((ph-5.4)/sl,3,3.0,model)*W*t if ph-5.4>=0 else 0
        tot+=amt
        ph2,_=adj_ph(bill,tot,3.0,model,ra,t)
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

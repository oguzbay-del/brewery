import math
def ToPlato(sg): return 135.997*sg**3 - 630.272*sg**2 + 1111.14*sg - 616.868
def ToSG(p): return p/(258.6 - (p/258.6)*227.1) + 1.0
def BrixToSG(b): return b/(258.6 - (b/258.2)*227.1) + 1.0
def SGToBrix(sg): return ((182.4601*sg - 775.6821)*sg + 1262.7794)*sg - 669.5622
def BrixToRI(b): return 1.33302 + 0.001427193*b + 5.791157e-6*b*b
def RIToBrix(ri): return (math.sqrt(2.036879859249e-06 - (1.33302-ri)*2.3164628e-05) - 0.001427193)/1.1582314e-05
def RE(og,fg):
    o=ToPlato(og); f=ToPlato(fg); o=max(o,f); return 0.1808*o+0.8192*f
def ABW(og,fg):
    o=ToPlato(og); f=ToPlato(fg); o=max(o,f); re=0.1808*o+0.8192*f
    return (o-re)/(2.0665-0.010665*o)
def ABV(og,fg): return fg/0.79661*ABW(og,fg)
def AA(og,fg):
    o=ToPlato(og); f=ToPlato(fg); o=max(o,f); return (o-f)/o*100
def RA(og,fg):
    o=ToPlato(og); f=ToPlato(fg); o=max(o,f); return (1-(0.1808*o+0.8192*f)/o)*100
def alc_cal(og,fg): return 1881.22*fg*(og-fg)/(1.775-og)
def ext_cal(og,fg): return 3550.0*fg*((0.1808*og+0.8192*fg)-1.0004)
def uk(og,fg):
    d=(og-fg)*1000; f=0.125
    for lim,ff in [(6.95,.126),(10.45,.127),(17.25,.128),(26.15,.129),(36.05,.13),(46.55,.131),(57.15,.132),(67.95,.133),(78.85,.134),(89.75,.135)]:
        if d>lim: f=ff
    return f*d
def morey(lb_srm_sum, gal): return 1.4922*(lb_srm_sum/gal)**0.6859
def hydro(sg,T_F,Tc_F):
    t=(T_F-32)*5/9; tc=(Tc_F-32)*5/9
    return sg + sg*((tc**3-t**3)*1.53243635e-08 + (tc-t)*1.62860587e-05 - (tc**2-t**2)*5.84994855e-06)
def refr0(b,zero=0.0,wcf=1.02): return ToSG((b-zero)/wcf)
def refr1(og,b,zero=0.0):
    ob=ToPlato(og); fb=b-zero
    return 1.001843-0.002318474*ob-7.775e-6*ob*ob-3.4e-8*ob**3+0.00574*fb+3.344e-5*fb*fb+8.6e-8*fb**3
def refr2(fg,b,zero=0.0):
    ri=BrixToRI(b-zero)
    abw=(1017.5596-277.4*fg+ri*(937.8135*ri-1805.1288))/100.0
    re=194.5935+129.8*fg+ri*(410.8815*ri-790.8732)
    oe=(re+2.0665*abw*100)/(1+0.010665*abw*100)
    return ToSG(oe), abw*100, re, oe
def fg_tol(og,fg,tol):
    hi=1.18; lo=fg
    for i in range(50):
        mid=(hi+lo)/2; a=ABV(og,mid)
        if abs(a-tol)<0.04: return mid
        if a>tol: lo=mid
        elif a<tol: hi=mid
    return mid
def gadj_oz(g,t,vol_floz,typ):
    p=(t-g)*1000
    if p<=0: return 0
    p*=max(vol_floz,0.01)/128
    return p*16/(44 if typ==0 else 36)
def gadj_water(g,t,v): return max(0, v*(g-1)*1000/((t-1)*1000)-v)
print("ToPlato(1.050)=%.4f ToSG(12)=%.5f BrixToSG(12)=%.5f SGToBrix(1.050)=%.4f"%(ToPlato(1.050),ToSG(12),BrixToSG(12),SGToBrix(1.050)))
print("RI(10Bx)=%.6f Brix(RI)=%.4f"%(BrixToRI(10),RIToBrix(BrixToRI(10))))
for og,fg in [(1.050,1.010),(1.065,1.012),(1.080,1.016)]:
    print(og,fg,"ABW=%.3f ABV=%.3f AA=%.2f RA=%.2f RE=%.3f alc=%.1f ext=%.1f tot=%.1f UK=%.3f"%(ABW(og,fg),ABV(og,fg),AA(og,fg),RA(og,fg),RE(og,fg),alc_cal(og,fg),ext_cal(og,fg),alc_cal(og,fg)+ext_cal(og,fg),uk(og,fg)))
# OG example: 10 lb (160 oz) 2-row yield 80%, 1 lb crystal 72%, 1 lb corn sugar (type 2, yield 100%), 5.5 gal (704 floz), eff 72%
pts = 160*(80*46/100)*72/1600 + 16*(72*46/100)*72/1600 + 16*(100*46/100)/16
og = pts/(704/128)/1000+1
print("pts=%.3f OG=%.4f"%(pts,og))
# FG: atten avg (73+77)/2=75, mash step 152F -> adj = -1.25*(152-153.5)=+1.875 -> 76.875
att=75+(-1.25)*(152-153.5)
fgp = 160*(80*46/100)*72/1600*(1-att/100) + 16*(72*46/100)*72/1600*(1-att/100) + 16*(100*46/100)/16*(-0.231)
fg = fgp/(704/128)/1000+1
print("att=%.3f fgpts=%.3f FG=%.4f ABV=%.3f"%(att,fgp,fg,ABV(og,fg)))
print("tol: fg_tol(1.100,1.010,10)=%.4f ABV=%.3f"%(fg_tol(1.100,1.010,10),ABV(1.100,fg_tol(1.100,1.010,10))))
# color: 10lb 2 SRM + 1 lb 60 SRM, vol 5.5 + trub 0.5 = 6 gal
mcu=(10*2+1*60)/6
print("MCU=%.3f SRM=%.3f EBC=%.3f"%(mcu,morey(80,6),morey(80,6)*1.97))
print("Hydro 1.050 @ 77F cal 60F -> %.5f"%hydro(1.050,77,60))
print("Refr0 12Bx -> %.5f"%refr0(12))
print("Refr1 OG1.050, 6.5Bx -> FG %.5f, ABV %.3f"%(refr1(1.050,6.5),ABV(1.050,refr1(1.050,6.5))))
r=refr2(1.010,6.5); print("Refr2 FG1.010, 6.5Bx -> OG %.5f ABW%% %.3f RE %.3f OE %.3f"%r)
print("GravAdj 1.040->1.050 5gal DME oz=%.3f LME oz=%.3f"%(gadj_oz(1.040,1.050,640,0),gadj_oz(1.040,1.050,640,1)))
print("Water 1.060->1.050 5gal add floz=%.2f (gal %.3f)"%(gadj_water(1.060,1.050,640),gadj_water(1.060,1.050,640)/128))
print("Dilution water: 5gal 1.060 + 1 gal -> %.5f; wort 1gal@1.030 -> %.5f"%(1+(5/6)*0.060, 1+0.060*5/6+0.030/6))
# efficiency: 10 lb grain 80% yield, measured OG 1.050 in 5.5 gal, plus 1 lb sugar
pot=(160*(80*46/100)/16)/(704/128); meas=50 - (16*46/16)/(704/128)
print("Eff=%.2f%%"%(meas/pot*100))
print("UK 1.050/1.010 =%.3f"%uk(1.050,1.010))

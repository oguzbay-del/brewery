import math
K=74.62686567164178
def tin_util(sg,t): return 1.65*0.000125**(sg-1)*(1-math.exp(-0.04*t))/4.15
def tempf(Tc): return 1.0 if Tc>=100 else 2.39e11*math.exp(-9773/(Tc+273.15))
RT=[(0,0),(5,.05),(10,.06),(15,.08),(20,.101),(25,.121),(30,.153),(35,.188),(40,.228),(45,.269),(50,.281),(55,.291)]
def rager_tab(n):
    if n==0: return 0.0
    for lim,v in RT[1:]:
        if n<=lim: return v
    return 0.30
def rager_ga(sg): return (sg-1.05)/0.2 if sg>1.05 else 0.0
def rager_util(sg,t): return rager_tab(int(t+0.5))/(1+rager_ga(sg))
GT=[0,0,2,5,8,11,14,16,18,19,20,21,22,23,23]
def garetz_ibu(w,a,t,sg,v,bv):
    if t<=0 or w<=0 or a<0: return 0.0
    idx=int((int(t)-1)/5)  # C truncation
    idx=min(idx,14)
    r=v/bv; GF=((sg-1)*r+1-1.05)*5+1
    prev,cur=1.0,50.0
    while abs(prev-cur)>0.05:
        prev=cur
        cur=GT[idx]*a*w*0.749/(v*GF*(r*cur/260+1)*1.02)
    return cur
def garetz_util(sg,t): return garetz_ibu(1,1,t,sg,1,1)/74.9
def postboil(sg,t0,dur):
    s=0.0; t=t0
    while t<t0+dur:
        f=2.39e11*math.exp(-9773/(((t-t0)*-1.344+210.64+459.67)*(5/9)))
        if t<5.0: f=1.0
        s+=1.65*0.000125**(sg-1)*0.04*math.exp(-0.04*t)/4.15*f*0.1
        t+=0.1
    return s
def boiltempC(alt_ft): return 100-alt_ft*0.3048*0.00324
def combine(segs):
    tot,rem=0.0,1.0
    for s in segs: tot+=s*rem; rem*=1-s
    return tot
def hop_alpha(a,hsi,tF,days,sf):
    hsi=min(hsi,99); k=-math.log(1-hsi/100)/180
    return a*math.exp(-k*0.5**((68-tF)/27)*days*sf)
w,a,t,sg,v=1.0,5.0,60,1.050,5.0
print("Tinseth util %.6f IBU %.4f"%(tin_util(sg,t), tin_util(sg,t)*a*w*K/v))
print("Rager util %.6f IBU %.4f"%(rager_util(sg,t), rager_util(sg,t)*a*w*K/v))
print("Garetz tools IBU %.4f"%garetz_ibu(w,a,t,sg,v,v))
gu=garetz_util(sg,t); print("Garetz recipe util %.6f IBU %.4f"%(gu,gu*a*w*K/v))
print("Garetz recipe OG1.040 util %.6f"%garetz_util(1.040,60))
print("Rager OG1.070 util %.6f IBU %.4f"%(rager_util(1.07,60),rager_util(1.07,60)*a*w*K/v))
wu=tin_util(sg,20)*tempf(80); print("Whirlpool 80C 20dk: tempf %.6f util %.6f IBU %.4f"%(tempf(80),wu,wu*a*w*K/v))
print("tempf(100)=%.6f tempf(99.99)=%.6f"%(tempf(100),tempf(99.99)))
for alt in (0,5000):
    Tc=boiltempC(alt); print("alt %d ft: boil %.4f C (%.3f F) tempf %.6f, Tinseth60 IBU %.4f"%(alt,Tc,Tc*9/5+32,tempf(Tc),tin_util(sg,60)*tempf(Tc)*a*w*K/v))
pb=postboil(sg,60,20); print("PostBoilUtil(1.050,60,20)=%.6f"%pb)
tot=combine([tin_util(sg,60),pb]); print("60dk+carry20 util %.6f IBU %.4f"%(tot,tot*a*w*K/v))
pb0=postboil(sg,0,20); print("PostBoilUtil(1.050,0,20)=%.6f IBU(0dk hop) %.4f"%(pb0,pb0*a*w*K/v))
print("Recipe pellet Tinseth (x1.10) IBU %.4f"%(1.10*tin_util(sg,60)*a*w*K/v))
print("FWH pellet 60 (x1.10x1.10) IBU %.4f"%(1.21*tin_util(sg,60)*a*w*K/v))
print("Mash pellet 60 (x1.10x0.20) IBU %.4f"%(1.1*0.2*tin_util(sg,60)*a*w*K/v))
print("ISO extract 0.1 floz 50%% 5 gal IBU %.4f"%(0.1*50*100*128/(5*128*1.34)))
# late extract example: boil 60, extract added at 15 min raises SG 1.040->1.050
segs=[tin_util(1.040,60)-tin_util(1.040,15), tin_util(1.050,15)-tin_util(1.050,0)]
print("Late-extract 2 segments util %.6f (vs tek SG1.040 %.6f)"%(combine(segs),tin_util(1.040,60)))
print("HopAlpha 5%%,HSI30,6ay(180g),68F,poly: %.4f ; 40F vacuum: %.4f"%(hop_alpha(5,30,68,180,1.0),hop_alpha(5,30,40,180,0.5)))
for m in (5,10,15,20,30,45,60,90): print(m,"tin %.4f rager %.4f garetz %.4f"%(tin_util(1.05,m),rager_util(1.05,m),garetz_util(1.05,m)))

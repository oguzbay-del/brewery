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
GARETZ_MAX_ITER=10000
def garetz_ibu(w,a,t,sg,v,bv):
    if t<=0 or w<=0 or a<0: return 0.0
    idx=int((int(t)-1)/5)  # C truncation
    idx=min(idx,14)
    r=v/bv; GF=((sg-1)*r+1-1.05)*5+1
    prev,cur=1.0,50.0
    for _ in range(GARETZ_MAX_ITER):  # T1.15 B9: üst sınır spesifikasyonda (§2.3)
        if abs(prev-cur)<=0.05: break
        prev=cur
        cur=GT[idx]*a*w*0.749/(v*GF*(r*cur/260+1)*1.02)
    return cur
def garetz_util(sg,t): return garetz_ibu(1,1,t,sg,1,1)/74.9
POSTBOIL_MAX_STEPS=20000
def postboil(sg,t0,dur):
    # T1.15 B1: sıcaklık 0 K'ye ulaşınca (e >= 498.74 dk) f -> 0 limiti uygulanır, döngü biter;
    # adım sayısı da POSTBOIL_MAX_STEPS ile sınırlı. 498 dk altında eski sonuçla birebir aynı.
    s=0.0; t=t0
    for _ in range(POSTBOIL_MAX_STEPS):
        if not t<t0+dur: break
        kelvin=((t-t0)*-1.344+210.64+459.67)*(5/9)
        if t<5.0: f=1.0
        elif kelvin<=0.0: break
        else: f=2.39e11*math.exp(-9773/kelvin)
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

# ======================================================================================
# T1.14a: spesifikasyonda olup yukarıda referansı olmayan hesaplar (formuller/02-hop-ibu.md).
# Bölüm no'ları spesifikasyona aittir. Hop sözlüğü: amount (oz; form 3/4 için fl oz), alpha (%),
# form (0..4), use (0..4), time (dk), whirlpool_temp_f (°F). Tarif sözlüğü: volume, trub_loss,
# boil_time, boil_vol (fl oz), altitude (ft), type (0 Extract), hop_util (%), whirlpool_carry (bool),
# whirlpool_time (dk). Fermentable: name, use, type, amount_oz, yield_pct, late_extract (dk),
# points (GetOGPoints sonucu, dış modülden gelir), ibu_gal_per_lb.
# ======================================================================================
def f_to_c(f): return (f-32)*5/9
def c_to_f(c): return c*9/5+32

DEFAULT_OPTS={"first_wort_adj":10.0,"type_adj":1,"leaf_adj":0.0,"plug_adj":2.0,"pellet_adj":10.0,"mash_adj":-80.0}

# §2.1 Tinseth tek hop IBU ve ters
def tin_ibu(w,a,t,sg,v):
    if t<=0 or w<=0 or a<0 or v==0: return 0.0
    return tin_util(sg,t)*a*w*K/v
def tin_weight(ibu,v,sg,t,a):
    d=tin_util(sg,t)*a*K
    return 0.0 if d==0 else ibu*v/d
# §2.2 Rager
def rager_ibu(w,a,t,sg,v):
    if t<=0 or w<=0 or a<0 or v==0: return 0.0
    return rager_tab(int(t+0.5))*a*w*K/((1+rager_ga(sg))*v)
def rager_weight(ibu,v,sg,t,a):
    d=rager_tab(int(t+0.5))*a*K
    return 0.0 if d==0 else (1+rager_ga(sg))*v*ibu/d
# §2.3 Garetz ters (ikiye bölme; V_boil = V)
def garetz_weight(ibu,a,t,sg,v):
    lo,hi,w=0.0,1e6,500.0
    for _ in range(200):
        cur=garetz_ibu(w,a,t,sg,v,v)
        if abs(cur-ibu)<=0.05: break
        if cur<ibu: lo=w
        else: hi=w
        w=(lo+hi)/2
    else:
        return 0.0  # T1.15 B10: 200 adımda hedefe girilmedi (ulaşılamaz hedef) -> 0, Tinseth/Rager terslerindeki gibi
    return w
# §3.4 whirlpool hop
def whirlpool_util(sg,t,temp_c): return tin_util(sg,t)*tempf(temp_c)
def whirlpool_ibu(w,a,t,sg,v,temp_c):
    if v==0: return 0.0
    return whirlpool_util(sg,t,temp_c)*a*w*K/v
# §3.2 BoilTemp (°F)
def boiltempF(alt_ft): return c_to_f(boiltempC(alt_ft))
# §1 denklem çözümü: 0..2 dışı -> global ayar
def resolve_equation(eq,global_eq): return eq if 0<=eq<=2 else global_eq
# §3.3 HopUtil(SG,t,denklem,whirlpool_mu,T_F)
def hop_util(sg,t,eq,wp,temp_f):
    tc=f_to_c(temp_f)
    if wp: u=tin_util(sg,t)
    elif eq==1: u=rager_util(sg,t)
    elif eq==2: u=garetz_util(sg,t)
    else: u=tin_util(sg,t)
    return u*tempf(tc)
# §4 HopAdjustments
def hop_adjust(form,use,util_pct,extract_recipe,opts):
    if form==4: return 1.0
    if use==1: return 0.0
    m=1.0
    if use==3 and not extract_recipe: m*=(100+opts["first_wort_adj"])/100
    if opts["type_adj"]!=0:
        if form==2: m*=(100+opts["leaf_adj"])/100
        elif form==1: m*=(100+opts["plug_adj"])/100
        elif form==0: m*=(100+opts["pellet_adj"])/100
    if use==2 and not extract_recipe: m*=(100+opts["mash_adj"])/100
    return util_pct/100*m
# §5.1 GetLateExtractSGs
def _late(g): return g["type"] in (1,2,4,5,6,7) and g["late_extract"]>0
def _sg_pts(p,v,r): return 1.0 if v<=0 else 1+p*r/(v/128)/1000
def late_extract_sgs(recipe,ferms,v_boil=0.0):
    vb=max(v_boil if v_boil>0 else recipe["boil_vol"],0.1)
    v=recipe["volume"]; r=(v+recipe["trub_loss"])/vb
    p=sum(g["points"] for g in ferms if g["use"] in (0,1,2,8) and not _late(g))
    prof=[[_sg_pts(p,v,r),recipe["boil_time"]]]
    for g in sorted([g for g in ferms if _late(g)],key=lambda g:(-g["late_extract"],g["name"])):
        p+=g["amount_oz"]/16*(g["yield_pct"]*46/100)
        t=min(recipe["boil_time"],g["late_extract"])
        if abs(prof[-1][1]-t)<0.1: prof[-1][0]=_sg_pts(p,v,r)
        else: prof.append([_sg_pts(p,v,r),t])
    p+=sum(g["points"] for g in ferms if g["use"]==3 and not _late(g))
    prof.append([1.0 if p<=0 else _sg_pts(p,v,r),0.0])
    return prof
# §5.2 HopTotalUtil
def hop_total_util(hop,recipe,prof,eq):
    if hop["form"]==4: return 1.0
    if hop["amount"]<=0 or hop["use"]==1: return 0.0
    th=hop["time"]
    if hop["use"]==4:
        return tin_util(prof[-1][0],th)*tempf(f_to_c(hop["whirlpool_temp_f"]))
    tbf=boiltempF(recipe["altitude"]); segs=[]
    if prof[0][1]<th: segs.append(hop_util(prof[0][0],th,eq,False,tbf)-hop_util(prof[0][0],prof[0][1],eq,False,tbf))
    i=0
    while prof[i][1]!=0:
        if prof[i+1][1]<=th:
            st=min(prof[i][1],th)
            segs.append(hop_util(prof[i][0],st,eq,False,tbf)-hop_util(prof[i][0],prof[i+1][1],eq,False,tbf))
        i+=1
    if recipe["whirlpool_carry"]: segs.append(postboil(prof[-1][0],th,recipe["whirlpool_time"]))
    return combine(segs)
# §5.3 NewHopIBU
def new_hop_ibu(hop,recipe,prof,eq,opts):
    vibu=recipe["volume"]+recipe["trub_loss"]
    if vibu<=0: vibu=0.01
    u=hop_total_util(hop,recipe,prof,eq)
    adj=hop_adjust(hop["form"],hop["use"],recipe["hop_util"],recipe["type"]==0,opts)
    return adj*hop["amount"]*u*hop["alpha"]*100*128/(vibu*1.34)
# §5.4 hopped extract ve tarif toplamı
def hopped_extract_ibu(amount_oz,ibu_gal_per_lb,volume,trub):
    return (amount_oz/16*ibu_gal_per_lb)/max((volume+trub)/128,0.0001)
def _ext_total(ferms,recipe):
    e=0.0
    for g in ferms:
        if g["type"]==1 and g["ibu_gal_per_lb"]>0:
            e+=hopped_extract_ibu(g["amount_oz"],g["ibu_gal_per_lb"],recipe["volume"],recipe["trub_loss"])
    return e
def recipe_ibu(hops,recipe,ferms,eq,opts,v_boil=0.0):
    prof=late_extract_sgs(recipe,ferms,v_boil)
    tot=0.0
    for h in hops: tot+=new_hop_ibu(h,recipe,prof,eq,opts)
    return tot+_ext_total(ferms,recipe)
def hop_percent(w,tot_hops): return 0.0 if tot_hops<=0 else w*100/tot_hops
# §6.1 ScaleBitterness (yeni miktarlar)
def scale_bitterness(cur_ibu,target,hops,recipe,ferms,eq,opts,v_boil=0.0):
    amounts=[h["amount"] for h in hops]
    if abs(target-cur_ibu)<1e-9: return {"status":"unchanged","amounts":amounts}
    if abs(cur_ibu)<1e-9 or abs(target)<1e-9: return {"status":"failed","amounts":amounts}
    rc=dict(recipe)
    if rc["volume"]==0: rc["volume"]=0.0001
    prof=late_extract_sgs(rc,ferms,v_boil)
    ibus=[]
    for h in hops:
        if h["use"]==1 and h["form"]!=4:  # T1.15 B2: form 4 dry hop dahil (§6.1 adım 3)
            ibus.append(0.0); continue
        i=new_hop_ibu(h,rc,prof,eq,opts)
        ibus.append(0.0 if i<1e-9 else i)
    e=_ext_total(ferms,rc)
    r2=cur_ibu-e if e<cur_ibu else cur_ibu
    t2=max(target-e,0.0)
    for k,h in enumerate(hops):
        if (h["use"]==1 and h["form"]!=4) or ibus[k]==0: continue
        u=hop_total_util(h,rc,prof,eq)
        if h["alpha"]*u<=0: continue
        adj=hop_adjust(h["form"],h["use"],rc["hop_util"],rc["type"]==0,opts)
        amounts[k]=(ibus[k]/adj)*(t2/r2)*(rc["volume"]+rc["trub_loss"])*1.34/(h["alpha"]*u*100*128)
    return {"status":"ok","amounts":amounts}
# §6.2 HopIBUAdjust
def hop_ibu_adjust(w_old,ibu_old,ibu_new): return 0.0 if ibu_old<=0 else w_old/ibu_old*ibu_new
# §7 Hop Bitterness aracı (CalcIBU)
def calc_ibu(hop,sg,v_floz,eq,util_pct,opts):
    w,a,t=hop["amount"],hop["alpha"],hop["time"]
    if hop["form"]==4: return 0.0 if v_floz<=0 else w*a*100*128/(v_floz*1.34)
    if w<=0 or hop["use"]==1 or v_floz<=0: return 0.0
    vg=v_floz/128
    if hop["use"]==4: ibu=whirlpool_ibu(w,a,t,sg,vg,f_to_c(hop["whirlpool_temp_f"]))
    elif eq==1: ibu=rager_ibu(w,a,t,sg,vg)
    elif eq==2: ibu=garetz_ibu(w,a,t,sg,vg,vg)
    else: ibu=tin_ibu(w,a,t,sg,vg)
    return ibu*hop_adjust(hop["form"],hop["use"],util_pct,False,opts)
def calc_aau(w,a,use,t): return w*a if use in (0,3) and t>=15 else 0.0
def tool_bitterness(hops,sg,v_floz,eq,util_pct,opts):
    c=[calc_ibu(h,sg,v_floz,eq,util_pct,opts) for h in hops]
    return {"ibu":sum(c),"aau":sum(calc_aau(h["amount"],h["alpha"],h["use"],h["time"]) for h in hops),"contribs":c}
# §8 hop oranları (g/L)
def tot_hops_phase(hops,phase):
    return sum(h["amount"] for h in hops if h["use"]==phase or (phase==0 and h["use"]==3))
def hop_rate(hops,volume_floz,phase):
    if volume_floz==0: return 0.0
    return tot_hops_phase(hops,phase)*28.349523125/(volume_floz*29.57352956/1000)
# §9 TotWhirlpoolTime
def tot_whirlpool_time(recipe_time,hops,misc):
    m=recipe_time
    for h in hops:
        if h["use"]==4: m=max(m,int(h["time"]+0.48))
    for x in misc:
        if x["use"]==6: m=max(m,int(x["time"]+0.48))
    return m
# §10 HopAlphaFunc (ay ve depolama tipi ile)
def storage_factor(storage): return {0:1.0,1:0.75,2:0.5}.get(storage,1.0)
def aged_alpha_months(a,hsi,tF,months,storage): return hop_alpha(a,hsi,tF,months*30,storage_factor(storage))

_o=dict(DEFAULT_OPTS)
_rec={"volume":640.0,"trub_loss":0.0,"boil_time":60,"boil_vol":768.0,"altitude":0,"type":2,"hop_util":100.0,"whirlpool_carry":False,"whirlpool_time":0}
_h={"amount":1.0,"alpha":5.0,"form":0,"use":0,"time":60,"whirlpool_temp_f":176.0}
_p=[[1.05,60.0],[1.05,0.0]]
print("T1.14a recipe pellet IBU (spec #12 ~18.9351): %.4f"%new_hop_ibu(_h,_rec,_p,0,_o))
print("T1.14a tinseth weight for 17.2137 IBU: %.4f oz"%tin_weight(17.2137,5.0,1.05,60,5.0))

//! Section 05 parity (mash, volumes) against `vektorler/05.json`.
mod ortak;

use engine::mash::{self, SpargeParams, Tun};
use ortak::Case;
use serde_json::{Value, json};

fn opt(c: &Case, name: &str) -> Option<f64> {
    c.girdi.get(name).and_then(Value::as_f64)
}

fn or0(c: &Case, name: &str) -> f64 {
    opt(c, name).unwrap_or(0.0)
}

fn tun(c: &Case) -> Tun {
    Tun {
        mass_kg: or0(c, "m_tun"),
        specific_heat: or0(c, "c_tun"),
        volume_l: or0(c, "v_tun"),
    }
}

fn sparge_params(c: &Case) -> SpargeParams {
    SpargeParams {
        sparge_vol: c.f("S"),
        absorb: c.f("absorb"),
        grain_kg: c.f("W"),
        tun_vol: c.f("tun_vol"),
        fill_pct: c.f("pct"),
        mash_water: c.f("mash_water"),
        even: c.b("even"),
        drain: c.b("drain"),
    }
}

fn opt_json(v: Option<f64>) -> Value {
    v.map_or(Value::Null, |x| json!(x))
}

#[test]
fn parite_05() {
    ortak::run("05", |c| {
        match c.fonk.as_str() {
            "constant" => {
                let v = match c.v("name").as_str().unwrap() {
                    "FLOZ_L" => mash::FLOZ_L,
                    "LB_KG" => mash::LB_KG,
                    "OZ_KG" => mash::OZ_KG,
                    "GRAIN_VOL" => mash::GRAIN_VOL,
                    "C_MALT" => mash::C_MALT,
                    "ABS_STD" => mash::ABS_STD,
                    "ABS_BIAB" => mash::ABS_BIAB,
                    _ => return false,
                };
                c.check_f(v);
            }
            "tun_eff" => c.check_f(mash::tun_eff(c.f("m_tun"), c.f("v_mash"), c.f("v_tun"))),
            "strike_temp" => c.check(&opt_json(mash::strike_temp(
                c.f("vw"),
                c.f("t_step"),
                c.f("w"),
                c.f("t_grain"),
                tun(c),
                opt(c, "t_tun"),
            ))),
            "infusion_temp" => c.check(&opt_json(mash::infusion_temp(
                c.f("va"),
                c.f("t2"),
                c.f("vw"),
                c.f("t1"),
                c.f("w"),
                tun(c),
            ))),
            "infusion_amount" => c.check_f(mash::infusion_amount(
                c.f("tinf"),
                c.f("t2"),
                c.f("vw"),
                c.f("t1"),
                c.f("w"),
                tun(c),
            )),
            "decoct_vol" => c.check_f(mash::decoct_vol(
                c.f("t2"),
                c.f("vw"),
                c.f("t1"),
                c.f("w"),
                tun(c),
                opt(c, "tb").unwrap_or(100.0),
            )),
            "boil_temp_c" => c.check_f(mash::boil_temp_c(c.f("alt_ft"))),
            "equip_boil_vol" => c.check_f(mash::equip_boil_vol(
                c.f("batch"),
                c.f("topup"),
                c.f("trub"),
                c.f("cool_pct"),
                c.f("boil_off_total"),
            )),
            "batch_sparge" => {
                let i = u32::try_from(c.i("i")).unwrap();
                let (v, flag) = mash::batch_sparge(i, &sparge_params(c));
                c.check(&json!([v, flag]));
            }
            "sparge_plan" => {
                let steps: Vec<Value> = mash::sparge_plan(&sparge_params(c))
                    .iter()
                    .map(|s| json!([s.index, s.volume, s.action.label()]))
                    .collect();
                c.check(&Value::Array(steps));
            }
            "parti" => {
                let p = mash::parti(c.f("og"), c.f("vol"), c.f("f1"));
                c.check(&json!({"vol1": p.vol1, "vol2": p.vol2, "og1": p.og1, "og2": p.og2}));
            }
            _ => return false,
        }
        true
    });
}

//! Sample library data (T2.19). The rows are only for development and tests.
//!
//! Every row is marked `SAMPLE DATA` (in `notes` or `license_note`). The source of the real
//! library data is a separate decision. Ids come from the names, so a second run adds nothing.
//! The rows are shared (no workspace) and only `app_owner` can write them.

use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

const MARK: &str = "SAMPLE DATA";

type Tx = sqlx::Transaction<'static, sqlx::Postgres>;

/// Writes the sample ingredients, style guides, styles and style mappings. Safe to repeat.
///
/// # Errors
/// Returns the database error.
#[allow(clippy::too_many_lines)] // a flat list of sample rows
pub async fn seed_sample_library(pool: &PgPool) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL ROLE app_owner")
        .execute(&mut *tx)
        .await?;

    for (name, grain, yield_pct, color) in [
        ("Sample Pale Malt", "grain", 80.5_f64, 2.5_f64),
        ("Sample Munich Malt", "grain", 79.0, 9.0),
        ("Sample Crystal 60", "grain", 74.0, 60.0),
        ("Sample Cane Sugar", "sugar", 100.0, 0.0),
    ] {
        let id = ingredient(&mut tx, "fermentable", name).await?;
        sqlx::query(
            "INSERT INTO ingredient_fermentables (id, workspace_id, grain_type, yield_pct, color_srm) \
             VALUES ($1, NULL, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(grain)
        .bind(yield_pct)
        .bind(color)
        .execute(&mut *tx)
        .await?;
    }
    for (name, alpha) in [
        ("Sample Cascade", 5.5_f64),
        ("Sample Citra", 12.0),
        ("Sample Fuggle", 4.5),
    ] {
        let id = ingredient(&mut tx, "hop", name).await?;
        sqlx::query(
            "INSERT INTO ingredient_hops (id, workspace_id, form, alpha_pct) \
             VALUES ($1, NULL, 'pellet', $2) ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(alpha)
        .execute(&mut *tx)
        .await?;
    }
    for (name, form, lo, hi) in [
        ("Sample Dry Ale Yeast", "dry", 73.0_f64, 80.0_f64),
        ("Sample Liquid Ale Yeast", "liquid", 73.0, 85.0),
    ] {
        let id = ingredient(&mut tx, "yeast", name).await?;
        sqlx::query(
            "INSERT INTO ingredient_yeasts (id, workspace_id, form, min_attenuation_pct, \
                 max_attenuation_pct) \
             VALUES ($1, NULL, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(form)
        .bind(lo)
        .bind(hi)
        .execute(&mut *tx)
        .await?;
    }
    for (name, kind, usage) in [
        ("Sample Irish Moss", "fining", "boil"),
        ("Sample Orange Peel", "spice", "boil"),
    ] {
        let id = ingredient(&mut tx, "misc", name).await?;
        sqlx::query(
            "INSERT INTO ingredient_miscs (id, workspace_id, misc_kind, default_usage) \
             VALUES ($1, NULL, $2, $3) ON CONFLICT (id) DO NOTHING",
        )
        .bind(id)
        .bind(kind)
        .bind(usage)
        .execute(&mut *tx)
        .await?;
    }

    let guide_a = guide(&mut tx, "Sample Guide A", 2021).await?;
    let guide_b = guide(&mut tx, "Sample Guide B", 2015).await?;
    let a_pale = style(
        &mut tx,
        guide_a,
        "A1",
        "Sample Pale Ale",
        (1.045, 1.060),
        (30.0, 50.0),
    )
    .await?;
    let a_stout = style(
        &mut tx,
        guide_a,
        "A2",
        "Sample Stout",
        (1.038, 1.056),
        (25.0, 40.0),
    )
    .await?;
    let b_pale = style(
        &mut tx,
        guide_b,
        "B1",
        "Sample American Ale",
        (1.044, 1.056),
        (30.0, 45.0),
    )
    .await?;
    let b_stout = style(
        &mut tx,
        guide_b,
        "B2",
        "Sample Dry Stout",
        (1.036, 1.050),
        (30.0, 45.0),
    )
    .await?;
    mapping(&mut tx, a_pale, b_pale, "closest").await?;
    mapping(&mut tx, a_stout, b_stout, "equivalent").await?;
    mapping(&mut tx, b_pale, a_pale, "closest").await?;

    tx.commit().await?;
    Ok(())
}

async fn derived_id(tx: &mut Tx, seed: &str) -> Result<Uuid> {
    Ok(sqlx::query_scalar("SELECT md5($1)::uuid")
        .bind(seed)
        .fetch_one(&mut **tx)
        .await?)
}

async fn ingredient(tx: &mut Tx, kind: &str, name: &str) -> Result<Uuid> {
    let id = derived_id(tx, &format!("sample-ingredient:{name}")).await?;
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, notes, source) \
         VALUES ($1, NULL, $2, $3, $4, 'library') ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(kind)
    .bind(name)
    .bind(MARK)
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

async fn guide(tx: &mut Tx, name: &str, year: i32) -> Result<Uuid> {
    let id = derived_id(tx, &format!("sample-guide:{name}")).await?;
    sqlx::query(
        "INSERT INTO style_guides (id, workspace_id, name, year, license_note) \
         VALUES ($1, NULL, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(name)
    .bind(year)
    .bind(format!("{MARK}, not real guideline data"))
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

async fn style(
    tx: &mut Tx,
    guide_id: Uuid,
    code: &str,
    name: &str,
    og: (f64, f64),
    ibu: (f64, f64),
) -> Result<Uuid> {
    let id = derived_id(tx, &format!("sample-style:{guide_id}:{code}")).await?;
    sqlx::query(
        "INSERT INTO styles (id, workspace_id, guide_id, code, name, og_min_sg, og_max_sg, \
             ibu_min, ibu_max, notes) \
         VALUES ($1, NULL, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(guide_id)
    .bind(code)
    .bind(name)
    .bind(og.0)
    .bind(og.1)
    .bind(ibu.0)
    .bind(ibu.1)
    .bind(MARK)
    .execute(&mut **tx)
    .await?;
    Ok(id)
}

async fn mapping(tx: &mut Tx, from: Uuid, to: Uuid, relation: &str) -> Result<()> {
    let id = derived_id(tx, &format!("sample-mapping:{from}:{to}")).await?;
    sqlx::query(
        "INSERT INTO style_mappings (id, workspace_id, from_style_id, to_style_id, relation, notes) \
         VALUES ($1, NULL, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING",
    )
    .bind(id)
    .bind(from)
    .bind(to)
    .bind(relation)
    .bind(MARK)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

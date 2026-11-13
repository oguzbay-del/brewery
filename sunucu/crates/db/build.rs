//! Rebuilds the crate when a migration file is added or changed (`sqlx::migrate!` embeds them).

fn main() {
    println!("cargo:rerun-if-changed=../../migrations");
}

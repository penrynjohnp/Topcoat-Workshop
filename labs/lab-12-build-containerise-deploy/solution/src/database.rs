use std::path::Path;

use crate::models::Berth;

pub const DEFAULT_DATABASE_URL: &str = "sqlite:./slipway.db";

// ANCHOR: database-setup
pub async fn connect(url: &str) -> toasty::Result<toasty::Db> {
    let fresh = url == "sqlite::memory:" || sqlite_path(url).is_some_and(|path| !path.exists());
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(url)
        .await?;

    if fresh {
        db.push_schema().await?;
        seed(&mut db).await?;
    }

    Ok(db)
}

pub async fn test_database() -> toasty::Result<toasty::Db> {
    connect("sqlite::memory:").await
}

pub async fn ensure_seeded(db: &mut toasty::Db) -> toasty::Result<()> {
    if Berth::all().exec(db).await?.is_empty() {
        seed(db).await?;
    }
    Ok(())
}

fn sqlite_path(url: &str) -> Option<&Path> {
    url.strip_prefix("sqlite:")
        .filter(|path| *path != ":memory:")
        .map(Path::new)
}
// ANCHOR_END: database-setup

// ANCHOR: database-seed
async fn seed(db: &mut toasty::Db) -> toasty::Result<()> {
    let a1 = create_berth(db, "a1", "A1", 8, "occupied", true).await?;
    let _a2 = create_berth(db, "a2", "A2", 8, "vacant", true).await?;
    let b7 = create_berth(db, "b7", "B7", 12, "maintenance", true).await?;
    let c3 = create_berth(db, "c3", "C3", 15, "occupied", true).await?;
    let visitors = create_berth(db, "visitors", "Visitors", 20, "vacant", false).await?;
    let yard = create_berth(db, "yard", "Yard", 30, "maintenance", false).await?;

    toasty::create!(in a1.vessels() { name: "Lady Jane" })
        .exec(db)
        .await?;
    toasty::create!(in c3.vessels() { name: "Sea Urchin" })
        .exec(db)
        .await?;
    toasty::create!(in visitors.vessels() { name: "Morning Star" })
        .exec(db)
        .await?;
    toasty::create!(in yard.vessels() { name: "Blue Moon" })
        .exec(db)
        .await?;

    toasty::create!(in a1.work_orders() {
        title: "Anti-foul A1",
        completed: false,
    })
    .exec(db)
    .await?;
    toasty::create!(in c3.work_orders() {
        title: "Replace C3 mooring line",
        completed: false,
    })
    .exec(db)
    .await?;

    let _ = b7;
    Ok(())
}

async fn create_berth(
    db: &mut toasty::Db,
    slug: &str,
    name: &str,
    length_m: i64,
    status: &str,
    managed: bool,
) -> toasty::Result<Berth> {
    toasty::create!(Berth {
        slug,
        name,
        length_m,
        status,
        managed
    })
    .exec(db)
    .await
}
// ANCHOR_END: database-seed

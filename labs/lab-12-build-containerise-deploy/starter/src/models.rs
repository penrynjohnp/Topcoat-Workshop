// ANCHOR: toasty-models
#[derive(Debug, toasty::Model)]
pub struct Berth {
    #[key]
    pub slug: String,
    pub name: String,
    pub length_m: i64,
    pub status: String,
    #[index]
    pub managed: bool,
    #[has_many]
    pub vessels: toasty::Deferred<Vec<Vessel>>,
    #[has_many]
    pub work_orders: toasty::Deferred<Vec<WorkOrder>>,
}

#[derive(Debug, toasty::Model)]
pub struct Vessel {
    #[key]
    #[auto]
    pub id: u64,
    pub name: String,
    #[index]
    pub berth_slug: String,
    #[belongs_to(key = berth_slug, references = slug)]
    pub berth: toasty::Deferred<Berth>,
}

#[derive(Debug, toasty::Model)]
pub struct WorkOrder {
    #[key]
    #[auto]
    pub id: u64,
    pub title: String,
    pub completed: bool,
    #[index]
    pub berth_slug: String,
    #[belongs_to(key = berth_slug, references = slug)]
    pub berth: toasty::Deferred<Berth>,
}
// ANCHOR_END: toasty-models

// ANCHOR: persisted-auth-models
#[derive(Debug, toasty::Model)]
pub struct SessionRecord {
    #[key]
    pub token_hash: String,
    pub email: String,
    pub expires_at_unix: i64,
}

#[derive(Debug, toasty::Model)]
pub struct MagicLinkRecord {
    #[key]
    pub token: String,
    pub email: String,
    pub expires_at_unix: i64,
}
// ANCHOR_END: persisted-auth-models

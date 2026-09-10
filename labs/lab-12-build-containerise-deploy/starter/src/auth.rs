use std::sync::{Arc, atomic::AtomicUsize};

use crate::{
    database,
    models::{MagicLinkRecord, SessionRecord},
};
use topcoat::{
    context::{Cx, app_context},
    cookie::{Cookie, Cookies, Key, SameSite, cookies},
    session,
};

// ANCHOR: database-app-state
#[derive(Clone)]
pub struct AppState {
    pub runtime_script: bool,
    pub load_count: Arc<AtomicUsize>,
    pub db: toasty::Db,
    pub cookie_key: Arc<Key>,
}

impl AppState {
    pub fn from_db(db: toasty::Db) -> Self {
        Self::from_db_with_cookie_key(db, Arc::new(Key::generate()))
    }

    pub fn from_db_with_cookie_key(db: toasty::Db, cookie_key: Arc<Key>) -> Self {
        Self {
            runtime_script: false,
            load_count: Arc::new(AtomicUsize::new(0)),
            db,
            cookie_key,
        }
    }

    pub async fn connect(url: &str) -> topcoat::Result<Self> {
        Ok(Self::from_db(database::connect(url).await?))
    }

    pub async fn test() -> topcoat::Result<Self> {
        Ok(Self::from_db(database::test_database().await?))
    }
}
// ANCHOR_END: database-app-state

pub fn secure_cookies(cx: &Cx) -> impl Cookies {
    cookies(cx)
        .private(&app_context::<AppState>(cx).cookie_key)
        .signed(&app_context::<AppState>(cx).cookie_key)
        .default_secure(false)
        .default_http_only(true)
        .default_same_site(SameSite::Lax)
        .default_path("/")
}

fn hash_key(hash: &session::TokenHash) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unix_time(time: web_time::SystemTime) -> i64 {
    time.duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn now_unix() -> i64 {
    unix_time(web_time::SystemTime::now())
}

// ANCHOR: database-session-store
async fn record_session(
    state: &AppState,
    session: session::Session,
    email: String,
) -> topcoat::Result<()> {
    let mut db = state.db.clone();
    toasty::create!(SessionRecord {
        token_hash: hash_key(&session.token_hash),
        email,
        expires_at_unix: unix_time(session.expires_at),
    })
    .exec(&mut db)
    .await?;
    Ok(())
}

async fn remove_session(state: &AppState, hash: &session::TokenHash) -> topcoat::Result<()> {
    let mut db = state.db.clone();
    SessionRecord::filter_by_token_hash(hash_key(hash))
        .delete()
        .exec(&mut db)
        .await?;
    Ok(())
}

async fn current_email_for_hash(
    state: &AppState,
    hash: &session::TokenHash,
) -> topcoat::Result<Option<String>> {
    let mut db = state.db.clone();
    let Some(record) = SessionRecord::filter_by_token_hash(hash_key(hash))
        .first()
        .exec(&mut db)
        .await?
    else {
        return Ok(None);
    };
    if record.expires_at_unix <= now_unix() {
        SessionRecord::filter_by_token_hash(record.token_hash)
            .delete()
            .exec(&mut db)
            .await?;
        return Ok(None);
    }
    Ok(Some(record.email))
}

async fn update_expiry(
    state: &AppState,
    hash: session::TokenHash,
    expires_at: web_time::SystemTime,
) -> topcoat::Result<()> {
    let mut db = state.db.clone();
    let Some(mut record) = SessionRecord::filter_by_token_hash(hash_key(&hash))
        .first()
        .exec(&mut db)
        .await?
    else {
        return Ok(());
    };
    toasty::update!(record {
        expires_at_unix: unix_time(expires_at)
    })
    .exec(&mut db)
    .await?;
    Ok(())
}
// ANCHOR_END: database-session-store

pub async fn current_user(cx: &Cx) -> topcoat::Result<Option<String>> {
    let Some(hash) = session::token_hash(cx).await? else {
        return Ok(None);
    };
    current_email_for_hash(app_context::<AppState>(cx), &hash).await
}

// ANCHOR: require-auth
pub async fn require_auth(cx: &Cx) -> topcoat::Result<String> {
    match current_user(cx).await? {
        Some(email) => Ok(email),
        None => Err(topcoat::router::error::redirect("/login").into()),
    }
}
// ANCHOR_END: require-auth

pub async fn begin_session(cx: &Cx, email: String) -> topcoat::Result<()> {
    let new_session = session::start(cx).await?;
    record_session(app_context::<AppState>(cx), new_session, email).await
}

pub async fn end_session(cx: &Cx) -> topcoat::Result<()> {
    if let Some(hash) = session::stop(cx).await? {
        remove_session(app_context::<AppState>(cx), &hash).await?;
    }
    Ok(())
}

pub async fn refresh_session(cx: &Cx) -> topcoat::Result<()> {
    if let Some(refreshed) = session::refresh(cx).await? {
        update_expiry(
            app_context::<AppState>(cx),
            refreshed.token_hash,
            refreshed.expires_at,
        )
        .await?;
    }
    Ok(())
}

pub async fn rotate_session(cx: &Cx) -> topcoat::Result<()> {
    let state = app_context::<AppState>(cx);
    if let Some(rotation) = session::rotate(cx).await? {
        let email = current_email_for_hash(state, &rotation.revoked)
            .await?
            .unwrap_or_default();
        remove_session(state, &rotation.revoked).await?;
        record_session(state, rotation.session, email).await?;
    }
    Ok(())
}

// ANCHOR: database-magic-links
pub async fn new_magic_link(state: &AppState, email: String) -> topcoat::Result<String> {
    let token = session::Token::random().encode();
    let mut db = state.db.clone();
    toasty::create!(MagicLinkRecord {
        token: token.as_str(),
        email,
        expires_at_unix: now_unix() + 900,
    })
    .exec(&mut db)
    .await?;
    Ok(token)
}

pub async fn consume_magic_link(
    state: &AppState,
    token: &str,
) -> topcoat::Result<Option<MagicLinkRecord>> {
    let mut db = state.db.clone();
    let Some(link) = MagicLinkRecord::filter_by_token(token)
        .first()
        .exec(&mut db)
        .await?
    else {
        return Ok(None);
    };
    MagicLinkRecord::filter_by_token(token)
        .delete()
        .exec(&mut db)
        .await?;
    Ok((link.expires_at_unix > now_unix()).then_some(link))
}
// ANCHOR_END: database-magic-links

pub fn remember_email(cx: &Cx, email: &str) {
    secure_cookies(cx).add(
        Cookie::build(("last_email", email.to_owned()))
            .path("/")
            .build(),
    );
}

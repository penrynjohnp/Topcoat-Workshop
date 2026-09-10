use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use topcoat::{
    context::{Cx, app_context, memoize},
    cookie::{Cookie, Cookies, Key, SameSite, cookies},
    session,
};

// ANCHOR: session-store
#[derive(Clone)]
pub struct AppState {
    pub runtime_script: bool,
    pub load_count: Arc<AtomicUsize>,
    pub sessions: Arc<Mutex<HashMap<session::TokenHash, SessionRecord>>>,
    pub magic_links: Arc<Mutex<HashMap<String, MagicLink>>>,
    pub completed_work_orders: Arc<Mutex<HashSet<String>>>,
    pub next_magic_link: Arc<AtomicUsize>,
    pub cookie_key: Arc<Key>,
}

#[derive(Clone)]
pub struct SessionRecord {
    pub email: String,
    pub expires_at: web_time::SystemTime,
}

#[derive(Clone)]
pub struct MagicLink {
    pub email: String,
    pub expires_at: web_time::SystemTime,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            runtime_script: false,
            load_count: Arc::new(AtomicUsize::new(0)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            magic_links: Arc::new(Mutex::new(HashMap::new())),
            completed_work_orders: Arc::new(Mutex::new(HashSet::new())),
            next_magic_link: Arc::new(AtomicUsize::new(1)),
            cookie_key: Arc::new(Key::generate()),
        }
    }

    pub fn record_session(&self, session: session::Session, email: String) {
        self.sessions.lock().unwrap().insert(
            session.token_hash,
            SessionRecord {
                email,
                expires_at: session.expires_at,
            },
        );
    }

    pub fn remove_session(&self, hash: &session::TokenHash) {
        self.sessions.lock().unwrap().remove(hash);
    }

    pub fn complete_work_order(&self, id: &str) -> bool {
        self.completed_work_orders
            .lock()
            .unwrap()
            .insert(id.to_owned())
    }

    pub fn work_order_is_complete(&self, id: &str) -> bool {
        self.completed_work_orders.lock().unwrap().contains(id)
    }

    pub fn current_email(&self, hash: &session::TokenHash) -> Option<String> {
        let mut sessions = self.sessions.lock().unwrap();
        let record = sessions.get(hash)?;
        if record.expires_at <= web_time::SystemTime::now() {
            sessions.remove(hash);
            return None;
        }
        Some(record.email.clone())
    }

    pub fn update_expiry(&self, hash: session::TokenHash, expires_at: web_time::SystemTime) {
        if let Some(record) = self.sessions.lock().unwrap().get_mut(&hash) {
            record.expires_at = expires_at;
        }
    }

    pub fn new_magic_link(&self, email: String) -> String {
        let id = self.next_magic_link.fetch_add(1, Ordering::Relaxed);
        let token = format!("magic-{id}");
        self.magic_links.lock().unwrap().insert(
            token.clone(),
            MagicLink {
                email,
                expires_at: web_time::SystemTime::now() + std::time::Duration::from_secs(900),
            },
        );
        token
    }

    pub fn consume_magic_link(&self, token: &str) -> Option<MagicLink> {
        let link = self.magic_links.lock().unwrap().remove(token)?;
        (link.expires_at > web_time::SystemTime::now()).then_some(link)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
// ANCHOR_END: session-store

pub fn secure_cookies(cx: &Cx) -> impl Cookies {
    cookies(cx)
        .private(&app_context::<AppState>(cx).cookie_key)
        .signed(&app_context::<AppState>(cx).cookie_key)
        .default_secure(false)
        .default_http_only(true)
        .default_same_site(SameSite::Lax)
        .default_path("/")
}

#[memoize(as_ref)]
pub async fn current_user(cx: &Cx) -> Option<String> {
    let hash = session::token_hash(cx).await.ok().flatten()?;
    app_context::<AppState>(cx).current_email(&hash)
}

// ANCHOR: require-auth
pub async fn require_auth(cx: &Cx) -> topcoat::Result<String> {
    match current_user(cx).await {
        Some(email) => Ok(email.clone()),
        None => Err(topcoat::router::error::redirect("/login").into()),
    }
}
// ANCHOR_END: require-auth

pub async fn begin_session(cx: &Cx, email: String) -> topcoat::Result<()> {
    let state = app_context::<AppState>(cx);
    let new_session = session::start(cx).await?;
    state.record_session(new_session, email);
    Ok(())
}

pub async fn end_session(cx: &Cx) -> topcoat::Result<()> {
    let state = app_context::<AppState>(cx);
    if let Some(hash) = session::stop(cx).await? {
        state.remove_session(&hash);
    }
    Ok(())
}

pub async fn refresh_session(cx: &Cx) -> topcoat::Result<()> {
    let state = app_context::<AppState>(cx);
    if let Some(refreshed) = session::refresh(cx).await? {
        state.update_expiry(refreshed.token_hash, refreshed.expires_at);
    }
    Ok(())
}

pub async fn rotate_session(cx: &Cx) -> topcoat::Result<()> {
    let state = app_context::<AppState>(cx);
    if let Some(rotation) = session::rotate(cx).await? {
        let email = state.current_email(&rotation.revoked).unwrap_or_default();
        state.remove_session(&rotation.revoked);
        state.record_session(rotation.session, email);
    }
    Ok(())
}

pub fn remember_email(cx: &Cx, email: &str) {
    secure_cookies(cx).add(
        Cookie::build(("last_email", email.to_owned()))
            .path("/")
            .build(),
    );
}

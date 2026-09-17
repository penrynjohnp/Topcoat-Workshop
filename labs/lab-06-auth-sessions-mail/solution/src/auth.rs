use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use topcoat::{
    context::{Cx, app_context, memoize},
    cookie::{Cookie, Cookies, Key, SameSite, cookies, time::Duration as CookieDuration},
    router::{
        Method,
        request::{method, uri},
    },
    session,
};

// ANCHOR: session-store
#[derive(Clone)]
pub struct AppState {
    pub load_count: Arc<AtomicUsize>,
    pub sessions: Arc<Mutex<HashMap<session::TokenHash, SessionRecord>>>,
    pub magic_links: Arc<Mutex<HashMap<String, MagicLink>>>,
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
            load_count: Arc::new(AtomicUsize::new(0)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            magic_links: Arc::new(Mutex::new(HashMap::new())),
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

// ANCHOR: cookie-access
pub fn secure_cookies(cx: &Cx) -> impl Cookies {
    cookies(cx)
        .private(&app_context::<AppState>(cx).cookie_key)
        .signed(&app_context::<AppState>(cx).cookie_key)
        .default_secure(false)
        .default_http_only(true)
        .default_same_site(SameSite::Lax)
        .default_path("/")
}
// ANCHOR_END: cookie-access

const RETURN_TO_COOKIE: &str = "return_to";
const DEFAULT_RETURN_TO: &str = "/admin";

fn remember_return_to(cx: &Cx) {
    if method(cx) != Method::GET && method(cx) != Method::HEAD {
        return;
    }

    let return_to = uri(cx).path_and_query().map_or("/", |value| value.as_str());
    secure_cookies(cx).add(
        Cookie::build((RETURN_TO_COOKIE, return_to.to_owned()))
            .path("/")
            .max_age(CookieDuration::minutes(10))
            .build(),
    );
}

fn validated_return_to(return_to: Option<String>) -> String {
    return_to
        .filter(|value| {
            value.starts_with('/')
                && !value.starts_with("//")
                && !value.contains('\\')
                && !value.contains("..")
        })
        .unwrap_or_else(|| DEFAULT_RETURN_TO.to_owned())
}

pub fn take_return_to(cx: &Cx) -> String {
    let jar = secure_cookies(cx);
    let return_to = jar
        .get(RETURN_TO_COOKIE)
        .map(|cookie| cookie.value().to_owned());
    if return_to.is_some() {
        jar.remove(Cookie::build((RETURN_TO_COOKIE, "")).path("/").build());
    }

    validated_return_to(return_to)
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
        None => {
            remember_return_to(cx);
            Err(topcoat::router::error::redirect("/login").into())
        }
    }
}
// ANCHOR_END: require-auth

// ANCHOR: session-lifecycle
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
// ANCHOR_END: session-lifecycle

// ANCHOR: cookie-write
pub fn remember_email(cx: &Cx, email: &str) {
    secure_cookies(cx).add(
        Cookie::build(("last_email", email.to_owned()))
            .path("/")
            .build(),
    );
}
// ANCHOR_END: cookie-write

#[cfg(test)]
mod tests {
    use super::{DEFAULT_RETURN_TO, validated_return_to};

    #[test]
    fn return_to_accepts_a_local_path() {
        assert_eq!(
            validated_return_to(Some("/dashboard?tab=open".to_owned())),
            "/dashboard?tab=open"
        );
    }

    #[test]
    fn return_to_rejects_unsafe_destinations() {
        for destination in [
            "//evil.com",
            r"/\evil.com",
            "http://evil.com",
            "https:/evil.com",
        ] {
            assert_eq!(
                validated_return_to(Some(destination.to_owned())),
                DEFAULT_RETURN_TO,
                "{destination}"
            );
        }
    }

    #[test]
    fn return_to_rejects_dot_dot() {
        assert_eq!(
            validated_return_to(Some("/dashboard/../admin".to_owned())),
            DEFAULT_RETURN_TO
        );
    }
}

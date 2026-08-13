use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::{Arc, RwLock},
};

use uuid::Uuid;

/// Stable identity for a live native window.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WindowId(Arc<str>);

impl WindowId {
    pub fn new(label: impl AsRef<str>) -> Self {
        Self(label.as_ref().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for WindowId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
    pub window_id: WindowId,
    pub token: String,
    pub start_url: String,
}

#[derive(Clone, Debug, Default)]
pub struct SessionStore {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    by_token: HashMap<String, Session>,
    by_window: HashMap<WindowId, String>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&self, window_id: WindowId, start_url: impl Into<String>) -> Session {
        self.insert(Session {
            window_id,
            token: Uuid::new_v4().simple().to_string(),
            start_url: start_url.into(),
        })
    }

    pub fn insert(&self, session: Session) -> Session {
        let mut inner = self.inner.write().expect("session store lock");
        if let Some(previous) = inner
            .by_window
            .insert(session.window_id.clone(), session.token.clone())
        {
            inner.by_token.remove(&previous);
        }
        inner
            .by_token
            .insert(session.token.clone(), session.clone());
        session
    }

    pub fn get_by_token(&self, token: &str) -> Option<Session> {
        let inner = self.inner.read().expect("session store lock");
        inner
            .by_token
            .iter()
            .find(|(stored, _)| tokens_equal(stored, token))
            .map(|(_, session)| session.clone())
    }

    pub fn get_by_window(&self, window_id: &WindowId) -> Option<Session> {
        let inner = self.inner.read().expect("session store lock");
        inner
            .by_window
            .get(window_id)
            .and_then(|token| inner.by_token.get(token).cloned())
    }

    pub fn remove_window(&self, window_id: &WindowId) -> Option<Session> {
        let mut inner = self.inner.write().expect("session store lock");
        let token = inner.by_window.remove(window_id)?;
        inner.by_token.remove(&token)
    }

    pub fn len(&self) -> usize {
        self.inner
            .read()
            .expect("session store lock")
            .by_token
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .read()
            .expect("session store lock")
            .by_token
            .is_empty()
    }
}

fn tokens_equal(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_a_window_session_revokes_the_previous_token() {
        let store = SessionStore::new();
        let first = store.create(WindowId::new("main"), "/");
        let second = store.create(WindowId::new("main"), "/");
        assert!(store.get_by_token(&first.token).is_none());
        assert_eq!(
            store.get_by_token(&second.token).unwrap().window_id,
            first.window_id
        );
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn token_lookup_rejects_a_different_token_of_the_same_length() {
        let store = SessionStore::new();
        let session = store.insert(Session {
            window_id: WindowId::new("main"),
            token: "a".repeat(32),
            start_url: "/".into(),
        });
        assert!(store.get_by_token(&"b".repeat(32)).is_none());
        assert_eq!(
            store.get_by_token(&session.token).unwrap().token,
            session.token
        );
    }
}

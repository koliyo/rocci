use std::time::{Duration, Instant};

use crate::{Opened, adapter::origin_from_url};

pub const SESSION_GRACE: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub adapter_id: String,
    pub root: String,
    pub document: Option<String>,
    pub url: String,
    pub origin: String,
    pub title: String,
    pub inspector_url: Option<String>,
    retire_at: Option<Instant>,
}

impl Session {
    pub fn from_opened(opened: &Opened) -> Self {
        Self {
            adapter_id: opened.target.adapter_id.clone(),
            root: opened.target.path.clone(),
            document: opened.document.clone(),
            url: opened.url.clone(),
            origin: origin_from_url(&opened.url),
            title: opened.title.clone(),
            inspector_url: opened.inspector_url.clone(),
            retire_at: None,
        }
    }

    pub fn root_key(&self) -> String {
        format!("{}::{}", self.adapter_id, self.root)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionTable {
    sessions: Vec<Session>,
    current: Option<usize>,
}

impl SessionTable {
    pub fn record(&mut self, session: Session) -> usize {
        let key = session.root_key();
        let index = if let Some(index) = self
            .sessions
            .iter()
            .position(|existing| existing.root_key() == key)
        {
            let mut next = session;
            next.retire_at = None;
            self.sessions[index] = next;
            index
        } else {
            self.sessions.push(session);
            self.sessions.len() - 1
        };
        self.current = Some(index);
        self.retire_others(index);
        index
    }

    pub fn reusable(&self, adapter_id: &str, root: &str) -> Option<&Session> {
        let key = format!("{adapter_id}::{root}");
        let session = self
            .sessions
            .iter()
            .find(|existing| existing.root_key() == key)?;
        match session.retire_at {
            None => Some(session),
            Some(retired) if retired.elapsed() < SESSION_GRACE => Some(session),
            Some(_) => None,
        }
    }

    pub fn current(&self) -> Option<&Session> {
        self.current.and_then(|index| self.sessions.get(index))
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    fn retire_others(&mut self, keep: usize) {
        let now = Instant::now();
        for (index, session) in self.sessions.iter_mut().enumerate() {
            if index == keep {
                session.retire_at = None;
            } else if session.retire_at.is_none() {
                session.retire_at = Some(now);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Opened, Target};

    fn opened(root: &str, document: Option<&str>, url: &str) -> Opened {
        Opened {
            url: url.into(),
            title: "Hello".into(),
            inspector_url: Some(format!("{url}inspect")),
            target: Target {
                id: "fixture".into(),
                path: root.into(),
                adapter_id: "fixture".into(),
                label: "Fixture".into(),
                detail: None,
            },
            document: document.map(str::to_string),
        }
    }

    #[test]
    fn same_root_reuses_one_session() {
        let mut table = SessionTable::default();
        table.record(Session::from_opened(&opened(
            "/tmp/a",
            None,
            "http://127.0.0.1:1/",
        )));
        table.record(Session::from_opened(&opened(
            "/tmp/a",
            Some("about"),
            "http://127.0.0.1:1/about",
        )));
        assert_eq!(table.len(), 1);
        assert_eq!(table.current().unwrap().document.as_deref(), Some("about"));
        assert!(table.reusable("fixture", "/tmp/a").is_some());
    }

    #[test]
    fn hopping_roots_keeps_the_first_warm() {
        let mut table = SessionTable::default();
        table.record(Session::from_opened(&opened(
            "/tmp/a",
            None,
            "http://127.0.0.1:1/",
        )));
        table.record(Session::from_opened(&opened(
            "/tmp/b",
            None,
            "http://127.0.0.1:2/",
        )));
        assert_eq!(table.len(), 2);
        assert_eq!(table.current().unwrap().root, "/tmp/b");
        let first = table.reusable("fixture", "/tmp/a").unwrap();
        assert_eq!(first.origin, "http://127.0.0.1:1");
        table.record(Session::from_opened(&opened(
            "/tmp/a",
            None,
            "http://127.0.0.1:1/",
        )));
        assert_eq!(table.len(), 2);
        assert_eq!(table.current().unwrap().root, "/tmp/a");
    }
}

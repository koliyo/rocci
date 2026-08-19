use crate::Opened;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub adapter_id: String,
    pub root: String,
    pub document: Option<String>,
    pub url: String,
    pub title: String,
    pub inspector_url: Option<String>,
}

impl Session {
    pub fn from_opened(opened: &Opened) -> Self {
        Self {
            adapter_id: opened.target.adapter_id.clone(),
            root: opened.target.path.clone(),
            document: opened.document.clone(),
            url: opened.url.clone(),
            title: opened.title.clone(),
            inspector_url: opened.inspector_url.clone(),
        }
    }

    pub fn key(&self) -> String {
        format!("{}::{}::{:?}", self.adapter_id, self.root, self.document)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionTable {
    sessions: Vec<Session>,
    current: Option<usize>,
}

impl SessionTable {
    pub fn record(&mut self, session: Session) -> usize {
        if let Some(index) = self
            .sessions
            .iter()
            .position(|existing| existing.key() == session.key())
        {
            self.sessions[index] = session;
            self.current = Some(index);
            return index;
        }
        self.sessions.push(session);
        let index = self.sessions.len() - 1;
        self.current = Some(index);
        index
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Opened, Target};

    fn opened(document: Option<&str>, url: &str) -> Opened {
        Opened {
            url: url.into(),
            title: "Hello".into(),
            inspector_url: Some(format!("{url}inspect")),
            target: Target {
                id: "fixture".into(),
                path: "/tmp/fixture".into(),
                adapter_id: "fixture".into(),
                label: "Fixture".into(),
                detail: None,
            },
            document: document.map(str::to_string),
        }
    }

    #[test]
    fn two_opens_keep_one_current_session() {
        let mut table = SessionTable::default();
        table.record(Session::from_opened(&opened(None, "http://127.0.0.1:1/")));
        table.record(Session::from_opened(&opened(
            Some("about"),
            "http://127.0.0.1:1/about",
        )));
        assert_eq!(table.len(), 2);
        assert_eq!(table.current().unwrap().document.as_deref(), Some("about"));
        table.record(Session::from_opened(&opened(None, "http://127.0.0.1:1/")));
        assert_eq!(table.len(), 2);
        assert_eq!(table.current().unwrap().document, None);
    }
}

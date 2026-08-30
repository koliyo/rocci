//! Sync wasm/host sqlite. Nested queries serialize other `handle`s (Phase 0).

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::abi::{OrdinaryResponse, OutcomeToHost, ServerHeader, ServerRequest};
use crate::guest::RocGuest;

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("open sqlite")?;
        conn.execute(
            "CREATE TABLE notes (id INTEGER PRIMARY KEY, body TEXT NOT NULL)",
            [],
        )?;
        conn.execute("INSERT INTO notes (body) VALUES ('hello-sqlite')", [])?;
        Ok(Self { conn })
    }

    pub fn read_body(&self) -> Result<String> {
        let body: String =
            self.conn
                .query_row("SELECT body FROM notes ORDER BY id LIMIT 1", [], |row| {
                    row.get(0)
                })?;
        Ok(body)
    }
}

pub struct SqliteGuest {
    store: SqliteStore,
}

impl SqliteGuest {
    pub fn new(store: SqliteStore) -> Self {
        Self { store }
    }
}

fn ordinary_body(body: String) -> OutcomeToHost {
    OutcomeToHost::Ordinary(OrdinaryResponse {
        exit_code: 0,
        body: body.into_bytes(),
        headers: vec![ServerHeader {
            name: "content-type".into(),
            value: "text/plain; charset=utf-8".into(),
        }],
        status: 200,
        stop: false,
    })
}

impl RocGuest for SqliteGuest {
    fn init(&mut self) {}

    fn respond(&mut self, _request: &ServerRequest) -> OutcomeToHost {
        let body = self.store.read_body().unwrap_or_else(|_| "err".into());
        ordinary_body(body)
    }

    fn shutdown(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::IncomingRequest;
    use crate::handle::Adapter;
    use std::time::{Duration, Instant};

    fn get_root() -> IncomingRequest {
        IncomingRequest {
            method: "GET".into(),
            path: "/".into(),
            headers: vec![],
            body: Vec::new(),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn sqlite_request_returns_200() {
        let store = SqliteStore::memory().unwrap();
        let mut adapter = Adapter::new(SqliteGuest::new(store));
        let response = adapter.handle(get_root()).await.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"hello-sqlite");
    }

    struct StallSqliteGuest {
        store: SqliteStore,
        stall: Duration,
    }

    impl RocGuest for StallSqliteGuest {
        fn init(&mut self) {}
        fn respond(&mut self, _request: &ServerRequest) -> OutcomeToHost {
            std::thread::sleep(self.stall);
            let body = self.store.read_body().unwrap_or_else(|_| "err".into());
            ordinary_body(body)
        }
        fn shutdown(&mut self) {}
    }

    #[tokio::test(flavor = "current_thread")]
    async fn sqlite_in_respond_serializes() {
        let stall = Duration::from_millis(40);
        let start = Instant::now();
        let mut a = Adapter::new(StallSqliteGuest {
            store: SqliteStore::memory().unwrap(),
            stall,
        });
        let mut b = Adapter::new(StallSqliteGuest {
            store: SqliteStore::memory().unwrap(),
            stall,
        });
        let (ra, rb) = tokio::join!(a.handle(get_root()), b.handle(get_root()));
        ra.unwrap();
        rb.unwrap();
        let wall = start.elapsed();
        assert!(
            wall >= stall + stall / 2,
            "nested sqlite/sleep in respond! serializes other handles: {wall:?}"
        );
    }
}

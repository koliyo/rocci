use http::HeaderMap;

use roc_core::Session;

pub const SESSION_COOKIE: &str = "roc_session";

pub fn parse_session_cookie(headers: &HeaderMap) -> Option<&str> {
    let cookies = headers
        .get(http::header::COOKIE)
        .and_then(|value| value.to_str().ok())?;
    cookies.split(';').find_map(|cookie| {
        let mut parts = cookie.trim().splitn(2, '=');
        (parts.next() == Some(SESSION_COOKIE)).then(|| parts.next())?
    })
}

pub fn session_cookie_header(session: &Session) -> String {
    format!(
        "{SESSION_COOKIE}={}; HttpOnly; SameSite=Strict; Path=/",
        session.token
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use roc_core::{Session, WindowId};

    #[test]
    fn reads_the_named_session_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::COOKIE,
            "other=1; roc_session=abc; extra=2".parse().unwrap(),
        );
        assert_eq!(parse_session_cookie(&headers), Some("abc"));
        let header = session_cookie_header(&Session {
            window_id: WindowId::new("main"),
            token: "abc".into(),
            start_url: "/".into(),
        });
        assert!(header.contains("HttpOnly; SameSite=Strict"));
    }
}

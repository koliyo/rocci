use rocci_datastar::signals::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TestSignals {
    count: i32,
    query: String,
}

#[test]
fn test_parse_signals_json() {
    let json = r#"{"count": 42, "query": "search"}"#;
    let signals: TestSignals = from_json_str(json).expect("valid signals json");
    assert_eq!(
        signals,
        TestSignals {
            count: 42,
            query: "search".to_string()
        }
    );
}

#[test]
fn test_parse_signals_query_string() {
    let qs = "?other=1&datastar=%7B%22count%22%3A5%2C%22query%22%3A%22test%22%7D";
    let signals: Option<TestSignals> = from_query_str(qs).expect("valid query signals");
    assert_eq!(
        signals,
        Some(TestSignals {
            count: 5,
            query: "test".to_string()
        })
    );
}

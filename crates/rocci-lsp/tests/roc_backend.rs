use std::env;
use std::path::PathBuf;

use lsp_types::{
    ClientCapabilities, DidOpenTextDocumentParams, GeneralClientCapabilities, HoverContents,
    HoverParams, InitializeParams, Position, PositionEncodingKind, TextDocumentItem,
    TextDocumentPositionParams, Uri, WorkDoneProgressParams,
};
use rocci_lsp::{ChildRocBackend, LanguageServer, RocBackend};

fn require_roc() -> bool {
    env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1")
}

#[test]
#[ignore]
fn live_child_hovers_generated_type_module() {
    let backend = match ChildRocBackend::spawn_from_env() {
        Ok(backend) => backend,
        Err(err) => {
            if require_roc() {
                panic!("roc experimental-lsp required: {err}");
            }
            return;
        }
    };
    let mut backend = backend;
    let dir = env::temp_dir().join(format!("rocci-lsp-roc-{}", std::process::id()));
    let path = dir.join("Hello.roc");
    let text = "Hello := [].{\n    greet = |name| name\n}\n";
    backend
        .sync_projection(&path, text)
        .expect("sync projection");
    let hover = backend
        .hover(&path, Position::new(1, 4))
        .expect("hover on greet");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(
        markup.value.contains("->") || markup.value.contains("greet"),
        "unexpected hover: {}",
        markup.value
    );
    let _keep: PathBuf = dir;
}

#[test]
#[ignore]
fn live_child_hovers_interpolation_through_language_server() {
    let backend = match ChildRocBackend::spawn_from_env() {
        Ok(backend) => backend,
        Err(err) => {
            if require_roc() {
                panic!("roc experimental-lsp required: {err}");
            }
            return;
        }
    };
    let src = r#"
@component Hello = |{ title }| {
    <p>{title}</p>
}
"#;
    let mut server = LanguageServer::new();
    server.initialize(InitializeParams {
        capabilities: ClientCapabilities {
            general: Some(GeneralClientCapabilities {
                position_encodings: Some(vec![PositionEncodingKind::UTF16]),
                ..GeneralClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        },
        ..InitializeParams::default()
    });
    server.set_roc_backend(Box::new(backend));
    let uri: Uri = "file:///test.rocci".parse().expect("uri");
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocci".to_string(),
                version: 1,
                text: src.to_string(),
            },
        })
        .expect("open");
    let title = src.find("{title}").expect("interp") + 1;
    let mut line = 0u32;
    let mut start = 0usize;
    for (i, ch) in src.char_indices() {
        if i >= title {
            break;
        }
        if ch == '\n' {
            line += 1;
            start = i + 1;
        }
    }
    let character = (title - start) as u32;
    let hover = server
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri },
                position: Position::new(line, character),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("hover on {title}");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(
        markup.value.contains("Str")
            || markup.value.contains("title")
            || markup.value.contains("->"),
        "unexpected hover: {}",
        markup.value
    );
}

#[test]
#[ignore]
fn live_child_hovers_platform_import_through_language_server() {
    let backend = match ChildRocBackend::spawn_from_env() {
        Ok(backend) => backend,
        Err(err) => {
            if require_roc() {
                panic!("roc experimental-lsp required: {err}");
            }
            return;
        }
    };
    let src = r#"
import pf.Sqlite
import pf.Path

@context { db : Sqlite.Db }

@init {
    db = Sqlite.open!(Sqlite.default_config(Path.utf8("./x.db")))?
    { db: db }
}
"#;
    let mut server = LanguageServer::new();
    server.initialize(InitializeParams {
        capabilities: ClientCapabilities {
            general: Some(GeneralClientCapabilities {
                position_encodings: Some(vec![PositionEncodingKind::UTF16]),
                ..GeneralClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        },
        ..InitializeParams::default()
    });
    server.set_roc_backend(Box::new(backend));
    let uri: Uri = "file:///test.rocci".parse().expect("uri");
    server
        .did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocci".to_string(),
                version: 1,
                text: src.to_string(),
            },
        })
        .expect("open");
    let open = src.find("Sqlite.open").expect("Sqlite.open") + "Sqlite.".len();
    let mut line = 0u32;
    let mut start = 0usize;
    for (i, ch) in src.char_indices() {
        if i >= open {
            break;
        }
        if ch == '\n' {
            line += 1;
            start = i + 1;
        }
    }
    let character = (open - start) as u32;
    let hover = server
        .hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri },
                position: Position::new(line, character),
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
        .expect("hover on Sqlite.open");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(
        markup.value.contains("Sqlite") && markup.value.contains("Try"),
        "unexpected hover: {}",
        markup.value
    );
}

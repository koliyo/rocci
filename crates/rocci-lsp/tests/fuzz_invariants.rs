use lsp_types::{
    ClientCapabilities, DidOpenTextDocumentParams, GeneralClientCapabilities, InitializeParams,
    PartialResultParams, Position, PositionEncodingKind, Range, SemanticTokens,
    SemanticTokensParams, SemanticTokensRangeParams, TextDocumentIdentifier, TextDocumentItem, Uri,
    WorkDoneProgressParams,
};
use rocci_lsp::{LanguageServer, RegionPurpose, RegionTree, extract_rocci_regions};
use rocci_template::{PositionEncoding, SourceFile};

const KITCHEN_SINK_ROCCI: &str = include_str!("../../../test/AllSyntax.rocci");
const EMBEDDED_ROCCI: &str = include_str!("../../../test/EmbeddedLanguages.rocci");

fn test_uri(path: &str) -> Uri {
    format!("file:///{path}").parse().expect("valid test uri")
}

fn initialize(utf8: bool) -> LanguageServer {
    let mut server = LanguageServer::new();
    let encodings = if utf8 {
        vec![PositionEncodingKind::UTF8, PositionEncodingKind::UTF16]
    } else {
        vec![PositionEncodingKind::UTF16]
    };
    server.initialize(InitializeParams {
        capabilities: ClientCapabilities {
            general: Some(GeneralClientCapabilities {
                position_encodings: Some(encodings),
                ..GeneralClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        },
        ..InitializeParams::default()
    });
    server
}

fn assert_token_invariants(tokens: &SemanticTokens, src: &str, encoding: PositionEncoding) {
    let legend = rocci_lsp::tokens::legend();
    let max_token_type = legend.token_types.len() as u32;

    let source = SourceFile::new("test", src);
    let mut curr_line = 0u32;
    let mut curr_col = 0u32;

    for (idx, token) in tokens.data.iter().enumerate() {
        assert!(
            token.token_type < max_token_type,
            "token #{idx} has invalid token_type {} (max is {})",
            token.token_type,
            max_token_type
        );
        assert!(
            token.length > 0,
            "token #{idx} has zero length at line {}, col {}",
            curr_line,
            curr_col
        );

        if token.delta_line == 0 {
            curr_col += token.delta_start;
        } else {
            curr_line += token.delta_line;
            curr_col = token.delta_start;
        }

        // Token must not exceed line length
        let line_offset = source.offset_at(curr_line, 0, encoding) as usize;
        let line_end_offset = source.offset_at(curr_line + 1, 0, encoding) as usize;
        let line_text = if line_end_offset > line_offset && line_end_offset <= src.len() {
            &src[line_offset..line_end_offset]
        } else if line_offset < src.len() {
            &src[line_offset..]
        } else {
            ""
        };
        let line_units = match encoding {
            PositionEncoding::Utf8 => line_text.trim_end_matches(['\r', '\n']).len() as u32,
            PositionEncoding::Utf16 => line_text
                .trim_end_matches(['\r', '\n'])
                .chars()
                .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
                .sum(),
        };

        assert!(
            curr_col + token.length <= line_units + 1, // allow at boundary
            "token #{idx} spans beyond line {curr_line}: start_col={curr_col}, length={}, line_units={line_units}",
            token.length
        );
    }
}

fn assert_region_invariants(tree: &RegionTree, src: &str) {
    tree.validate(src.len()).expect("region tree must be valid");

    for (idx, region) in tree.regions.iter().enumerate() {
        assert!(
            region.span.start <= region.span.end,
            "region #{idx} has inverted span: {:?}",
            region.span
        );
        assert!(
            region.span.end as usize <= src.len(),
            "region #{idx} span {:?} exceeds source length {}",
            region.span,
            src.len()
        );

        if let Some(parent_id) = region.parent {
            let parent = &tree.regions[parent_id];
            assert!(
                parent.span.start <= region.span.start && region.span.end <= parent.span.end,
                "child region #{idx} {:?} not contained in parent #{:?} {:?}",
                region.span,
                parent_id,
                parent.span
            );

            if parent.purpose == RegionPurpose::Executable {
                assert_ne!(
                    region.purpose,
                    RegionPurpose::DisplayOnly,
                    "executable region #{:?} contains display-only child #{idx}",
                    parent_id
                );
            }
        }
    }
}

#[test]
fn test_invariants_on_all_standard_fixtures() {
    let fixtures = [
        ("AllSyntax.rocci", KITCHEN_SINK_ROCCI),
        ("EmbeddedLanguages.rocci", EMBEDDED_ROCCI),
    ];

    for (name, text) in fixtures {
        for utf8 in [true, false] {
            let mut server = initialize(utf8);
            let encoding = server.encoding();
            let uri = test_uri(name);

            server.did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "rocci".to_string(),
                    version: 1,
                    text: text.to_string(),
                },
            });

            // Validate regions
            let parsed = rocci_template::parse(SourceFile::new(name, text));
            let tree = extract_rocci_regions(name, text, &parsed.document);
            assert_region_invariants(&tree, text);

            // Validate full tokens
            let full_result = server
                .semantic_tokens_full(SemanticTokensParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                })
                .expect("token response");
            let lsp_types::SemanticTokensResult::Tokens(tokens) = full_result else {
                panic!("expected full tokens for {name}");
            };
            assert_token_invariants(&tokens, text, encoding);

            // Validate range tokens
            let range_result = server
                .semantic_tokens_range(SemanticTokensRangeParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    range: Range::new(Position::new(2, 0), Position::new(20, 0)),
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                })
                .expect("range token response");
            let lsp_types::SemanticTokensRangeResult::Tokens(range_tokens) = range_result else {
                panic!("expected range tokens for {name}");
            };
            assert_token_invariants(&range_tokens, text, encoding);
        }
    }
}

fn run_byte_slicing_stress(stride: usize) {
    let unicode_rocci = r#"
module [UnicodeApp]

# 🦀 Rustacean rocket 🚀 and non-BMP character 𠜎 (U+2070E)
# Mathematical symbols: ∑ ∫ π ≠ ≤ ≥
# Multi-byte text: こんにちは世界, مرحبا بالعالم, שלום עולם, Привет мир

@component UnicodeCard = |{ title : Str, emoji : Str }| {
    @css {
        .card-🦀 {
            font-family: "Noto Color Emoji", sans-serif;
            padding: 1rem;
            content: "🚀";
        }
    }
    <div class="card-🦀" data-emoji={emoji} data-nonbmp="𠜎">
        <h3>{title} " 🚀 " {emoji}</h3>
        <p>"Non-BMP character: 𠜎 in text"</p>
    </div>
}
"#;

    for (name, text) in [("Unicode.rocci", unicode_rocci)] {
        for utf8 in [true, false] {
            let mut server = initialize(utf8);
            let encoding = server.encoding();
            let uri = test_uri(name);

            let indices: Vec<_> = text.char_indices().map(|(idx, _)| idx).collect();
            for (i, &byte_idx) in indices.iter().enumerate() {
                if stride > 1 && i % stride != 0 && i != indices.len() - 1 {
                    continue;
                }
                let slice = &text[..byte_idx];
                server.did_open(DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: uri.clone(),
                        language_id: "rocci".to_string(),
                        version: 1,
                        text: slice.to_string(),
                    },
                });

                if let Some(lsp_types::SemanticTokensResult::Tokens(tokens)) = server
                    .semantic_tokens_full(SemanticTokensParams {
                        text_document: TextDocumentIdentifier { uri: uri.clone() },
                        work_done_progress_params: WorkDoneProgressParams::default(),
                        partial_result_params: PartialResultParams::default(),
                    })
                {
                    assert_token_invariants(&tokens, slice, encoding);
                }
            }
        }
    }
}

#[test]
#[ignore = "byte slicing stress; run with: cargo test -p rocci-lsp --test fuzz_invariants -- --ignored"]
fn test_multibyte_and_non_bmp_byte_slicing_stress() {
    run_byte_slicing_stress(8);
}

#[test]
#[ignore = "exhaustive byte slicing stress; run with: cargo test -p rocci-lsp --test fuzz_invariants -- --ignored"]
fn test_multibyte_and_non_bmp_byte_slicing_exhaustive() {
    run_byte_slicing_stress(1);
}

#[test]
#[ignore = "malformed-construct stress; run with: cargo test -p rocci-lsp --test fuzz_invariants -- --ignored"]
fn test_truncated_and_malformed_constructs_stress() {
    let malformed_cases = [
        // Unclosed strings & comments
        r#"@component A = |{}| { "unclosed string"#,
        r#"@component A = |{}| { """multiline unclosed"#,
        r#"@component A = |{}| { /* unclosed block comment"#,
        r#"@component A = |{}| { <!-- unclosed html comment"#,
        r#"@component A = |{}| { <!-- <tag> inside comment"#,
        // Unclosed tags & components
        r#"@component A = |{}| { <div"#,
        r#"@component A = |{}| { <div class="foo""#,
        r#"@component A = |{}| { <div <span <p>"#,
        r#"@component A = |{}| { <Component"#,
        r#"@component A = |{}| { <Component prop={"#,
        r#"@component A = |{}| { <Component prop={person."#,
        r#"@component A = |{}| { <Component prop="val" <Other />"#,
        r#"@component A = |{}| { </div"#,
        r#"@component A = |{}| { </Component.Path.Name"#,
        // Unclosed brackets & delimiters
        r#"@component A = |{}| { {{{{{(((([[[["#,
        r#"@component A = |{}| { } } } ) ) ] ]"#,
        r#"@component A = |{}| { <><><><>"#,
        r#"@component A = |{}| { { a: { b: { c:"#,
        // Directives truncated at all stages
        r#"@if"#,
        r#"@if active"#,
        r#"@if active {"#,
        r#"@if active { <p>yes</p> } @else"#,
        r#"@if active { <p>yes</p> } @else if"#,
        r#"@if active { <p>yes</p> } @else if other {"#,
        r#"@for"#,
        r#"@for item"#,
        r#"@for item in"#,
        r#"@for item in items"#,
        r#"@for item in items {"#,
        r#"@match"#,
        r#"@match status"#,
        r#"@match status {"#,
        r#"@match status { Active ->"#,
        r#"@match status { Active =>"#,
        r#"@let"#,
        r#"@let x"#,
        r#"@let x ="#,
        r#"@let x = 42"#,
        r#"@on:get"#,
        r#"@on:get("/api""#,
        r#"@on:get("/api") {"#,
        // CSS truncated & broken
        r#"@css"#,
        r#"@css {"#,
        r#"@css { .card"#,
        r#"@css { .card {"#,
        r#"@css { .card { color"#,
        r#"@css { .card { color:"#,
        r#"@css { .card { color: #fff"#,
        r#"@css { .card { color: #fff;"#,
        r#"@css { @media"#,
        r#"@css { @media (min-width: 600px) {"#,
        r#"@css { :root { --custom-color: "#,
    ];

    for (idx, &src) in malformed_cases.iter().enumerate() {
        for utf8 in [true, false] {
            let mut server = initialize(utf8);
            let encoding = server.encoding();
            let uri = test_uri(&format!("Malformed_{idx}.rocci"));

            server.did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "rocci".to_string(),
                    version: 1,
                    text: src.to_string(),
                },
            });

            if let Some(lsp_types::SemanticTokensResult::Tokens(tokens)) = server
                .semantic_tokens_full(SemanticTokensParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                })
            {
                assert_token_invariants(&tokens, src, encoding);
            }
        }
    }
}

#[test]
#[ignore = "deep nesting stress; run with: cargo test -p rocci-lsp --test fuzz_invariants -- --ignored"]
fn test_deeply_nested_structures() {
    // 100 levels of nested tags
    let mut deep_tags = String::from("@component Deep = |{}| {\n");
    for i in 0..100 {
        deep_tags.push_str(&format!("<div class=\"level-{i}\">"));
    }
    deep_tags.push_str("<span>Deep content</span>");
    for _ in 0..100 {
        deep_tags.push_str("</div>");
    }
    deep_tags.push_str("\n}\n");

    // 100 levels of nested braces / expressions
    let mut deep_braces = String::from("@component Braces = |{}| {\n<p>");
    for _ in 0..100 {
        deep_braces.push('{');
    }
    deep_braces.push_str("42");
    for _ in 0..100 {
        deep_braces.push('}');
    }
    deep_braces.push_str("</p>\n}\n");

    for (name, text) in [
        ("DeepTags.rocci", &deep_tags),
        ("DeepBraces.rocci", &deep_braces),
    ] {
        for utf8 in [true, false] {
            let mut server = initialize(utf8);
            let encoding = server.encoding();
            let uri = test_uri(name);

            server.did_open(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: uri.clone(),
                    language_id: "rocci".to_string(),
                    version: 1,
                    text: text.to_string(),
                },
            });

            let result = server.semantic_tokens_full(SemanticTokensParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            });

            if let Some(lsp_types::SemanticTokensResult::Tokens(tokens)) = result {
                assert_token_invariants(&tokens, text, encoding);
            }
        }
    }
}

fn run_mutation_fuzzing(iterations: usize) {
    // Simple LCG PRNG for determinism across platforms
    struct SimplePrng {
        state: u64,
    }
    impl SimplePrng {
        fn new(seed: u64) -> Self {
            Self { state: seed }
        }
        fn next_u32(&mut self) -> u32 {
            self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (self.state >> 32) as u32
        }
        fn gen_range(&mut self, min: usize, max: usize) -> usize {
            if min >= max {
                return min;
            }
            min + (self.next_u32() as usize % (max - min))
        }
    }

    let mut prng = SimplePrng::new(0xDEAD_BEEF_CAFE_BABE);
    let mut server = initialize(true);
    let encoding = server.encoding();

    let base_fixtures = [KITCHEN_SINK_ROCCI, EMBEDDED_ROCCI];

    let injection_bytes = [
        b'{', b'}', b'<', b'>', b'@', b'"', b'/', b':', b'=', b';', b'#', b'\\', b'\n', b'\r',
        b' ', b'\t', b'0', b'a', b'Z', 0xF0, 0x9F, 0x94, 0xA5, // 🔥 (UTF-8 fragment)
    ];

    for iteration in 0..iterations {
        let base = base_fixtures[prng.gen_range(0, base_fixtures.len())];
        let mut mutated = base.as_bytes().to_vec();

        // Apply 1 to 5 random mutations
        let num_mutations = prng.gen_range(1, 6);
        for _ in 0..num_mutations {
            let mutation_type = prng.gen_range(0, 4);
            match mutation_type {
                0 => {
                    // Random insertion
                    if mutated.len() < 10_000 {
                        let pos = prng.gen_range(0, mutated.len() + 1);
                        let byte = injection_bytes[prng.gen_range(0, injection_bytes.len())];
                        mutated.insert(pos, byte);
                    }
                }
                1 => {
                    // Random deletion
                    if !mutated.is_empty() {
                        let pos = prng.gen_range(0, mutated.len());
                        mutated.remove(pos);
                    }
                }
                2 => {
                    // Random replacement
                    if !mutated.is_empty() {
                        let pos = prng.gen_range(0, mutated.len());
                        let byte = injection_bytes[prng.gen_range(0, injection_bytes.len())];
                        mutated[pos] = byte;
                    }
                }
                3 => {
                    // Random truncation
                    if !mutated.is_empty() {
                        let new_len = prng.gen_range(0, mutated.len());
                        mutated.truncate(new_len);
                    }
                }
                _ => unreachable!(),
            }
        }

        let mutated_str = String::from_utf8_lossy(&mutated);
        let uri = test_uri(&format!("Fuzz_{iteration}.rocci"));

        server.did_open(DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "rocci".to_string(),
                version: 1,
                text: mutated_str.to_string(),
            },
        });

        let result = server.semantic_tokens_full(SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        });

        if let Some(lsp_types::SemanticTokensResult::Tokens(tokens)) = result {
            assert_token_invariants(&tokens, &mutated_str, encoding);
        }
    }
}

#[test]
#[ignore = "50-iteration mutation fuzz; run with: cargo test -p rocci-lsp --test fuzz_invariants -- --ignored"]
fn test_deterministic_mutation_fuzzing() {
    let iterations = std::env::var("ROCCI_FUZZ_ITERATIONS")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(50);
    run_mutation_fuzzing(iterations);
}

#[test]
#[ignore = "exhaustive 5000-iteration mutation fuzzing; run with: cargo test -p rocci-lsp --test fuzz_invariants -- --ignored"]
fn test_deterministic_mutation_fuzzing_deep() {
    let iterations = std::env::var("ROCCI_FUZZ_ITERATIONS")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(5_000);
    run_mutation_fuzzing(iterations);
}

use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

fn rocci_okf_bin() -> PathBuf {
    let mut path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rocci-okf");
    if !path.is_file() {
        path = PathBuf::from(env!("CARGO_BIN_EXE_rocci-okf"));
    }
    path
}

fn rpc(
    stdin: &mut impl Write,
    stdout: &mut impl BufRead,
    id: u64,
    method: &str,
    params: serde_json::Value,
) -> serde_json::Value {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    writeln!(stdin, "{request}").unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn probe_and_list_over_stdio() {
    let dir = env::temp_dir().join(format!(
        "okf-browser-adapter-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(dir.join("plans")).unwrap();
    fs::write(
        dir.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Bundle\n",
    )
    .unwrap();
    fs::write(dir.join("plans/example.md"), "# Example\n").unwrap();

    let mut child = Command::new(rocci_okf_bin())
        .arg("browser-adapter")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    let init = rpc(
        &mut stdin,
        &mut stdout,
        1,
        "initialize",
        serde_json::json!({ "protocolVersion": 1 }),
    );
    assert_eq!(init["result"]["adapterId"], "okf");

    let missed = rpc(
        &mut stdin,
        &mut stdout,
        2,
        "probe",
        serde_json::json!({ "path": env::temp_dir().display().to_string() }),
    );
    assert_eq!(missed["result"]["claimed"], false);

    let hit = rpc(
        &mut stdin,
        &mut stdout,
        3,
        "probe",
        serde_json::json!({ "path": dir.display().to_string() }),
    );
    assert_eq!(hit["result"]["claimed"], true);

    let listed = rpc(
        &mut stdin,
        &mut stdout,
        4,
        "listDocuments",
        serde_json::json!({ "root": dir.display().to_string() }),
    );
    assert_eq!(listed["result"]["documents"][0]["id"], "plans/example");

    let _ = rpc(
        &mut stdin,
        &mut stdout,
        5,
        "shutdown",
        serde_json::json!({}),
    );
    drop(stdin);
    let _ = child.wait();
    let _ = fs::remove_dir_all(&dir);
}

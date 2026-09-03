app [main!] {
    pf: platform "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst",
}

import pf.Cmd
import pf.Env
import pf.OsStr
import pf.Path
import pf.Stdout

Metadata : {
    workspace_members : List(Str),
    pkgs : List(Pkg),
}

Pkg : {
    name : Str,
    id : Str,
    dependencies : List(Dep),
}

Dep : {
    name : Str,
}

subset_json = "{\"workspace_members\":[\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\",\"beta 0.1.0 (path+file:///tmp/ws/beta)\"],\"packages\":[{\"name\":\"alpha\",\"id\":\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\",\"dependencies\":[{\"name\":\"beta\"},{\"name\":\"serde\"}]},{\"name\":\"beta\",\"id\":\"beta 0.1.0 (path+file:///tmp/ws/beta)\",\"dependencies\":[]}]}"

rename_packages_key = |src| {
    match Str.split_first(src, "\"packages\":") {
        Ok({ before, after }) => Str.concat(before, Str.concat("\"pkgs\":", after))
        Err(_) => src
    }
}

decode_metadata : Str -> Try(Metadata, [InvalidJson(Str), MissingRequiredField(Str), ..])
decode_metadata = |src|
    Encoding.Json.parse(rename_packages_key(src))

argv_tag = |arg|
    match OsStr.to_raw(arg) {
        Utf8(_) => "Utf8"
        UnixBytes(_) => "UnixBytes"
        WindowsU16s(_) => "WindowsU16s"
    }

os_utf8 = |arg|
    match OsStr.to_raw(arg) {
        Utf8(s) => Ok(s)
        UnixBytes(bytes) => Ok(Str.from_utf8_lossy(bytes))
        _ => Err({})
    }

path_display = |path|
    match Path.to_str(path) {
        Ok(s) => s
        Err(_) => Path.display(path)
    }

main! = |args| {
    Stdout.line!("argv_count=${List.len(args).to_str()}")?
    match List.get(args, 0) {
        Ok(first) => Stdout.line!("argv0_tag=${argv_tag(first)}")?
        Err(_) => Stdout.line!("argv0_tag=missing")?
    }
    match List.get(args, 1) {
        Ok(second) => {
            match os_utf8(second) {
                Ok(s) => Stdout.line!("argv1=${s}")?
                Err(_) => Stdout.line!("argv1=<non-utf8>")?
            }
        }
        Err(_) => Stdout.line!("argv1=<none>")?
    }

    true_code = Cmd.new("true").exec_exit_code!()?
    false_code = Cmd.new("false").exec_exit_code!()?
    Stdout.line!("true_exit=${true_code.to_str()}")?
    Stdout.line!("false_exit=${false_code.to_str()}")?

    tmp = Env.temp_dir!()
    pin_path = Path.join(tmp, "rocci-ops-pin.txt")
    Path.write_utf8!(pin_path, "pin-ok")?
    read_back = Path.read_utf8!(pin_path)?
    Path.delete!(pin_path)?
    Stdout.line!("path_roundtrip=${read_back}")?

    match Env.var_str!(OsStr.utf8("HOME")) {
        Ok(home) => Stdout.line!("env_HOME_len=${Str.to_utf8(home).len().to_str()}")?
        Err(_) => Stdout.line!("env_HOME=<unset>")?
    }

    fixture = Path.read_utf8!(Path.utf8("roc/rocci-ops/fixtures/cargo-metadata-subset.json"))?
    meta = decode_metadata(fixture)?
    Stdout.line!("json_members=${List.len(meta.workspace_members).to_str()}")?
    Stdout.line!("json_pkgs=${List.len(meta.pkgs).to_str()}")?

    before = Env.cwd!()?
    Env.set_cwd!(tmp)?
    pwd = Cmd.new("pwd").exec_output!()?
    Env.set_cwd!(before)?
    Stdout.line!("cwd_before=${path_display(before)}")?
    Stdout.line!("cwd_tmp=${path_display(tmp)}")?
    Stdout.line!("cwd_pwd=${Str.trim_end(pwd.stdout_utf8)}")?
    Ok({})
}

expect
    match decode_metadata(subset_json) {
        Ok(meta) => List.len(meta.pkgs) == 2 and List.len(meta.workspace_members) == 2
        Err(_) => Bool.False
    }

expect
    match decode_metadata(subset_json) {
        Ok(meta) => {
            match List.get(meta.pkgs, 0) {
                Ok(pkg) => pkg.name == "alpha" and List.len(pkg.dependencies) == 2
                Err(_) => Bool.False
            }
        }
        Err(_) => Bool.False
    }

expect
    match decode_metadata("{\"workspace_members\":[],\"packages\":[],\"version\":1}") {
        Ok(meta) => List.len(meta.pkgs) == 0 and List.len(meta.workspace_members) == 0
        Err(_) => Bool.False
    }

expect
    match decode_metadata("{\"workspace_members\":[\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\"],\"packages\":[{\"name\":\"alpha\",\"id\":\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\",\"version\":\"0.1.0\",\"source\":null,\"dependencies\":[{\"name\":\"beta\",\"source\":null}]}]}") {
        Ok(meta) => {
            match List.get(meta.pkgs, 0) {
                Ok(pkg) => pkg.name == "alpha" and List.len(pkg.dependencies) == 1
                Err(_) => Bool.False
            }
        }
        Err(_) => Bool.False
    }

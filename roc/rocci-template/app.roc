app [main!] {
    pf: platform "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst",
    tpl: "main.roc",
}

import pf.OsStr
import pf.Path
import pf.Stdout
import pf.Stderr
import pf.Stdin
import tpl.Template

main! = |args| {
    parsed = parse_args(args.drop_first(1))
    match parsed {
        Err(msg) => {
            Stderr.line!(msg)?
            Err(Exit(1))
        }
        Ok(opts) => {
            src = read_src!(opts.input)?
            file_name = input_name(opts.input)
            roc = Template.compile(src, file_name)
            write_out!(opts.output, roc)?
            Ok({})
        }
    }
}

parse_args = |args| {
    var $i = 0.U64
    var $input = ""
    var $output = ""
    var $ok = Bool.True
    var $err = ""
    while $ok and $i < List.len(args) {
        match List.get(args, $i) {
            Err(_) => {
                $ok = Bool.False
            }
            Ok(arg) => {
                match os_utf8(arg) {
                    Err(_) => {
                        $ok = Bool.False
                        $err = "non-UTF-8 argument"
                    }
                    Ok("-h") => {
                        $ok = Bool.False
                        $err = "usage: app.roc [--] <file.rocci|-> [-o <file.roc|->]"
                    }
                    Ok("-o") => {
                        $i = $i + 1
                        match List.get(args, $i) {
                            Ok(out_arg) => {
                                match os_utf8(out_arg) {
                                    Ok(path) => {
                                        $output = path
                                    }
                                    Err(_) => {
                                        $ok = Bool.False
                                        $err = "non-UTF-8 -o path"
                                    }
                                }
                            }
                            Err(_) => {
                                $ok = Bool.False
                                $err = "expected path after -o"
                            }
                        }
                    }
                    Ok(path) => {
                        if $input == "" {
                            $input = path
                        } else {
                            $ok = Bool.False
                            $err = "unexpected extra argument"
                        }
                    }
                }
            }
        }
        $i = $i + 1
    }
    if !$ok {
        Err($err)
    } else if $input == "" {
        Err("usage: app.roc [--] <file.rocci|-> [-o <file.roc|->]")
    } else {
        Ok({ input: $input, output: $output })
    }
}

os_utf8 = |arg|
    match OsStr.to_raw(arg) {
        Utf8(s) => Ok(s)
        UnixBytes(bytes) => Ok(Str.from_utf8_lossy(bytes))
        _ => Err({})
    }

input_name = |path|
    if path == "-" {
        "stdin.rocci"
    } else {
        path
    }

read_src! = |path| {
    if path == "-" {
        read_stdin_all!({})
    } else {
        Path.read_utf8!(Path.utf8(path))
    }
}

read_stdin_all! = |_| {
    var $acc = ""
    var $loop = Bool.True
    while $loop {
        match Stdin.line!() {
            Ok(line) => {
                if $acc == "" {
                    $acc = line
                } else {
                    $acc = Str.concat($acc, Str.concat("\n", line))
                }
            }
            Err(_) => {
                $loop = Bool.False
            }
        }
    }
    Ok($acc)
}

write_out! = |output, roc| {
    if output == "" or output == "-" {
        Stdout.write!(roc)
    } else {
        Path.write_utf8!(Path.utf8(output), roc)
    }
}

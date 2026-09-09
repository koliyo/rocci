use crate::abi::roc_host;
use crate::roc_platform_abi::*;
use rocci_template::{compile, format_ast, parse, Diagnostic, LowerOptions, SourceFile};

fn diagnostics_list(
    diags: &[Diagnostic],
    roc_host: &RocHost,
) -> RocList<HostRocciCompileDiagnostics> {
    let diagnostics =
        unsafe { RocList::<HostRocciCompileDiagnostics>::allocate(diags.len(), roc_host) };
    for (index, diag) in diags.iter().enumerate() {
        let item = HostRocciCompileDiagnostics {
            end: u64::from(diag.span.end),
            start: u64::from(diag.span.start),
            code: RocStr::from_str(diag.code.unwrap_or(""), roc_host),
            message: RocStr::from_str(&diag.message, roc_host),
        };
        unsafe {
            diagnostics.elements.add(index).write(item);
        }
    }
    diagnostics
}

#[no_mangle]
pub extern "C" fn hosted_rocci_compile(args: HostRocciCompileArgs) -> HostRocciCompile {
    let roc_host = roc_host();
    let name = args.name.as_str().to_owned();
    let source = args.source.as_str().to_owned();
    unsafe { args.decref(roc_host) };

    let output = compile(SourceFile::new(&name, &source), &LowerOptions::default());
    HostRocciCompile {
        diagnostics: diagnostics_list(&output.diagnostics, roc_host),
        roc: RocStr::from_str(&output.roc, roc_host),
    }
}

#[no_mangle]
pub extern "C" fn hosted_rocci_parse(args: HostRocciParseArgs) -> HostRocciParse {
    let roc_host = roc_host();
    let name = args.name.as_str().to_owned();
    let source = args.source.as_str().to_owned();
    unsafe { args.decref(roc_host) };

    let parsed = parse(SourceFile::new(&name, &source));
    HostRocciParse {
        ast: RocStr::from_str(&format_ast(&source, &parsed.document), roc_host),
        diagnostics: diagnostics_list(&parsed.diagnostics, roc_host),
    }
}

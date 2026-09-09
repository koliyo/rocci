use crate::abi::roc_host;
use crate::roc_platform_abi::*;
use rocci_template::{compile, LowerOptions, SourceFile};

#[no_mangle]
pub extern "C" fn hosted_rocci_compile(args: HostRocciCompileArgs) -> HostRocciCompile {
    let roc_host = roc_host();
    let name = args.name.as_str().to_owned();
    let source = args.source.as_str().to_owned();
    unsafe { args.decref(roc_host) };

    let output = compile(SourceFile::new(&name, &source), &LowerOptions::default());
    let diagnostics = unsafe {
        RocList::<HostRocciCompileDiagnostics>::allocate(output.diagnostics.len(), roc_host)
    };
    for (index, diag) in output.diagnostics.iter().enumerate() {
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
    HostRocciCompile {
        diagnostics,
        roc: RocStr::from_str(&output.roc, roc_host),
    }
}

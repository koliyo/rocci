## String helpers for method-role dispatch. This module must stay platform-free:
## `import pf.Server` in an authored sibling currently SIGSEGVs `roc` for this app.

Rocci := [].{
    slash_alternate = alt_slash
    prefix_remainder = remainder_after_prefix
}

alt_slash = |path|
    if path == "/" {
        Err({})
    } else if Str.ends_with(path, "/") {
        Ok(Str.from_utf8_lossy(List.drop_last(Str.to_utf8(path), 1)))
    } else {
        Ok("${path}/")
    }

remainder_after_prefix = |path, prefix|
    if Str.starts_with(path, prefix) {
        Ok(Str.drop_prefix(path, prefix))
    } else {
        Err({})
    }

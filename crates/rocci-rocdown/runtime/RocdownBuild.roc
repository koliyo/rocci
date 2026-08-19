import pf.Env
import pf.Path
import Html
import RocdownTheme
import DocsComponents
import RocdownPages

render_tree! = |segments, index| {
    seg = List.get(segments, index)?
    if seg.tag == "html" {
        html = Path.utf8(seg.path).read_utf8!()?
        Ok((Html.dangerously_include_unescaped_html(html), index + 1))
    } else {
        (body, after) = render_children!(segments, index + 1, seg.child_count)?
        Ok((DocsComponents.render(seg, body), after))
    }
}

render_children! = |segments, index, remaining|
    if remaining == 0 {
        Ok((Html.empty, index))
    } else {
        (node, after) = render_tree!(segments, index)?
        (rest, end) = render_children!(segments, after, remaining - 1)?
        Ok((Html.fragment([node, rest]), end))
    }

render_forest! = |segments, index|
    match List.get(segments, index) {
        Err(_) => Ok(Html.empty)
        Ok(_) => {
            (node, after) = render_tree!(segments, index)?
            rest = render_forest!(segments, after)?
            Ok(Html.fragment([node, rest]))
        }
    }

write_page! = |staging, item| {
    content = render_forest!(item.segments, 0)?
    html = Html.render_document(
        RocdownTheme.siteShell(
            item.view,
            content,
        ),
    )
    Path.utf8("${staging}/${item.output_path}").write_utf8!(html)?
    Ok({})
}

write_all! = |staging, pages|
    match pages {
        [] => Ok({})
        [page, .. as rest] => {
            write_page!(staging, page)?
            write_all!(staging, rest)
        }
    }

RocdownBuild := [].{
    run! = |{}| {
        staging = Env.var_str!("ROCDOWN_STAGING")?
        write_all!(staging, RocdownPages.pages)
    }
}

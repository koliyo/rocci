import pf.Env
import pf.Path
import Html
import RocdownTheme
import DocsComponents
import RocdownPages

render_tree! = |segments, index| {
    match List.get(segments, index)? {
        HtmlFile(seg) => {
            html = Path.utf8(seg.path).read_utf8!()?
            Ok((Html.dangerously_include_unescaped_html(html), index + 1))
        }
        Note(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.note({ title: seg.title }, body), after))
        }
        Tip(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.tip({ title: seg.title }, body), after))
        }
        Caution(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.caution({ title: seg.title }, body), after))
        }
        Danger(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.danger({ title: seg.title }, body), after))
        }
        Deprecated(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.deprecated({ title: seg.title }, body), after))
        }
        Details(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.details({ summary: seg.summary, open: seg.open }, body), after))
        }
        Steps(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.steps({}, body), after))
        }
        Step(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.step({ title: seg.title, verify: seg.verify }, body), after))
        }
        Figure(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.figure({ caption: seg.caption, credit: seg.credit }, body), after))
        }
        Definition(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.definition({ title: seg.title }, body), after))
        }
        Tabs(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.tabs({}, body), after))
        }
        Tab(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.tab({ label: seg.label }, body), after))
        }
        Badge(seg) =>
            Ok((DocsComponents.badge({ label: seg.label }), index + 1))
        LinkCard(seg) =>
            Ok((DocsComponents.linkCard({ href: seg.href, title: seg.title, summary: seg.summary }), index + 1))
        CardGrid(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.cardGrid({}, body), after))
        }
        FileTree(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.fileTree({}, body), after))
        }
        Compatibility(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.compatibility({ caption: seg.caption }, body), after))
        }
        Example(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.example({}, body), after))
        }
        Include(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((DocsComponents.include({}, body), after))
        }
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

ensure_parent! = |dest| {
    parts = Str.split_on(dest, "/")
    len = List.len(parts)
    if len <= 1 {
        Ok({})
    } else {
        parent = Str.join_with(List.drop_last(parts, 1), "/")
        Path.utf8(parent).create_all!()?
        Ok({})
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
    dest = "${staging}/${item.output_path}"
    ensure_parent!(dest)?
    Path.utf8(dest).write_utf8!(html)?
    Ok({})
}

write_all! = |staging, pages| {
    for page in pages {
        write_page!(staging, page)?
    }
    Ok({})
}

RocdownBuild := [].{
    run! = |{}| {
        staging = Env.var_str!("ROCDOWN_STAGING")?
        write_all!(staging, RocdownPages.pages)
    }
}

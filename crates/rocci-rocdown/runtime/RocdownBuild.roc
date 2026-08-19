import pf.Env
import pf.Path
import Html
import RocdownTheme
import BlockPainters
import RocdownPages

render_tree! = |segments, index| {
    match List.get(segments, index)? {
        HtmlFile(seg) => {
            html = Path.utf8(seg.path).read_utf8!()?
            Ok((Html.dangerously_include_unescaped_html(html), index + 1))
        }
        Note(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.note({ title: seg.title }, body), after))
        }
        Tip(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.tip({ title: seg.title }, body), after))
        }
        Caution(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.caution({ title: seg.title }, body), after))
        }
        Danger(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.danger({ title: seg.title }, body), after))
        }
        Deprecated(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.deprecated({ title: seg.title }, body), after))
        }
        Details(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.details({ summary: seg.summary, open: seg.open }, body), after))
        }
        Steps(seg) => {
            (items, after) = render_step_items!(segments, index + 1, seg.child_count)?
            body = html_from_records(items)
            Ok((BlockPainters.steps({}, body), after))
        }
        Step(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.step({ title: seg.title, verify: seg.verify }, body), after))
        }
        Figure(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.figure({ caption: seg.caption, credit: seg.credit }, body), after))
        }
        Definition(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.definition({ title: seg.title }, body), after))
        }
        Tabs(seg) => {
            (items, after) = render_tab_items!(segments, index + 1, seg.child_count)?
            body = html_from_records(items)
            Ok((BlockPainters.tabs({ group: seg.group, kind: seg.kind }, body), after))
        }
        Tab(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.tab({ id: seg.id, label: seg.label }, body), after))
        }
        Badge(seg) =>
            Ok((BlockPainters.badge({ label: seg.label }), index + 1))
        LinkCard(seg) =>
            Ok((BlockPainters.linkCard({ href: seg.href, title: seg.title, summary: seg.summary }), index + 1))
        CardGrid(seg) => {
            (items, after) = render_card_items!(segments, index + 1, seg.child_count)?
            body = html_from_records(items)
            Ok((BlockPainters.cardGrid({}, body), after))
        }
        FileTree(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.fileTree({}, body), after))
        }
        Compatibility(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.compatibility({ caption: seg.caption }, body), after))
        }
        Example(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.example({}, body), after))
        }
        Include(seg) => {
            (body, after) = render_children!(segments, index + 1, seg.child_count)?
            Ok((BlockPainters.include({}, body), after))
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

html_from_records = |items|
    Html.fragment(List.map(items, |item| item.content))

render_tab_items! = |segments, index, remaining|
    if remaining == 0 {
        Ok(([], index))
    } else {
        match List.get(segments, index)? {
            Tab(seg) => {
                (body, after) = render_children!(segments, index + 1, seg.child_count)?
                item = {
                    id: seg.id,
                    label: seg.label,
                    content: BlockPainters.tab({ id: seg.id, label: seg.label }, body),
                }
                (rest, end) = render_tab_items!(segments, after, remaining - 1)?
                Ok((List.prepend(rest, item), end))
            }
            _ => {
                (node, after) = render_tree!(segments, index)?
                item = { id: "", label: "", content: node }
                (rest, end) = render_tab_items!(segments, after, remaining - 1)?
                Ok((List.prepend(rest, item), end))
            }
        }
    }

render_step_items! = |segments, index, remaining|
    if remaining == 0 {
        Ok(([], index))
    } else {
        match List.get(segments, index)? {
            Step(seg) => {
                (body, after) = render_children!(segments, index + 1, seg.child_count)?
                item = {
                    title: seg.title,
                    verify: seg.verify,
                    content: BlockPainters.step({ title: seg.title, verify: seg.verify }, body),
                }
                (rest, end) = render_step_items!(segments, after, remaining - 1)?
                Ok((List.prepend(rest, item), end))
            }
            _ => {
                (node, after) = render_tree!(segments, index)?
                item = { title: "", verify: False, content: node }
                (rest, end) = render_step_items!(segments, after, remaining - 1)?
                Ok((List.prepend(rest, item), end))
            }
        }
    }

render_card_items! = |segments, index, remaining|
    if remaining == 0 {
        Ok(([], index))
    } else {
        match List.get(segments, index)? {
            LinkCard(seg) => {
                item = {
                    href: seg.href,
                    title: seg.title,
                    summary: seg.summary,
                    content: BlockPainters.linkCard({ href: seg.href, title: seg.title, summary: seg.summary }),
                }
                (rest, end) = render_card_items!(segments, index + 1, remaining - 1)?
                Ok((List.prepend(rest, item), end))
            }
            _ => {
                (node, after) = render_tree!(segments, index)?
                item = { href: "", title: "", summary: "", content: node }
                (rest, end) = render_card_items!(segments, after, remaining - 1)?
                Ok((List.prepend(rest, item), end))
            }
        }
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

import pf.Env
import pf.Path
import Html
import RocsTheme
import RocsPages

write_page! = |staging, item| {
    article = Path.utf8(item.article_path).read_utf8!()?
    html = Html.render_document(
        RocsTheme.siteShell(
            {
                site: RocsPages.site,
                nav: RocsPages.nav,
                route: item.route,
                title: item.title,
                description: item.description,
                outline: item.outline,
                breadcrumbs: item.breadcrumbs,
                previous: item.previous,
                next: item.next,
            },
            Html.dangerously_include_unescaped_html(article),
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

RocsBuild := [].{
    run! = |{}| {
        staging = Env.var_str!("ROCS_STAGING")?
        write_all!(staging, RocsPages.pages)
    }
}

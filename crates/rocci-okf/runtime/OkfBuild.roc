import pf.Env
import pf.Path
import Html
import OkfTheme
import OkfPages

write_page! = |staging, item| {
    article_html = Path.utf8(item.article_path).read_utf8!()?
    article_node = Html.dangerously_include_unescaped_html(article_html)
    page_html = Html.render(
        OkfTheme.knowledgeShell(
            item.view,
            article_node,
        ),
    )
    full_html = "<!doctype html><html lang=\"en\" class=\"rd-document\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"color-scheme\" content=\"dark\"><title>${item.title}</title><link rel=\"stylesheet\" href=\"/__rocci_okf/app.css\"><script src=\"/__rocci_okf/reload.js\" defer></script></head><body>${page_html}</body></html>\n"
    Path.utf8("${staging}/${item.output_path}").write_utf8!(full_html)?
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

OkfBuild := [].{
    run! = |{}| {
        staging = Env.var_str!("OKF_STAGING")?
        write_all!(staging, OkfPages.pages)
    }
}

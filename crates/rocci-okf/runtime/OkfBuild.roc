import Html
import OkfTheme
import OkfPages

OkfBuild := [].{
    render_page = |item| {
        article_node = Html.dangerously_include_unescaped_html(item.article_html)
        page_html = Html.render(
            OkfTheme.knowledgeShell(
                item.view,
                article_node,
            ),
        )
        "<!doctype html><html lang=\"en\" class=\"rd-document\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"color-scheme\" content=\"dark\"><title>${item.title}</title><link rel=\"stylesheet\" href=\"/__rocci_okf/app.css\"><script src=\"/__rocci_okf/reload.js\" defer></script></head><body>${page_html}</body></html>\n"
    }

    render_all = |pages|
        List.map(pages, OkfBuild.render_page)
}

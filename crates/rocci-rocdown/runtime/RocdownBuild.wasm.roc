import Html
import RocdownTheme
import RocdownPages

render_page = |item|
    Html.render_document(
        RocdownTheme.siteShell(
            item.view,
            Html.empty,
        ),
    )

render_all = |pages|
    List.map(pages, render_page)

RocdownBuild := [].{
    render_page = render_page
    render_all = render_all
}

import Html
import RocdownTheme
import RocdownPages
import Views

render_page : Views.Page(_) -> Str
render_page = |item|
    Html.render_document(
        RocdownTheme.siteShell(
            item.view,
            Html.empty,
        ),
    )

render_all : List(Views.Page(_)) -> _
render_all = |pages|
    List.map(pages, render_page)

RocdownBuild := [].{
    render_page = render_page
    render_all = render_all
}

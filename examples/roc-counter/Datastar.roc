import pf.Html as PlatformHtml
import pf.Sse

Datastar := [].{
    patch_elements = |node|
        Sse.Event.keyed(
            "datastar-patch-elements",
            "elements",
            PlatformHtml.render_without_doc_type(node),
        )
}

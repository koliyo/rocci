import pf.Attribute
import pf.Html as PlatformHtml

Html := [].{
    attribute = Attribute.attribute

    boolean_attribute = |name, enabled|
        if enabled {
            Attribute.attribute(name, "")
        } else {
            Attribute.attribute(name, "")
        }

    dangerously_include_unescaped_html = PlatformHtml.dangerously_include_unescaped_html
    element = PlatformHtml.element
    empty = PlatformHtml.text("")
    render = PlatformHtml.render
    render_document = PlatformHtml.render_document
    render_fragment = PlatformHtml.render_fragment
    render_without_doc_type = PlatformHtml.render_without_doc_type
    text = PlatformHtml.text
    void_element = PlatformHtml.void_element

    fragment = |nodes|
        PlatformHtml.dangerously_include_unescaped_html(
            PlatformHtml.render_fragment(nodes).to_str(),
        )
}

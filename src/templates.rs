use askama::Template;

#[derive(Template)]
#[template(path = "datastar.html", blocks = ["counter"])]
struct DatastarPage {
    count: u64,
}

#[derive(Template)]
#[template(path = "htmx.html", blocks = ["counter"])]
struct HtmxPage {
    count: u64,
}

pub(crate) fn datastar_page(count: u64) -> String {
    render(DatastarPage { count })
}

pub(crate) fn datastar_counter(count: u64) -> String {
    render(DatastarPage { count }.as_counter())
}

pub(crate) fn htmx_page(count: u64) -> String {
    render(HtmxPage { count })
}

pub(crate) fn htmx_counter(count: u64) -> String {
    render(HtmxPage { count }.as_counter())
}

fn render(template: impl Template) -> String {
    // These templates only write to a String and contain no fallible filters.
    // Their structure and field access are checked by Askama at compile time.
    template
        .render()
        .expect("rendering a compile-time checked template into String failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datastar_counter_is_the_same_block_used_by_the_page() {
        let fragment = datastar_counter(42);
        assert!(fragment.contains("<output>42</output>"));
        assert!(datastar_page(42).contains(&fragment));
    }

    #[test]
    fn htmx_counter_is_the_same_block_used_by_the_page() {
        let fragment = htmx_counter(42);
        assert_eq!(fragment, r#"<output id="htmx-counter">42</output>"#);
        assert!(htmx_page(42).contains(&fragment));
    }
}

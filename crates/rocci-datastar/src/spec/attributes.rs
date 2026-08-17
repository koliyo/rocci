#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttributeSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub syntax_example: &'static str,
    pub supports_custom_key: bool,
    pub supports_modifiers: bool,
}

pub const ATTRIBUTES: &[AttributeSpec] = &[
    AttributeSpec {
        name: "data-bind",
        description: "Two-way bind a signal to a form input, select, or textarea value.",
        syntax_example: r#"data-bind="searchQuery""#,
        supports_custom_key: true,
        supports_modifiers: true,
    },
    AttributeSpec {
        name: "data-signals",
        description: "Declare or initialize reactive client signals.",
        syntax_example: r#"data-signals="{ count: 0, query: '' }""#,
        supports_custom_key: true,
        supports_modifiers: true,
    },
    AttributeSpec {
        name: "data-computed",
        description: "Define a computed reactive signal derived from other signals.",
        syntax_example: r#"data-computed:doubled="$count * 2""#,
        supports_custom_key: true,
        supports_modifiers: false,
    },
    AttributeSpec {
        name: "data-ref",
        description: "Store a reference to this DOM element in a named signal.",
        syntax_example: r#"data-ref="myInput""#,
        supports_custom_key: true,
        supports_modifiers: false,
    },
    AttributeSpec {
        name: "data-text",
        description: "Set the textContent of the element to the evaluated expression.",
        syntax_example: r#"data-text="$count""#,
        supports_custom_key: false,
        supports_modifiers: false,
    },
    AttributeSpec {
        name: "data-show",
        description: "Toggle element visibility (display: none) based on a boolean expression.",
        syntax_example: r#"data-show="$isOpen""#,
        supports_custom_key: false,
        supports_modifiers: false,
    },
    AttributeSpec {
        name: "data-class",
        description: "Conditionally toggle one or more CSS classes.",
        syntax_example: r#"data-class:active="$isActive""#,
        supports_custom_key: true,
        supports_modifiers: false,
    },
    AttributeSpec {
        name: "data-on",
        description: "Attach an event listener to the element and execute an expression.",
        syntax_example: r#"data-on:click="@get('/api/items')""#,
        supports_custom_key: true,
        supports_modifiers: true,
    },
    AttributeSpec {
        name: "data-indicator",
        description: "Set a boolean loading signal while a backend request is in-flight.",
        syntax_example: r#"data-indicator="isLoading""#,
        supports_custom_key: true,
        supports_modifiers: false,
    },
    AttributeSpec {
        name: "data-persist",
        description: "Persist signal values across browser reloads via localStorage.",
        syntax_example: r#"data-persist="theme""#,
        supports_custom_key: true,
        supports_modifiers: false,
    },
    AttributeSpec {
        name: "data-replace-url",
        description: "Dynamically update the browser URL based on an expression.",
        syntax_example: r#"data-replace-url="`/search?q=${$query}`""#,
        supports_custom_key: false,
        supports_modifiers: false,
    },
    AttributeSpec {
        name: "data-scroll-into-view",
        description: "Scroll the element into view when the expression evaluates to true.",
        syntax_example: r#"data-scroll-into-view="$isTarget""#,
        supports_custom_key: false,
        supports_modifiers: true,
    },
    AttributeSpec {
        name: "data-view-transition",
        description: "Opt element changes into the browser View Transition API.",
        syntax_example: "data-view-transition",
        supports_custom_key: false,
        supports_modifiers: false,
    },
];

pub fn lookup_attribute(name: &str) -> Option<&'static AttributeSpec> {
    let base = name.split([':', '_']).next().unwrap_or(name);
    ATTRIBUTES.iter().find(|attr| attr.name == base)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModifierSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub takes_arg: bool,
}

pub const MODIFIERS: &[ModifierSpec] = &[
    ModifierSpec {
        name: "debounce",
        description: "Delay execution until a specified quiet duration elapses (e.g. __debounce.500ms).",
        takes_arg: true,
    },
    ModifierSpec {
        name: "throttle",
        description: "Limit execution frequency to once per specified duration (e.g. __throttle.1s).",
        takes_arg: true,
    },
    ModifierSpec {
        name: "passive",
        description: "Add event listener with { passive: true } for smooth scrolling performance.",
        takes_arg: false,
    },
    ModifierSpec {
        name: "capture",
        description: "Add event listener in the capture phase instead of bubbling.",
        takes_arg: false,
    },
    ModifierSpec {
        name: "once",
        description: "Trigger the event handler at most once.",
        takes_arg: false,
    },
    ModifierSpec {
        name: "window",
        description: "Attach the event listener to the global window object.",
        takes_arg: false,
    },
    ModifierSpec {
        name: "outside",
        description: "Trigger when a click/interaction occurs outside this element.",
        takes_arg: false,
    },
    ModifierSpec {
        name: "prevent",
        description: "Call event.preventDefault() automatically before running expression.",
        takes_arg: false,
    },
    ModifierSpec {
        name: "stop",
        description: "Call event.stopPropagation() automatically before running expression.",
        takes_arg: false,
    },
    ModifierSpec {
        name: "self",
        description: "Only fire if event.target is the element itself.",
        takes_arg: false,
    },
];

pub fn lookup_modifier(name: &str) -> Option<&'static ModifierSpec> {
    MODIFIERS.iter().find(|m| m.name == name)
}

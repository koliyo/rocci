#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub signature: &'static str,
}

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "@get",
        description: "Send an HTTP GET request to backend; morphs DOM with received Datastar SSE events.",
        signature: "@get(url, [options])",
    },
    ActionSpec {
        name: "@post",
        description: "Send an HTTP POST request with current signals to backend.",
        signature: "@post(url, [options])",
    },
    ActionSpec {
        name: "@put",
        description: "Send an HTTP PUT request with current signals to backend.",
        signature: "@put(url, [options])",
    },
    ActionSpec {
        name: "@patch",
        description: "Send an HTTP PATCH request with current signals to backend.",
        signature: "@patch(url, [options])",
    },
    ActionSpec {
        name: "@delete",
        description: "Send an HTTP DELETE request to backend.",
        signature: "@delete(url, [options])",
    },
    ActionSpec {
        name: "@clipboard",
        description: "Write text or signal expression value to system clipboard.",
        signature: "@clipboard(text)",
    },
    ActionSpec {
        name: "@fit",
        description: "Fit text content dynamically to fill container dimensions.",
        signature: "@fit()",
    },
];

pub fn lookup_action(name: &str) -> Option<&'static ActionSpec> {
    ACTIONS.iter().find(|a| a.name == name)
}

module CounterPage exposing [counterPage]

import pf.Html
import Design
import Datastar

Tone : [Neutral, Positive]



helloSample = { name: "Roc" }


badge = |{ tone }, content| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            Html.element(
                "span",
                [
                    Html.attribute("class", badgeClass(tone)),
                    Html.attribute("data-rocci-css", "AllSyntax-02b8f884"),
                ],
                [
                    content,
                ],
            ),
        ],
    )
}


hello = |{ name }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            Html.element(
                "p",
                [
                    Html.attribute("data-rocci-css", "AllSyntax-02b8f884"),
                ],
                [
                    Html.text("Hello, "),
                    Html.text(name),
                ],
            ),
        ],
    )
}


counterCard = |{ count }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            Html.element(
                "section",
                [
                    Html.attribute("id", "counter"),
                    Html.attribute("class", "counter-card"),
                    Html.attribute("data-rocci-css", "AllSyntax-02b8f884"),
                ],
                [
                    Html.element(
                        "output",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-02b8f884"),
                        ],
                        [
                            Html.text(Num.toStr(count)),
                        ],
                    ),
                    badge(
                        { tone: Positive },
                        Html.text("Current count"),
                    ),
                ],
            ),
        ],
    )
}


counterPage = |{ person, count }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            Html.element(
                "main",
                [
                    Html.attribute("id", "counter-page"),
                    Html.attribute("data-rocci-css", "AllSyntax-02b8f884"),
                ],
                [
                    hello(
                        { name: "World" },
                    ),
                    hello(
                        { name: person.name },
                    ),
                    counterCard(
                        { count: count },
                    ),
                ],
            ),
        ],
    )
}


todoList = |{ items, state }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            match state {
                Loading => spinner(
                    {},
                )
                Failed(message) => errorNotice(
                    { message: message },
                )
                Ready => if List.isEmpty(items) {
                    emptyState(
                        {},
                    )
                } else {
                    Html.element(
                        "ul",
                        [
                            Html.attribute("data-rocci-css", "AllSyntax-02b8f884"),
                        ],
                        List.map(items, |item| {
                            todoRow(
                                { item: item },
                            )
                        }),
                    )
                }
            },
        ],
    )
}


accountActions = |{ user }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            if user.isSignedIn {
                logoutButton(
                    {},
                )
            } else if user.canRegister {
                registerButton(
                    {},
                )
            } else {
                loginButton(
                    {},
                )
            },
        ],
    )
}


filteredList = |{ items, query }| {
    visible = List.keepIf(items, |item| matches(item, query))

    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            if List.isEmpty(visible) {
                emptyState(
                    { query: query },
                )
            } else {
                itemList(
                    { items: visible },
                )
            },
        ],
    )
}


requestState = |{ state, active }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            match state {
                Loading => spinner(
                    {},
                )
                Failed({ message }) => errorNotice(
                    { message: message },
                )
                Ready(items) if !List.isEmpty(items) => itemList(
                    { items: items },
                )
                Ready(_) => Html.fragment(
                    [
                        heading(
                            { text: "Ready" },
                        ),
                        Design.button(
                            { tone: if active { Positive } else { Neutral } },
                        ),
                    ],
                )
            },
        ],
    )
}


profileLink = |{ person, selected }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            Html.element(
                "a",
                [
                    Html.attribute("href", person.url),
                    Html.attribute("class", if selected { "selected" } else { "" }),
                    Html.attribute("aria-current", if selected { "page" } else { "false" }),
                    Html.attribute("data-rocci-css", "AllSyntax-02b8f884"),
                ],
                [
                    Html.text(person.name),
                ],
            ),
        ],
    )
}


visibleCheck = |{ user, permissions }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            if isVisible({ user, permissions }) {
                profile(
                    {},
                )
            } else {
                Html.element(
                    "p",
                    [
                        Html.attribute("data-rocci-css", "AllSyntax-02b8f884"),
                    ],
                    [],
                )
            },
        ],
    )
}


statusMatch = |{ pair }| {
    Html.fragment(
        [
            Html.element(
                "style",
                [],
                [
                    Html.text("@scope ([data-rocci-css~=\"AllSyntax-02b8f884\"]) {\nbody { font-family: system-ui, sans-serif; }\n}"),
                ],
            ),
            match ({ status, items }) {
                { status: Loading } => spinner(
                    {},
                )
                { status: Ready } => itemList(
                    { items: items },
                )
            },
        ],
    )
}


badgeClass = |tone| {
    match tone {
        Neutral => "badge"
        Positive => "badge badge--positive"
    }
}

on_get_root! = |state, _request| {
    rocci_value = {
        counterPage({ person: { name: "Roc" }, count: 0 })
    }
    Ok(rocci_value)
}


live! = |state, _request| {
    rocci_value = {
        counterPage({ person: { name: "Roc" }, count: 0 })
    }
    Ok(rocci_value)
}


on_post_actions_increment! = |state, _request| {
    rocci_value = {
        "{\"count\": 0}"
    }
    Ok(rocci_value)
}


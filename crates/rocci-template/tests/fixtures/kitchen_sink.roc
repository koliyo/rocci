module CounterPage exposing [counterPage]

import pf.Html
import Design

Tone : [Neutral, Positive]

badge = |{ tone ?? Neutral }, content| {
    Html.element(
        "span",
        [
            Html.attribute("class", badgeClass(tone)),
        ],
        [
            content,
        ],
    )
}


hello = |{ name ?? "World" }| {
    Html.element(
        "p",
        [],
        [
            Html.text("Hello, "),
            Html.text(name),
        ],
    )
}


counterCard = |{ count }| {
    Html.element(
        "section",
        [
            Html.attribute("id", "counter"),
            Html.attribute("class", "counter-card"),
        ],
        [
            Html.element(
                "output",
                [],
                [
                    Html.text(Num.toStr(count)),
                ],
            ),
            badge(
                { tone: Positive },
                Html.text("Current count"),
            ),
        ],
    )
}


counterPage = |{ person, count }| {
    Html.element(
        "main",
        [
            Html.attribute("id", "counter-page"),
        ],
        [
            hello(
                {},
            ),
            hello(
                { name: person.name },
            ),
            counterCard(
                { count: count },
            ),
        ],
    )
}


todoList = |{ items, state }| {
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
                [],
                List.map(items, |item| {
                    todoRow(
                        { item: item },
                    )
                }),
            )
        }
    }
}


accountActions = |{ user }| {
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
    }
}


filteredList = |{ items, query }| {
    visible = List.keepIf(items, |item| matches(item, query))

    if List.isEmpty(visible) {
        emptyState(
            { query: query },
        )
    } else {
        itemList(
            { items: visible },
        )
    }
}


requestState = |{ state, active }| {
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
    }
}


profileLink = |{ person, selected }| {
    Html.element(
        "a",
        [
            Html.attribute("href", person.url),
            Html.attribute("class", if selected { "selected" } else { "" }),
            Html.attribute("aria-current", if selected { "page" } else { "false" }),
        ],
        [
            Html.text(person.name),
        ],
    )
}


visibleCheck = |{ user, permissions }| {
    if isVisible({ user, permissions }) {
        profile(
            {},
        )
    } else {
        Html.element(
            "p",
            [],
            [],
        )
    }
}


statusMatch = |{ pair }| {
    match ({ status, items }) {
        { status: Loading } => spinner(
            {},
        )
        { status: Ready } => itemList(
            { items: items },
        )
    }
}


badgeClass = |tone| {
    match tone {
        Neutral => "badge"
        Positive => "badge badge--positive"
    }
}

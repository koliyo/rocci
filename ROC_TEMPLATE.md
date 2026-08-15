# Rocci template language

**Investigation date:** 2026-08-13  
**Status:** Proposed language and compiler-package design; examples are illustrative, not a committed grammar.

## Purpose

This document defines the proposed template surface for `.rocci` files and the package boundary which implements it. Runtime hosting, HTTP/session security, Datastar transport, state ownership, and Roc platform integration belong in the broader [Roc + Datastar component report](ROC_DATASTAR_COMPONENT_FILETYPE_REPORT.md).

The central design goals are:

- allow several render components in one file;
- keep component rendering pure and compile it to ordinary Roc;
- compose components with HTML-like tags;
- use Roc expressions and structurally typed records for props;
- escape dynamic content according to its HTML context;
- preserve exact source locations for Roc and template diagnostics;
- keep the template compiler independent of HTTP, Datastar, routes, state, and the Rocci application runtime.

## Recommended source shape

A `.rocci` file is a Roc module containing ordinary Roc declarations and explicit component declarations:

```rocci
module CounterPage exposing [counterPage]

import pf.Html

Tone : [Neutral, Positive]

@component Badge = |{ tone }, content| {
    <span class={badgeClass(tone)}>
        {content}
    </span>
}

@component Hello = |{ name }| {
    <p>Hello, {name}</p>
}

@component CounterCard = |{ count }| {
    <section id="counter" class="counter-card">
        <output>{Num.toStr(count)}</output>
        <Badge tone={Positive}>Current count</Badge>
    </section>
}

@component CounterPage = |{ person, count }| {
    <main id="counter-page">
        <Hello name={person.name} />
        <CounterCard count={count} />
    </main>
}

badgeClass = |tone| {
    match tone {
        Neutral => "badge"
        Positive => "badge badge--positive"
    }
}
```

The file is a module, not a component instance. Small components can remain next to the page that uses them. A component should move to another file only when module ownership or reuse makes that clearer, not because the format permits one template per file. The later JSX section remains an analyzed alternative rather than the recommended v1 grammar.

## Why the templ-style declaration

[templ](https://templ.guide/) places ordinary Go declarations and multiple HTML-producing component declarations in one source file, then compiles the latter to Go. That is the useful model for Rocci. Rocci should adapt the model to Roc rather than copy templ's call syntax.

templ composition:

```templ
templ Hello(name string) {
    <div>Hello, { name }</div>
}

templ Greeting(person Person) {
    <div class="greeting">
        @Hello(person.Name)
    </div>
}
```

Proposed Rocci composition:

```rocci
@component hello = |{ name }| {
    <div>Hello, {name}</div>
}

@component greeting = |{ person }| {
    <div class="greeting">
        <Hello name={person.name} />
    </div>
}
```

The user's example therefore becomes the complete HTML-like call:

```html
<Hello name={person.name} />
```

It lowers approximately to:

```roc
hello({ name: person.name })
```

Named HTML attributes map naturally to a Roc props record. Roc record destructuring gives the declaration named inputs, order independence, structural typing, and useful inferred constraints without a component class or runtime props object.

## Alternative: JSX-style HTML expressions in ordinary Roc

The `component` marker is not required for the runtime or type model. A more integrated design treats HTML markup as another Roc expression, analogous to JSX being an XML-like expression syntax which TypeScript parses, type-checks, and lowers to ordinary calls. “JSX” here describes source grammar only: it does not imply React, a browser-side Roc runtime, hydration, hooks, or a virtual DOM.

### Surface syntax

Every render component becomes a normal Roc function:

```rocci
hello : { name : Str } -> Html
hello = |{ name }|
    <p>Hello, {name}</p>

greeting = |{ person }|
    <div class="greeting">
        <Hello name={person.name} />
    </div>
```

There is no `component` declaration form, component registry, decorator, or lifecycle. A value is usable as a component when its inferred Roc type is compatible with the arguments produced by a tag.

The functions remain callable as ordinary Roc:

```roc
html = hello({ name: "Ada" })
```

and testable without a template-specific harness:

```roc
expect
    Html.render(hello({ name: "Ada" })) == "<p>Hello, Ada</p>"
```

### Normal Roc flow

The strongest reason to prefer JSX-style expressions is that `if`, `match`, local definitions, and higher-order functions remain Roc rather than becoming template directives.

```rocci
statusView = |status| {
    match status {
        Loading =>
            <p class="loading">Loading…</p>

        Failed(message) =>
            <ErrorNotice message={message} />

        Ready(items) =>
            <ul>
                {List.map(items, |item|
                    <ItemRow item={item} />
                )}
            </ul>
    }
}
```

Ordinary local definitions work as expected:

```rocci
userList = |{ users, selectedId }|
    row = |user|
        isSelected = user.id == selectedId

        <li class={if isSelected { "selected" } else { "" }}>
            <UserLink user={user} />
        </li>

    <ul>{List.map(users, row)}</ul>
```

Conditional markup inside a parent is just a Roc expression in braces:

```rocci
toolbar = |{ user }|
    <nav>
        <HomeLink />
        {if user.isAdmin { <AdminLink /> } else { Html.empty }}
    </nav>
```

This last example is the important threshold for “full JSX.” If markup is allowed only as a function's final expression but not inside `if` branches, `match` branches, closures, lists, and `{...}` expressions, applications will be forced to extract many trivial helper functions. Full JSX requires recursive alternation between Roc expressions and markup expressions.

### Fragments and collections

A fragment form is needed when one Roc expression produces several sibling nodes:

```rocci
personSummary = |person|
    <>
        <dt>Name</dt>
        <dd>{person.name}</dd>
    </>
```

It lowers to `Html.fragment(...)`. A child expression may produce `Html` or `List Html`, allowing normal Roc collection functions:

```rocci
tableBody = |rows|
    <tbody>
        {List.map(rows, |row| <Row value={row} />)}
    </tbody>
```

The exact flattening contract must remain explicit: Rocci should not recursively accept arbitrary lists, strings, optional values, and tag unions through a broad runtime conversion protocol. Prefer a small statically known set such as `Html`, `List Html`, and text values accepted by `Html.text`.

### Function-call lowering

The earlier props and body model works unchanged because tags lower to ordinary function calls.

```rocci
card : { title : Str }, Html -> Html
card = |{ title }, body|
    <section>
        <h2>{title}</h2>
        {body}
    </section>

page = |{}|
    <Card title="Settings">
        <SettingsForm />
    </Card>
```

Conceptually:

```roc
card(
    { title: "Settings" },
    Html.fragment([settingsForm({})]),
)
```

The context-free tag rules can remain:

- a self-closing component tag produces a one-argument props-record call;
- a paired component tag produces a two-argument call whose second argument is `Html`;
- HTML attributes lower to typed element attributes rather than a component props record;
- named and scoped content remain ordinary `Html` and function-valued props as described later.

### Component-name resolution

JSX conventionally distinguishes intrinsic elements from components using capitalization. Roc values cannot begin with uppercase letters, so Rocci still needs a small source-level mapping:

- `<div>` is an intrinsic HTML element;
- `<UserCard>` resolves to `userCard`;
- `<Design.Button>` resolves to `Design.button`;
- `htmlShell` is written `<HtmlShell>`, not `<HTMLShell>`.

This is the only sense in which a normal function becomes a “component.” No declaration metadata is needed for local calls. The generated Roc type checker validates whether `userCard` accepts the generated argument record and optional body argument.

Possible alternatives avoid capitalization mapping but are less pleasant:

| Form | Example | Problem |
| --- | --- | --- |
| Lowercase function tags | `<userCard />` | Cannot reliably distinguish functions from intrinsic/custom HTML elements |
| Explicit expression tag | `<{userCard} />` | Unambiguous and supports higher-order components, but visually noisy and needs custom closing-tag rules |
| Universal call element | `<Component fn={userCard} />` | Ordinary and explicit, but loses the main readability benefit of JSX |
| Function calls only | `{userCard({ ... })}` | Requires no tag mapping, but composition no longer reads like HTML |

PascalCase-to-lower-camel mapping is the best default. An explicit expression-tag form can be researched later for genuinely dynamic component values.

### Three parser strategies

#### Strategy A: full Roc+JSX grammar

Accept a markup expression anywhere Roc accepts an expression:

```rocci
view = |model| {
    if model.ready {
        <Ready model={model} />
    } else {
        <Loading />
    }
}
```

This gives the cleanest language. It also requires the template front end to understand enough Roc grammar to know when `<` starts markup rather than an operator and when a nested Roc expression ends. Markup expressions can contain Roc expressions which contain further markup, so a regex or one-way block splitter is insufficient.

The robust implementations are either:

1. extend or reuse the pinned Roc parser with a markup-expression AST node; or
2. maintain a complete enough Roc+JSX parser in `rocci-template` and lower its output before invoking Roc.

The first has the best long-term diagnostics and formatting but needs upstream cooperation or a compiler fork/plugin surface which does not currently exist as a stable public contract. The second preserves external tooling but tightly couples `rocci-template` to Roc grammar changes.

#### Strategy B: explicit `html { ... }` expression islands

Keep view functions ordinary, but mark transitions into markup:

```rocci
hello = |{ name }|
    html {
        <p>Hello, {name}</p>
    }

statusView = |status| {
    match status {
        Loading => html { <Loading /> }
        Failed(message) => html { <ErrorNotice message={message} /> }
        Ready(items) => html {
            <ul>{List.map(items, itemRow)}</ul>
        }
    }
}
```

This eliminates the special component declaration while giving the external parser strong island boundaries. It composes with normal Roc control flow, but every branch returning inline markup needs an `html` marker. Nested markup inside a Roc expression still needs either another island or extraction to a normal function.

Advantages:

- substantially simpler scanning, recovery, and formatting;
- normal Roc functions, annotations, higher-order use, and tests;
- ordinary Roc outside clearly delimited markup ranges;
- a plausible implementation entirely inside `rocci-template`.

Costs:

- noisier than JSX;
- braces compete visually with Roc records and markup expression braces;
- not all inline examples are seamless;
- `html` is still language syntax even though it looks like a function.

This is the safest fallback if full JSX cannot be implemented without duplicating the Roc parser.

#### Strategy C: root-only implicit markup

Allow unmarked markup only as the final body of a function:

```rocci
hello = |{ name }|
    <p>Hello, {name}</p>
```

but require helpers or explicit islands in branches and closures. This is easier than full JSX, but the boundary is contextual and the limitation will surprise users precisely when views become interesting. It offers less implementation clarity than `html { ... }` and less expressive power than full JSX.

This should be rejected unless a spike shows a uniquely simple and reliable parse strategy.

### Ambiguities and recovery

Full JSX introduces parser problems which the explicit `component ... { HTML }` form avoids:

- `<` already participates in Roc comparisons, so tag recognition must depend on expression context and following tokens;
- `{...}` changes from markup to Roc, but the Roc expression may recursively contain markup;
- HTML text whitespace and Roc indentation have different significance;
- incomplete opening/closing tags must not consume the following Roc definitions;
- generic formatter edits must preserve both Roc layout and intended HTML text;
- `<Foo>` maps to a lowercase Roc value rather than an identically spelled identifier;
- diagnostics from generated constructors need finer segment maps than one template body.

A stateful lexer is necessary but not sufficient. For editor-quality recovery, the parser needs explicit modes (`Roc`, `Markup`, `Tag`, `Attribute`, and `RocExpressionInMarkup`) plus Roc-aware delimiter and indentation handling. The parsing spike should include incomplete code at every transition, not only valid golden files.

### Effect on the `rocci-template` package

The package boundary remains valid, but its implementation becomes more ambitious:

- it parses a Roc language superset rather than scanning top-level component bodies;
- it lowers markup expressions wherever they appear, preserving surrounding Roc verbatim where possible;
- it must version-lock its Roc grammar assumptions to the supported compiler;
- it returns generated Roc and segment maps exactly as before;
- it still does not type-check Roc, invoke the Roc compiler, define routes, or own runtime behavior.

Normal-function JSX improves the generated model: there is no component metadata or declaration rewrite beyond markup expressions. It makes the source parser and formatter harder while making runtime semantics, type inference, unit testing, and higher-order composition simpler.

### Comparison with the explicit `component` form

| Concern | `component |props| { HTML }` | Ordinary Roc functions with JSX |
| --- | --- | --- |
| Function model | Special source declaration lowered to a function | Function is already ordinary Roc |
| Roc control flow around markup | Requires expressions returning `Html` or template structure | Native `if`, `match`, closures, and local defs |
| Multiple components per file | Yes | Yes, automatically |
| Type annotations and tests | Map through generated declaration | Written directly against the normal function |
| Higher-order views | Possible after lowering | Natural in source |
| Parser boundary | Strong and local | Recursive Roc/markup grammar |
| Formatter/LSP difficulty | High but bounded | Highest; requires Roc-aware mixed parser |
| Compiler coupling | Can preserve most Roc as opaque ranges | Tracks essentially all expression grammar |
| Future upstream path | External preprocessor can remain viable | Best with upstream parser support |

### Assessment of the JSX direction

Full JSX is the better **language model** because views really are ordinary Roc functions and Roc already has expression-oriented `if` and exhaustive `match`. It removes the need to invent template control-flow directives and makes render functions easy to annotate, pass, and test.

It is not automatically the better **first implementation**. Without a reusable Roc parser or extension point, an external full-JSX front end risks becoming a second Roc parser and formatter. Before freezing `component`, run a focused parser spike with:

1. JSX as both branches of `if` and `match`;
2. JSX inside a `List.map` closure inside a child expression;
3. local definitions before the returned markup;
4. nested records, strings, comments, and `<` comparisons inside `{...}`;
5. incomplete tags and braces followed by another top-level Roc definition;
6. exact diagnostic mapping and stable formatting.

If a future Roc parser exposes a stable markup-expression extension point, full JSX should be reconsidered. With an external source-to-source compiler today, its language elegance does not outweigh the risk of duplicating a large fraction of Roc's grammar and formatter. The bounded component grammar below is the more feasible initial design.

## Alternative: tagged HTML template literals

A quasiquoted or JavaScript-style tagged literal would provide a strong boundary while keeping components as ordinary Roc functions:

```rocci
card = |{ my_class_var, title }| {
    html`<div class=${my_class_var}>${title}</div>`
}
```

This is an attractive middle ground between full JSX and explicit `component` declarations. The opening `` html` `` switches into template mode, the matching backtick returns to Roc, and `${...}` holes switch temporarily into ordinary Roc expressions. Markup can therefore appear wherever an expression is accepted without making every `<` token context-sensitive.

Conceptually, lowering would operate on alternating static segments and typed expression holes:

```text
html`<div class=${my_class_var}>${title}</div>`

TemplateLiteral(
    static markup: "<div class=", ">", "</div>",
    holes: Attribute(my_class_var), Child(title),
)
```

It should lower directly to safe `Html` constructors rather than concatenate and reparse a rendered string. The template parser knows that the first hole is in attribute context and the second is in child context, so it can generate context-appropriate types, escaping, and source maps.

### Why Roc compile-time evaluation is not sufficient by itself

Roc's new compiler performs constant folding: pure functions applied to constants may run during compilation. That could make an ordinary call such as this cheap at runtime:

```roc
static_node = Html.parse("<div class=\"card\"></div>")
```

However, constant evaluation and syntax extension solve different problems. An ordinary `html` function receives values only after Roc has parsed and type-checked its arguments. It cannot:

- make `html` followed by a backtick legal Roc syntax;
- inspect the source expression `my_class_var` inside a string;
- turn embedded source text into typed Roc expression holes;
- splice a newly generated Roc AST into its call site;
- reliably issue template diagnostics at exact tag, attribute, and hole ranges;
- require evaluation during compilation as a semantic rule rather than benefit from constant folding when possible.

Roc's existing string interpolation also does not provide this behavior:

```roc
# This constructs a Str. It is not a typed HTML quasiquote.
Html.parse("<div class=\"${my_class_var}\"></div>")
```

Here interpolation converts the hole into part of one string before `Html.parse` sees it. The parser loses the distinction between static trusted markup and a dynamic attribute value, making contextual escaping and non-`Str` child values awkward. If `my_class_var` is not compile-time constant, the complete string cannot be parsed at compile time anyway.

A library-only approximation could separate the constant format from explicitly tagged values:

```roc
Html.template(
    "<div class={0}>{1}</div>",
    [Attr(my_class_var), Child(Html.text(title))],
)
```

This can be safe and the static format may be constant-folded, but it is substantially less ergonomic, forces heterogeneous holes through a common wrapper type, and still cannot provide syntax-aware completion and precise diagnostics without external tooling.

### Feasible implementation paths

There are three distinct implementation levels:

| Path | Can use a tagged backtick literal? | Compile-time validation | Typed holes and precise diagnostics | Assessment |
| --- | --- | --- | --- | --- |
| Ordinary Roc function | No | Opportunistic constant folding only | Limited; function receives values | Useful runtime API, not the proposed language |
| `rocci-template` preprocessing | Yes, in `.rocci` files | Yes, before Roc compilation | Yes, with segment maps | Feasible alternative surface syntax |
| Upstream Roc tagged/quasiquoted literals | Yes, in `.roc` files | Yes | Best compiler/LSP integration | Best long-term path, requires Roc language support |

### Could native Roc tagged templates be parsed soundly?

Yes. Backtick literals can be added with a deterministic lexical mode switch; they do not require the parser to guess whether `<` means comparison or markup. If Roc adopted JavaScript's actual hole marker, the core grammar could be approximately:

```text
TaggedTemplate ::= TagName "`" TemplatePart* "`"
TemplatePart   ::= RawTemplateText | "${" RocExpression "}"
TagName        ::= LowerIdent | QualifiedLowerIdent
```

For example:

```rocci
card = |{ my_class_var, body }| {
    html`<div class=${my_class_var}>${body}</div>`
}
```

The lexer/parser uses an explicit mode stack:

1. In Roc-expression mode, a permitted tag name immediately followed by a backtick starts a tagged template expression.
2. In template mode, characters are raw data until an unescaped closing backtick or `${`.
3. `${` pushes normal Roc-expression mode with a delimiter-depth counter.
4. The matching depth-zero `}` returns to template mode.
5. The closing backtick returns one `TaggedTemplate` expression to the surrounding Roc parser.

Strings, comments, records, blocks, and nested delimiters inside a hole are handled by the existing Roc lexer/parser. A tagged template inside a hole is recursively well-defined because it pushes another template mode. Recovery can synchronize at the hole's `}`, the literal's closing backtick, or the surrounding Roc block when either is missing.

JavaScript permits more complex expressions as template tags. Roc should initially restrict the tag to a lowercase value reference such as `html` or a qualified reference such as `Html.template`; this keeps precedence, formatting, and recovery simple. The restriction can be widened later without changing literal contents.

The earlier HTML examples use JSX-like `{expression}` holes. That is also parseable because the HTML-template parser can treat balanced braces specially, but it is not exactly JavaScript template syntax and makes literal braces more troublesome. `${expression}` is the better choice for a general Roc tagged-literal feature because ordinary `{` remains raw data. A specialized `html` literal could still deliberately choose `{...}` for visual consistency with HTML attributes, at the cost of an escape rule for literal braces.

Syntactic soundness does not by itself define the typing model. JavaScript can pass an array of strings followed by arbitrarily typed dynamic arguments because it is dynamically typed. Roc needs one of these designs:

- a compiler/elaborator API in which the tag processes syntax nodes and returns a typed expression;
- a statically typed heterogeneous tuple/argument representation generated per literal;
- an HTML-specific intrinsic which assigns an expected type to each hole from its markup context; or
- preprocessing which lowers every hole directly into ordinary Roc constructor calls before Roc type checking.

For Rocci, the final option is the practical one. `rocci-template` parses the static HTML and lowers `${my_class_var}` as an attribute value and `${body}` as a child value. Roc then type-checks the generated calls. Dynamic values never become part of a string that is reparsed, preserving contextual escaping and preventing markup injection by construction.

If Roc eventually exposes user-defined compile-time elaborators, they also need deterministic evaluation rules, resource limits, cache keys, and a diagnostic/source-span API. Those concerns affect whether arbitrary user-defined tags are safe and reproducible; they do not make the tagged-template grammar ambiguous.

Therefore:

- **Parsing:** sound and substantially simpler than full JSX.
- **Type safety:** sound if holes remain typed expressions rather than interpolated strings.
- **HTML safety:** sound if lowering uses context-safe constructors and requires an explicit raw-HTML escape hatch.
- **Current Roc compatibility:** not valid `.roc` syntax today; it needs `rocci-template` preprocessing or an upstream parser extension.

For a `.rocci` preprocessor, this form is considerably easier than full JSX. The scanner recognizes `` html` `` only outside Roc strings and comments, scans template text to an unescaped closing backtick, and uses balanced `${...}` regions for Roc holes. It still needs the version-locked Roc island lexer, but it does not need to decide whether arbitrary `<` means comparison or markup.

Important syntax decisions would include escaping literal backticks and literal `${`, multiline indentation, whether `html` can be qualified, and whether tagged literals nest inside their Roc holes. Although nesting is grammatically sound, the bounded v1 preprocessor should initially reject nested template literals inside holes; authors can call a named render function instead. This reduces recovery and source-map complexity.

### Assessment

Tagged HTML literals are technically credible and deserve a parser spike. Their main advantage over `component` declarations is that the result is visibly an ordinary Roc expression and composes naturally with `if`, `match`, closures, lists, and local definitions. Their main disadvantages are unfamiliar backtick syntax, the need to parse mixed Roc/template expressions throughout a module, and the fact that the function-like `html` prefix is actually compiler/preprocessor syntax.

Compile-time evaluation improves the implementation of a library-level static-template API, but it does not eliminate the need for syntax-aware parsing and lowering. For v1, tagged literals should be evaluated as an alternative source grammar implemented exclusively by `rocci-template`, not described as an ordinary Roc parser function.

## Recommended: explicit components with a bounded template grammar

The feasible middle ground is to retain an unmistakable component declaration boundary and support a small structural template language inside it:

```rocci
@component todoList = |{ items, state }| {
    @match state {
        Loading => <Spinner />

        Failed(message) => <ErrorNotice message={message} />

        Ready =>
            @if List.isEmpty(items) {
                <EmptyState />
            } @else {
                <ul>
                    @for item in items {
                        <TodoRow item={item} />
                    }
                </ul>
            }
    }
}
```

This is not arbitrary Roc with HTML inserted into it. The component body has its own finite grammar. Roc appears in directive headers, explicitly braced HTML interpolation/attributes, and pattern positions. The normal Roc compiler ultimately parses and type-checks the generated expressions and patterns.

### Is it a subset of Roc?

Not literally. A sound design has two layers:

1. **Template structure:** HTML, component tags, interpolation, `@if`, `@for`, `@match`, and perhaps `@let`.
2. **Roc regions:** component parameter patterns, directive-header expressions, expressions inside HTML `{...}`, and patterns in `@match`/`@let`.

The structural constructs lower to a small subset of ordinary Roc—`if`, `match`, local definitions, closures, `List.map`, and `Html` constructors—but their source syntax is intentionally not valid Roc. Calling it “a subset of Roc” would suggest that arbitrary Roc syntax should work inside a component body, which recreates the full JSX parsing problem.

A more accurate description is:

> **A bounded HTML template grammar with lexically delimited Roc expressions and Roc-pattern regions.**

The template parser can preserve Roc tokens without understanding their types or complete AST. The generated virtual module delegates those responsibilities to the pinned Roc compiler.

### Why explicit directive markers

templ can place Go `if`, `switch`, and `for` directly among markup because its compiler owns a Go-aware mixed grammar. Rocci could copy that appearance:

```rocci
if condition {
    <Panel />
}
```

but bare words in an HTML-oriented body create awkward recovery and text ambiguities. A misspelled keyword can become rendered text, and the parser must decide whether `if` begins control flow or is content.

An `@` prefix creates an explicit mode switch without requiring another pair of interpolation braces around directive expressions:

```rocci
@if condition {
    <Panel />
}
```

It also leaves ordinary text, HTML tags, and Roc interpolation visually distinct:

```text
<tag>       markup
{expr}      Roc value inserted into an HTML text/attribute position
@if expr {  template structure with a Roc header expression
```

This is more important for parser recovery than for valid files. An editor can identify the intended construct even when its condition or body is incomplete.

Alternative markers are possible:

| Form | Strength | Weakness |
| --- | --- | --- |
| `if condition { ... }` | Closest to templ | Ambiguous with text and requires parsing an unbounded condition |
| `@if condition { ... }` | Explicit mode; concise; body brace terminates the expression | Top-level record braces require parentheses |
| `@if {condition} { ... }` | Two explicit boundaries and easy recovery | Redundant braces around a value already in template syntax |
| `{#if condition}...{/if}` | Excellent recovery and familiar from template languages | Least Roc-like; duplicates opening/closing structure |
| `<If condition={...}>...</If>` | Uses tag parsing only | Makes binders, else branches, and exhaustive matching awkward; looks like a runtime component |
| `{if condition { viewA } else { viewB }}` | Genuine Roc expression | Cannot contain inline markup without the full JSX parser |

`@directive RocExpression { TemplateBody }` is viable if the body opener is defined lexically rather than inferred from Roc precedence. The exact rule is specified below.

### Proposed grammar

An illustrative grammar is:

```text
ComponentDecl  ::= "@component" PascalName "=" RocParams TemplateBlock
StyleDecl      ::= RocName "=" "styles" "module" CssBlock
                 | "styles" "global" CssBlock

TemplateBlock  ::= "{" TemplateItem* "}"
CssBlock       ::= "{" CssTokens "}"

TemplateItem   ::= Element
                 | ComponentCall
                 | Fragment
                 | Text
                 | Interpolation
                 | IfDirective
                 | ForDirective
                 | MatchDirective
                 | LetDirective
                 | TemplateComment

Interpolation  ::= "{" BracedRocExpression "}"
HeaderExpr     ::= RocTokensBeforeBodyBrace

IfDirective    ::= "@if" HeaderExpr TemplateBlock
                   ("@else" "if" HeaderExpr TemplateBlock)*
                   ("@else" TemplateBlock)?

ForDirective   ::= "@for" RocBinder "in" HeaderExpr TemplateBlock

MatchDirective ::= "@match" HeaderExpr "{" MatchArm+ "}"
MatchArm       ::= RocMatchPattern "=>" MatchValue
MatchValue     ::= Element
                 | ComponentCall
                 | Fragment
                 | Interpolation
                 | IfDirective
                 | ForDirective
                 | MatchDirective

LetDirective   ::= "@let" RocBinder "=" LineRocExpression
```

`HeaderExpr`, `LineRocExpression`, `RocMatchPattern`, `RocParams`, and `RocBinder` are token ranges governed by explicitly documented boundaries. They are not parsed as template syntax.

For v1, `RocBinder` in `@for` should be one lowercase identifier. Tuple, record, and tag destructuring can be added only after the pattern scanner and diagnostic mapping are proven. `@match` needs richer patterns from the start, but it only has to capture balanced Roc tokens up to a top-level `=>` and let generated Roc perform final validation.

### Conditional rendering

```rocci
@component accountActions = |{ user }| {
    @if user.isSignedIn {
        <LogoutButton />
    } @else if user.canRegister {
        <RegisterButton />
    } @else {
        <LoginButton />
    }
}
```

Lowering is approximately:

```roc
accountActions = |{ user }| {
    if user.isSignedIn {
        logoutButton({})
    } else if user.canRegister {
        registerButton({})
    } else {
        loginButton({})
    }
}
```

Two policies are possible for an omitted `@else`:

- require `@else`, exactly matching Roc's expression semantics; or
- lower a missing branch to `Html.empty`, matching common template expectations.

Requiring `@else` is more Roc-like, but conditional rendering without an else branch is extremely common. The recommended template rule is that `@else` is optional and its absence means `Html.empty`. This is custom template semantics and should be documented plainly rather than disguised as raw Roc.

The generated Roc compiler verifies that each condition is `Bool` and that all generated branches return compatible `Html` values.

### Iteration

```rocci
@component todoRows = |{ todos }| {
    <ul>
        @for todo in todos {
            <TodoRow todo={todo} />
        }
    </ul>
}
```

Lowering is approximately:

```roc
todoRows = |{ todos }|
    Html.element(
        "ul",
        [],
        List.map(todos, |todo| todoRow({ todo })),
    )
```

`@for` is declarative list rendering, not a general imperative loop. V1 should support `List` only by lowering to `List.map`; the Roc compiler then verifies the collection and binder types. Indexes, filters, dictionaries, streams, and folds should use ordinary Roc helpers which return a list or `Html` until concrete syntax is justified.

An optional empty branch can be expressed with `@if List.isEmpty(...) { ... }` rather than adding `@empty` to the loop grammar.

### Pattern matching

Mirror the current Roc compiler's `match expression { Pattern => expression }` form and preserve its pattern vocabulary:

```rocci
@component requestState = |{ state }| {
    @match state {
        Loading => <Spinner />

        Failed({ message }) => <ErrorNotice message={message} />

        Ready(items) if !List.isEmpty(items) => <ItemList items={items} />
        Ready(_) => <EmptyState />
    }
}
```

It lowers to a normal Roc `match` expression whose branches each return `Html`. Exhaustiveness, unreachable branches, guards, payload types, and bound names are checked by Roc, not reimplemented in the template parser.

Conceptually, the generated control flow is:

```roc
requestState = |{ state }| {
    match state {
        Loading => spinner({})
        Failed({ message }) => errorNotice({ message })
        Ready(items) if !List.isEmpty(items) => itemList({ items })
        Ready(_) => emptyState({})
    }
}
```

Each arm produces exactly one self-delimiting `MatchValue`, paralleling Roc's rule that an arm produces one expression. This removes a visually redundant template block around every result. Use a fragment when an arm needs multiple sibling nodes:

```rocci
@match state {
    Ready(items) => <>
        <Heading text="Ready" />
        <ItemList items={items} />
    </>

    Loading | Refreshing => <Spinner />
    _ => <EmptyState />
}
```

A nested `@if`, `@for`, or `@match` is also one match value. Bare text is not accepted directly as an arm result because it has no reliable lexical endpoint; wrap it in an element or fragment. Roc alternatives (`|`), list and record patterns, aliases (`as`), wildcards, and `if` guards can all remain captured pattern tokens. The pinned Roc compiler is the authority on which precise forms are valid.

### Local bindings

Local derived values are useful because component props are often transformed before rendering:

```rocci
@component filteredList = |{ items, query }| {
    @let visible = List.keepIf(items, |item| matches(item, query))

    @if List.isEmpty(visible) {
        <EmptyState query={query} />
    } @else {
        <ItemList items={visible} />
    }
}
```

For predictable scope and lowering, v1 should permit `@let` only before render-producing items in its current template block. Its binding is visible to subsequent siblings and nested blocks, never to preceding markup. The expression remains full Roc; the binder should initially be a single identifier.

If arbitrary interleaving of `@let` and rendered nodes is allowed, lowering must nest fragments to preserve lexical scope. That is feasible, but the simpler “bindings first” restriction improves formatting, diagnostics, and generated code.

### Roc expressions, but no markup inside them

HTML interpolation and attributes still require braces because the parser is otherwise in markup/tag mode:

```rocci
<span class={badgeClass(if active { Positive } else { Neutral })}>
    {formatUser({ user, locale })}
</span>
```

Directive headers do not need those interpolation braces:

```rocci
@if badgeClass(if active { Positive } else { Neutral }) == "badge" {
    <Badge />
}
```

Nested records, lists, closures, calls, `if`, and `match` are legal in Roc regions as long as the pinned Roc compiler accepts them and the directive header follows the termination rule below. However, inline markup is not legal inside a Roc expression:

```rocci
# Not valid in the bounded grammar:
{if active { <ActiveIcon /> } else { <IdleIcon /> }}
```

Use a structural directive:

```rocci
@if active {
    <ActiveIcon />
} @else {
    <IdleIcon />
}
```

or call ordinary named render functions from the expression:

```rocci
{if active { activeIcon({}) } else { idleIcon({}) }}
```

This single restriction is what keeps the parser bounded. Roc regions never switch recursively back into markup mode.

### How an unbraced directive expression ends

After `@if`, `@match`, or the `in` keyword of `@for`, the scanner collects Roc tokens until it reaches the first `{` at delimiter depth zero. That brace is the template-body opener and is not part of the expression.

The scanner tracks Roc lexical modes plus nested parentheses and brackets. Braces inside strings/comments do not count. Braces nested inside `(...)` or `[...]` belong to the Roc expression:

```rocci
# Valid: the record is nested inside a call.
@if isVisible({ user, permissions }) {
    <Profile />
}

# Valid: a top-level record expression is parenthesized.
@match ({ status, items }) {
    { status: Loading } => <Spinner />
    { status: Ready } => <ItemList items={items} />
}
```

An unparenthesized record literal or record update at header top level is ambiguous and therefore rejected:

```rocci
# Invalid: the first top-level `{` starts the template body.
@match { status, items } {
    # ...
}
```

Parentheses are an escape hatch for any expression containing a top-level brace, so this is a syntactic inconvenience rather than an expressiveness loss.

For strong recovery, the body-opening `{` should occur on the same logical header. Newlines are permitted while `(` or `[` remains open, but a depth-zero newline before the body brace is an incomplete-directive error. This prevents a missing `{` from consuming following markup or another directive as if it were Roc.

`@let` has no following body brace, so it needs a different boundary. The recommended v1 rule is a single logical line: its expression ends at a depth-zero newline, with continuation allowed only inside unmatched `(...)` or `[...]`. More elaborate multiline computation belongs in an ordinary Roc helper. Alternatively, `@let` can retain `{expression}` in v1 if this line restriction proves awkward; this choice does not affect the other directives.

### Downsides of omitting expression braces

The simplified syntax has real but bounded costs:

- top-level record literals and updates require parentheses;
- a missing body brace has weaker local recovery than `@if {expr} {`, mitigated by the same-logical-header rule;
- formatting must know the exact header/body boundary and preserve required parentheses;
- `@let` cannot use precisely the same termination rule because it has no template body;
- syntax highlighting cannot color the directive expression merely by finding paired braces—it needs the directive scanner;
- future directives whose headers contain multiple expressions need explicit separators or keywords.

These costs are smaller than the persistent visual noise of redundant braces in every conditional, loop, and match. The recommended syntax is therefore the simplified form, while retaining braces for actual HTML interpolation and attribute values.

### What the template subset should include

| Capability | V1 syntax | Lowering | Type/semantic authority |
| --- | --- | --- | --- |
| Static element | `<div>...</div>` | Safe `Html.element` constructors | Template validator + Roc types |
| Component call | `<UserCard user={user} />` | Direct `userCard({ user })` call | Roc arity and record typing |
| Text/attribute expression | `{expression}` | Context-specific safe constructor | Roc expression/type checker |
| Conditional | `@if expr {...} @else {...}` | Roc `if` returning `Html` | Roc requires `Bool` and compatible branches |
| Iteration | `@for item in items {...}` | `List.map` plus fragment flattening | Roc verifies list/binder types |
| Match | `@match expr { Pattern => value }` | Roc `match` returning `Html` | Roc verifies patterns, guards, and exhaustiveness |
| Local binding | `@let name = expr` | Roc local definition | Roc name/type checker |
| Fragment | `<>...</>` | `Html.fragment` | Template validator + Roc type checker |
| Default body | second component parameter | second direct function argument | Roc arity/type checker |
| Named/scoped content | `Html` or function-valued props | ordinary record fields/functions | Roc type checker |

Do not include arbitrary Roc definitions, effects, imports, type declarations, `expect`, record builders, or general loops directly in the template body. They remain available in the ordinary Roc region of the module and can be called from template expressions.

### Block values and whitespace

Every `TemplateBlock` lowers to one `Html` value, normally `Html.fragment` over its render-producing items. `@if` and `@match` contribute one `Html`; `@for` contributes the ordered result of its mapped bodies; `@let` contributes no node.

Whitespace must be deterministic rather than an accidental result of code formatting. A conservative initial rule is:

- preserve text inside HTML elements exactly as template text, subject to normal HTML rendering;
- discard leading and trailing indentation-only text at component and directive block boundaries;
- do not synthesize spaces between adjacent dynamic values or tags;
- preserve whitespace in raw-text/preformatted elements according to HTML rules;
- expose the normalized text nodes through `rocci inspect` and golden render tests.

This lets authors indent structural directives without each branch acquiring different leading newlines, while avoiding broad whitespace collapsing which could change inline content.

### Parser feasibility

This grammar can be parsed soundly by an external package without implementing the full Roc parser.

The outer scanner needs only enough Roc lexical awareness to find this exact top-level form:

```text
@component PascalName = RocParams {
```

It must be token-aware and indentation/delimiter-aware so the same text inside a string, comment, record, or nested expression is ignored. Restricting `@component` to the start of a top-level definition provides a strong synchronization point.

Inside the component body, a conventional recursive-descent parser owns all structure. The first significant token of an item determines its grammar:

| Token | Mode |
| --- | --- |
| `<` | HTML element, component call, or fragment |
| `{` | Roc expression interpolation |
| `@if` | conditional directive |
| `@for` | list-rendering directive |
| `@match` | pattern-match directive |
| `@let` | local binding directive |
| text | escaped text node |
| `}` | end of the current template block |

Roc regions require a real Roc-aware lexer, not raw searches for `{`, `}`, or newlines. It must recognize strings, escapes, comments, character/byte syntax if applicable, and nested delimiters. For braced HTML interpolation it finds the matching outer `}`. For directive headers it identifies the first `{` at depth zero as the body opener. It does not decide whether the captured expression is well typed—or even fully syntactically valid. Generated Roc diagnostics provide final validation.

This division is sound provided the supported Roc compiler and the island lexer are version-locked. If Roc adds a literal or comment form which changes delimiter meaning, `rocci-template` must update before claiming support for that compiler version.

### Parser recovery

Valid-file parsing is the easy part; the language will feel sound only if incomplete editor states recover predictably.

Recommended synchronization points are:

- the closing tag of the current element;
- `@else` after a damaged `@if` body;
- the next top-level `=>` separator inside `@match`;
- the closing brace of the current directive or component;
- the next column-zero/top-level Roc definition after an unterminated component.

Diagnostics should distinguish:

- template-owned errors, such as an unmatched closing tag or missing directive body;
- Roc-region boundary errors, such as an unterminated interpolation or missing directive-body opener;
- Roc-owned errors inside a successfully bounded expression or pattern;
- generated-scaffolding errors, which map to the owning directive or component declaration.

Unknown directive names should be errors with suggestions. To render a literal `@` followed by a directive-like word, either use `@@` or an HTML entity; choose one escape and specify it rather than silently treating typos as text.

### Hygiene and semantic soundness

Generated code must be hygienic even though Roc does not provide a macro system:

- generated temporary names use a reserved prefix which source declarations cannot use, or names proven absent from the source token stream;
- user binders are emitted exactly once in the corresponding Roc closure or pattern;
- no directive evaluates its expression more than once unless documented;
- branch and loop order matches source order;
- dynamic text and attributes always flow through context-safe constructors;
- raw HTML remains an explicit trusted type/API;
- the same input and compiler-package version produce byte-identical Roc and segment maps.

The template compiler itself does not prove Roc type soundness. It preserves a semantics which the Roc compiler checks. Confidence comes from:

1. a deterministic grammar with no heuristic tag/expression guessing;
2. golden parse and lowering tests;
3. malformed-input recovery tests;
4. source-map round-trip/property tests;
5. generated Roc compilation tests for every directive;
6. differential tests showing directive lowering matches hand-written Roc constructors.

### Feasibility conclusion

This parser is substantially more feasible than full JSX. It needs:

- a small outer Roc lexer/scanner;
- an HTML/component recursive-descent parser;
- a finite directive grammar;
- a Roc-aware lexer for braced interpolation, directive headers, line expressions, and patterns;
- deterministic lowering and segment maps.

It does **not** need to parse Roc operator precedence, type annotations, or `if`/`match` nesting inside expressions. Braces delimit HTML interpolation; the first depth-zero `{` delimits a directive header from its body; newline delimits the restricted `@let`; and a depth-zero `=>` delimits a match pattern. The Roc compiler diagnoses the captured contents.

The main risks are compiler-version drift in the Roc lexer, recovery when a directive body brace is missing, top-level record-brace ambiguity, whitespace semantics around structural blocks, and accurate diagnostic maps. Those are bounded engineering problems. Full JSX's recursive Roc/markup expression grammar is not similarly bounded without parser integration.

## Component semantics

A render component is only a pure function:

```text
Component props = props -> Html
```

It does not intrinsically have:

- mutable or retained state;
- a lifecycle or mounted instance;
- a route or HTTP method;
- an effect capability;
- a Datastar subscription;
- an independently patchable DOM boundary.

Those concerns belong to page/program and runtime APIs outside the template package. A component may render an element with a stable ID that another layer chooses as a patch boundary, but the template compiler does not register or retain that boundary.

## Separate template syntax from the component flow model

The template language should answer only one question:

> Given typed Roc values, what `Html` value should be produced?

It should not answer where those values came from, how an interaction becomes a message, where state is stored, how effects run, or which rendered node is patched. Those are flow-model decisions.

This produces three independent layers:

| Layer | Typical values | Owner |
| --- | --- | --- |
| Render | `counterView : CounterProps -> Html` | `rocci-template` lowers markup to ordinary Roc |
| Flow | handlers, `update`, reducer/effects, actors, or another library | Ordinary Roc declarations and an optional runtime package |
| Hosting | HTTP decoding, sessions, persistence, SSE, patch delivery | Roc platform/application integration |

The dependency direction is one-way: a flow calls a render function. The render function does not ask the compiler for its current model, dispatch messages, retain hooks, or acquire capabilities implicitly.

### Self-contained does not mean framework-owned

A `.rocci` file is a Roc module with additional template declarations, not a serialized component instance. It may colocate all of these:

```text
Counter.rocci
  domain types and helpers       ordinary Roc
  messages/actions               ordinary Roc
  update/handlers/effects        ordinary Roc
  small render components        template declarations
  page view                      template declaration
  route/program value            ordinary Roc
  optional styles                extracted template artifact
```

`rocci-template` transforms only the render declarations and preserves the ordinary Roc definitions. A later application/runtime stage consumes any explicitly exposed route or program values. Therefore a feature can be fully self-contained in one file without the template package knowing whether it uses Elm, direct handlers, reducers, or another architecture.

For example, this mixed source:

```rocci
Msg := [Increment]

update = |model, msg| # ordinary Roc

@component view = |{ count }| {
    <output>{count.to_str()}</output>
}

program = Elm.program({ update, view })
```

is conceptually lowered to:

```roc
Msg := [Increment]

update = |model, msg| # preserved ordinary Roc

view = |{ count }| {
    Html.element("output", [], [Html.text(count.to_str())])
}

program = Elm.program({ update, view }) # preserved ordinary Roc
```

The lowering neither recognizes nor validates `Elm.program`; normal Roc type checking does that after template lowering. Replacing the last line with `Server.route(...)` changes the architecture without changing the template AST or generated view function.

### Architecture-neutral render layer

The following views can be reused by every flow model below:

```rocci
@component counterButton = |{ label, action, disabled }| {
    <button data-on:click={action} disabled={disabled}>
        {label}
    </button>
}

@component counterPanel = |{ count, incrementAction, resetAction, busy }| {
    <section id="counter">
        <output>{count.to_str()}</output>
        <CounterButton
            label="Increment"
            action={incrementAction}
            disabled={busy}
        />
        <CounterButton
            label="Reset"
            action={resetAction}
            disabled={busy}
        />
    </section>
}
```

The view knows how to render actions but not how the action strings are decoded or handled. A stricter API could replace `Str` with a platform-defined `BrowserAction` value. That improves construction safety without coupling the template compiler to a flow architecture.

### Option A: explicit request handlers in one file

This is the smallest server-oriented model. State remains in an authoritative store and each handler renders the same pure view:

```rocci
@component counterPanel = |{ count, incrementAction, resetAction, busy }| {
    <section id="counter">
        <output>{count.to_str()}</output>
        <button data-on:click={incrementAction} disabled={busy}>Increment</button>
        <button data-on:click={resetAction} disabled={busy}>Reset</button>
    </section>
}

renderCounter = |model| {
    counterPanel({
        count: model.count,
        incrementAction: Browser.post("/counter/increment"),
        resetAction: Browser.post("/counter/reset"),
        busy: Bool.False,
    })
}

show! = |request, context| {
    model = Counter.load!(context.db, request.session)
    Server.html(renderCounter(model))
}

increment! = |request, context| {
    Auth.require!(request, Counter.Write)
    Counter.increment!(context.db, request.session)
    model = Counter.load!(context.db, request.session)
    Server.patch("#counter", renderCounter(model))
}

reset! = |request, context| {
    Auth.require!(request, Counter.Write)
    Counter.reset!(context.db, request.session)
    model = Counter.load!(context.db, request.session)
    Server.patch("#counter", renderCounter(model))
}

counterRoutes = Server.routes([
    Server.get("/counter", show!),
    Server.post("/counter/increment", increment!),
    Server.post("/counter/reset", reset!),
])
```

Only `counterPanel` uses the template grammar. The loader, authorization, mutations, patch choice, and route table are ordinary Roc. The example is one deployable feature module, but no lifecycle has been added to the render component.

### Option B: Elm-style Model–View–Update in one file

An Elm-style runtime can use the same template compiler. The `.rocci` file supplies a model, exhaustive messages, a pure update function, a view adapter, and a program value:

```rocci
Model : { count: I64 }

Msg := [Increment, Reset]

init : Model
init = { count: 0 }

update : Model, Msg -> Model
update = |model, msg| {
    match msg {
        Increment => { ..model, count: model.count + 1 }
        Reset => { ..model, count: 0 }
    }
}

@component counterPanel = |{ count, incrementAction, resetAction }| {
    <section id="counter">
        <output>{count.to_str()}</output>
        <button data-on:click={incrementAction}>Increment</button>
        <button data-on:click={resetAction}>Reset</button>
    </section>
}

view = |model| {
    counterPanel({
        count: model.count,
        incrementAction: Elm.dispatch(Increment),
        resetAction: Elm.dispatch(Reset),
    })
}

counterProgram = Elm.program({ init, update, view })
```

Here `Elm.program` owns message delivery and model retention. It is a library/runtime choice above templates; `component` still lowers to a normal function. Another file can call `counterPanel` without running an Elm program, and the template compiler does not special-case `Model`, `Msg`, `init`, `update`, or `view`.

For a backend, a request-driven MVU variant is usually safer because durable state, authentication, and concurrent windows remain explicit:

```rocci
Msg := [Increment, Reset]

load! = |request, context| Counter.load!(context.db, request.session)

handle! = |msg, request, context| {
    Auth.require!(request, Counter.Write)

    match msg {
        Increment => Counter.increment!(context.db, request.session)
        Reset => Counter.reset!(context.db, request.session)
    }
}

view = |model| counterPanel({
    count: model.count,
    incrementAction: Server.dispatch(Increment),
    resetAction: Server.dispatch(Reset),
})

counterProgram = Server.mvu({
    path: "/counter",
    load!,
    handle!,
    view,
    patch: "#counter",
})
```

The runtime performs `load! -> handle! -> reload -> view -> patch`. The template remains identical to the pure Elm version.

### Option C: pure reducer with explicit effects

A larger feature may want pure decisions while retaining an explicit server effect boundary:

```rocci
Msg := [Increment, Reset]
Command := [Add(I64), Set(I64)]

decide : CounterState, Msg -> List(Command)
decide = |state, msg| {
    match msg {
        Increment => [Add(1)]
        Reset => [Set(0)]
    }
}

run! = |commands, context| Counter.execute!(context.db, commands)

handle! = |msg, request, context| {
    state = Counter.load!(context.db, request.session)
    commands = decide(state, msg)
    run!(commands, context)
}

view = |state| counterPanel({
    count: state.count,
    incrementAction: Server.dispatch(Increment),
    resetAction: Server.dispatch(Reset),
})

counterProgram = Server.reducer({ load!: Counter.load!, handle!, view })
```

This changes testing and effect interpretation, not template syntax. `decide` can be tested without HTML; `counterPanel` can be snapshot-tested without a database; integration tests cover their composition.

### Option D: parent-controlled or state-hoisted component

A reusable component does not need to own any flow declarations at all. Its parent supplies values and actions:

```rocci
@component counterPanel = |{ count, incrementAction, resetAction }| {
    <section class="counter">
        <output>{count.to_str()}</output>
        <button data-on:click={incrementAction}>+</button>
        <button data-on:click={resetAction}>Reset</button>
    </section>
}

@component settingsPage = |{ settings, actions }| {
    <main id="settings">
        <h1>Settings</h1>
        <CounterPanel
            count={settings.retryCount}
            incrementAction={actions.incrementRetries}
            resetAction={actions.resetRetries}
        />
    </main>
}
```

The same file may stop here and expose `counterPanel`, or it may additionally define the parent program. File boundaries express ownership and convenience; they do not determine state ownership.

### What must remain outside template lowering

To preserve architecture choice, `rocci-template` must not assign special meaning to declarations named `model`, `init`, `update`, `view`, `handle!`, or `program`. It must not generate stores, message buses, effect interpreters, routes, subscriptions, hook slots, or component identities. A flow library can establish conventions around those names through ordinary Roc types and functions, but that contract belongs to the library and application-entry generator.

The only bridge required is a normal typed value:

```text
flow-owned state/actions -> props record -> render function -> Html
```

This narrow bridge allows many flow architectures while keeping each feature colocated and independently testable in one `.rocci` module.

### Architecture focus for the first POC

The first POC should not attempt to prove every architecture above. It should prove one runtime pipeline at two authoring levels:

1. **Primary: explicit request handlers.** A single `Counter.rocci` contains pure views, a loader, authorized mutation handlers, route values, and an explicit `#counter` patch response. This is the reference semantics.
2. **Secondary: request-driven MVU adapter.** The same view is connected through a typed `Msg`, `load!`, `handle!`, and `Server.mvu`. The adapter must reuse the reference handler pipeline and durable store rather than retain its own model.

The acceptance criterion is architectural substitution: changing between the two flows must not change the template AST, component lowering, generated render function, or `rocci-template` dependencies. Only ordinary Roc flow declarations and application wiring change.

Pure Elm `init/update/view` remains a valuable example and may be implemented entirely as a third-party runtime, but it should not be part of the first backend POC. Its retained local model does not exercise the difficult server requirements—authorization, canonical persistence, concurrent windows, and reload after effects. Similarly, reducer/effect, actor, LiveView, hooks, and reactive-graph models should remain library experiments until the explicit pipeline is stable.

## Names and resolution

Recommended rules:

- Component declarations are PascalCase: `@component CounterCard`.
- Lowercase tags are HTML elements.
- PascalCase tags are component references and must match a declaration.
- `<CounterCard>` and `@component CounterCard` both resolve to the local Roc value `counterCard`.
- `<Design.Button>` resolves to the imported Roc value `Design.button`.
- Ordinary Roc in a `.rocci` file (`exposing` lists, `@on` handlers, helpers) uses the camelCase Roc name, because those regions are copied through as Roc.
- Unknown PascalCase tags are compile errors.
- Component declarations use normal Roc visibility: a generated value is private unless exposed by the module.
- Initialisms use ordinary lower-camel names: `htmlShell` is written `@component HtmlShell` / `<HtmlShell>`. Ambiguous spellings such as `<HTMLShell>` should be rejected or require an explicit future alias feature.

This transformation is compile-time only. Generated code calls the resolved function directly; there is no runtime tag registry or dynamic dispatch.

## Props and attributes

Static attributes produce strings:

```rocci
<Button tone="quiet" />
```

Expression attributes contain Roc expressions:

```rocci
<Button tone={tone} disabled={isSaving} />
```

The component invocation lowers approximately to:

```roc
button({ tone, disabled: isSaving })
```

For HTML elements, lowering is context-aware:

- text expressions are escaped as text;
- ordinary attributes are escaped as attribute values;
- boolean attributes accept `Bool` and are included or omitted correctly;
- URL-bearing attributes use URL-aware constructors or validation;
- attribute names cannot be dynamic in v1;
- raw strings never become raw HTML;
- raw HTML requires a conspicuous trusted type or API.

Prop spreading, dynamic tag names, and string-to-component lookup are excluded from v1. They obscure the component's inferred input shape and complicate source mapping.

## Nested content and slots

### The problem with a magic `children` field

The earlier proposal placed nested markup into a record field named `children`:

```rocci
@component card = |{ title, children }| {
    <section>
        <h2>{title}</h2>
        {children}
    </section>
}

@component page = |{}| {
    <Card title="Settings">
        <SettingsForm />
    </Card>
}
```

This declaration mentions `children`, but the call site never passes that field. The compiler silently rewrites nested markup into it. That makes `children` a reserved-by-convention identifier, creates a special record field which ordinary Roc calls must understand, and becomes awkward when a component wants `body`, `label`, `emptyState`, or several content regions instead.

Other systems make different versions of this tradeoff:

- templ exposes nested content through a special `{ children... }` expression and passes it through context;
- React injects nested JSX as the `children` prop;
- Vue declares `<slot>` outlets and supports named and scoped slots;
- Astro and Web Components use `<slot>` outlets plus named slot assignment;
- Svelte 5 models reusable markup as function-like snippets, while retaining `children` as shorthand for a default snippet.

These designs show two distinct needs which should not be conflated:

1. a convenient single body for wrappers such as `Card`; and
2. named or parameterized render inputs for layouts, lists, and headless components.

### Design requirements

A Rocci design should make the following visible in the declaration:

- whether the component accepts nested content;
- whether content is one HTML value or several named values;
- the local names used by the implementation;
- whether a content value is plain `Html` or a function such as `Item -> Html`;
- whether omitting content differs from providing an empty fragment.

It should also lower to ordinary Roc values without a runtime slot registry, context lookup, or component instance.

| Declaration family | Default body | Named regions | Scoped regions | Magic introduced |
| --- | --- | --- | --- | --- |
| `{ children }` prop | Yes | Additional reserved props/conventions | Render props | Reserved `children` identifier |
| Second argument `|props, body|` | Yes | No | Body could explicitly be a function | Positional body convention |
| Named `Html` props | Only if named explicitly | Yes | No | None beyond component syntax |
| Slot-record argument `|props, { header, body }|` | Optional `body` field | Yes | Function-valued fields | Call-site fill syntax |
| `<slot>` outlets | Yes | Yes | Needs scoped-slot syntax | Reserved outlet element and synthesized signature |
| Render-function props | If modeled as a function | Yes | Yes | None beyond ordinary props |

### Alternative 1: reserved `children` record field

```rocci
@component card = |{ title, children }| {
    <section>{children}</section>
}
```

Nested markup lowers to:

```roc
card({ title: "Settings", children: settingsForm({}) })
```

Advantages:

- familiar to React users;
- only one props record;
- simple when every component uses the same convention.

Disadvantages:

- `children` is semantically magic despite looking like an ordinary field;
- it cannot be renamed to `body` or `content`;
- ordinary Roc calls and generated component calls have subtly different record construction;
- named content regions need another mechanism anyway.

This should not be the core Rocci model.

### Alternative 2: explicit body parameter

Treat nested markup as a second component argument:

```rocci
card = |{ title }, body|
    <section class="card">
        <h2>{title}</h2>
        <div class="card__body">{body}</div>
    </section>

settings = |{ form }|
    <Card title="Settings">
        {form}
    </Card>
```

The call lowers approximately to:

```roc
card(
    { title: "Settings" },
    Html.fragment([form]),
)
```

The parameter can be named `body`, `content`, `label`, `items`, or anything else. Its position—not its identifier—declares that the component accepts a default body. This is ordinary Roc function arity after lowering.

A useful context-free call rule is:

- `<Icon name="save" />` lowers to a one-argument call;
- `<Card title="Settings">...</Card>` lowers to a two-argument call;
- `<Card title="Settings"></Card>` lowers to a two-argument call with `Html.empty`;
- the Roc compiler reports an arity mismatch if the declaration and call disagree.

This preserves the meaningful distinction between a self-closing call and an explicitly empty paired call without requiring the template compiler to load imported component signatures.

Advantages:

- no reserved local identifier;
- acceptance of body content is explicit in function arity;
- direct lowering with no synthetic props field;
- cross-file mistakes become normal Roc type/arity errors;
- the same component remains straightforward to call from ordinary generated Roc.

Disadvantages:

- the second argument has language-defined meaning inside a `component` declaration;
- optional body content needs a deliberate policy;
- it covers only one unnamed body.

This is the recommended default-body design.

### Alternative 3: named `Html` props

Named content regions can be ordinary record fields rather than a slot subsystem:

```rocci
layout = |{ header, main, footer ?? Html.empty }|
    <div class="layout">
        <header>{header}</header>
        <main>{main}</main>
        <footer>{footer}</footer>
    </div>

pageHeader = |{ user }|
    <h1>Welcome, {user.name}</h1>

dashboard = |{ model }|
    <Dashboard model={model} />

page = |{ user, model }|
    <Layout
        header={pageHeader({ user })}
        main={dashboard({ model })}
    />
```

There is no special slot declaration: `header`, `main`, and `footer` are typed `Html` props. Roc's default record-field syntax can express an optional region if the pinned compiler supports it in this context.

Advantages:

- completely ordinary Roc records and calls;
- names, requiredness, and types are explicit;
- works without new nested slot syntax;
- easy to pass content through several layers.

Disadvantages:

- substantial markup must be extracted into a named component or built in an expression;
- the call site is less visually nested;
- repeated `foo({ ... })` expressions may feel lower-level than HTML composition.

This is the recommended v1 approach for named static regions.

### Alternative 4: explicit slot-record parameter

Several named regions can be declared together as a second record argument:

```rocci
layout = |{ title }, { header, main, footer ?? Html.empty }|
    <div class="layout">
        <header>{header}</header>
        <main>{main}</main>
        <footer>{footer}</footer>
    </div>
```

A future nested fill syntax could make the call read:

```rocci
page = |{ user, model }|
    <Layout title="Dashboard">
        <Fill name="header">
            <h1>Welcome, {user.name}</h1>
        </Fill>
        <Fill name="main">
            <Dashboard model={model} />
        </Fill>
    </Layout>
```

and lower to:

```roc
layout(
    { title: "Dashboard" },
    {
        header: Html.element(...),
        main: dashboard({ model }),
    },
)
```

`Fill` would be explicit compiler syntax, not a runtime component. Unlike a magic `children` identifier, the declaration's second record pattern states the accepted names, and the call states which region it fills.

Possible surface spellings include:

| Call-site spelling | Benefit | Cost |
| --- | --- | --- |
| `<Fill name="header">...</Fill>` | Explicit, readable, and easy to diagnose | Introduces one reserved pseudo-element |
| `<template slot="header">...</template>` | Uses familiar HTML vocabulary and supports fragments | Repurposes the native `<template>` element in component-call context |
| `<Fragment slot="header">...</Fragment>` | Similar to Astro and avoids a rendered wrapper | Requires a built-in `Fragment` and special `slot` attribute |
| `h1 slot="header"` on direct children | Closest to Web Components | Awkward for multiple nodes; compiler must strip or reinterpret a normal HTML attribute |
| `<:header>...</:header>` | Concise and unambiguous to the compiler | Not HTML-like and weaker in existing editors |

This option is attractive if named content is common, but should be added only after ordinary named `Html` props prove too verbose.

### Alternative 5: outlet elements inside the declaration

Vue, Astro, and Web Components declare insertion points using `<slot>`:

```rocci
layout = |{ title }|
    <header><slot name="header" /></header>
    <main><slot /></main>
```

This makes visual placement obvious, supports fallback markup, and is familiar. However, it hides the slots from the Roc function signature. The compiler must synthesize a slots record, reserve names such as `default`, decide how ordinary Roc calls provide it, and create special rules for scoped slots.

For Rocci, explicit function parameters provide a stronger typed declaration than outlet tags. `<slot>` should not be the semantic core.

### Alternative 6: render-function props

Content which depends on values supplied by the child is a function, not static `Html`. This is Vue's scoped-slot/render-prop idea expressed directly in Roc:

```rocci
dataList = |{ items, renderItem }|
    <ul>{List.map(items, renderItem)}</ul>

userRow = |user|
    userRowView({ user })

usersPage = |{ users }|
    <DataList items={users} renderItem={userRow} />
```

Conceptually, `renderItem` has a type such as `User -> Html`. Multiple scoped regions are simply multiple function-valued fields. This retains Roc inference, closures, and normal composition; no `slotProps` object or scope-changing template directive is necessary.

Advantages:

- exactly models parameterized content;
- ordinary Roc functions and types;
- works for lists, tables, empty states, and headless components;
- no runtime or compiler slot registry.

Disadvantages:

- inline markup closures would require the harder recursive Roc/markup grammar;
- v1 callers may need to extract the rendering function as a named declaration.

This is the recommended model for scoped slots from the beginning, even if the first syntax requires named functions.

Static `Html` content and render functions also have different evaluation semantics. A body argument or `Html` prop is built eagerly by the parent and passed as a value. A function prop is invoked by the child, may receive child-owned data, and may be called zero, one, or several times. Rocci should preserve that distinction rather than silently wrapping every body in a closure. If a wrapper truly needs lazy content, it can declare an explicit function such as `renderBody : {} -> Html`.

### Alternative 7: compound subcomponents

A component library can expose several explicit components rather than inspect or redistribute arbitrary children:

```rocci
page = |{ title, content }|
    <Panel>
        <PanelHeader title={title} />
        <PanelBody content={content} />
    </Panel>
```

This is useful when each region has real semantics or styling behavior. It is not a general slot replacement: the parent either needs the default body parameter or must receive the assembled region as `Html`. React's documentation similarly recommends explicit subcomponents and render props over manipulating opaque child collections.

### Recommended layered model

Use ordinary Roc mechanisms first and add syntax only where it materially improves authoring:

1. **Default nested body:** a second component parameter with any local name.
2. **Named static regions:** ordinary `Html` fields in the props record.
3. **Scoped/parameterized regions:** ordinary function-valued props such as `Item -> Html`.
4. **Named nested syntax:** defer; if needed, lower an explicit `<Fill name="...">` form to a second record argument.

In compact form:

```rocci
# One default body: second argument, arbitrary name.
card = |{ title }, body|
    <section><h2>{title}</h2>{body}</section>

# Named static and scoped content: ordinary props.
table = |{ caption, rows, renderRow }|
    <table>
        <caption>{caption}</caption>
        <tbody>{List.map(rows, renderRow)}</tbody>
    </table>
```

This removes `children` as a magic identifier. The only special rule is structural and visible: paired component tags supply a second argument. Everything richer is represented by Roc records and functions, so the template layer does not invent a parallel component type system.

## Expressions

Use `{expression}` consistently in text and attribute positions:

```rocci
profileLink = |{ person, selected }|
    <a
        href={person.url}
        class={if selected { "selected" } else { "" }}
        aria-current={if selected { "page" } else { "false" }}
    >
        {person.name}
    </a>
```

An expression position accepts a value supported by its context. At minimum:

- text-like values in text positions;
- `Html` and `List Html` in child positions;
- the attribute-specific value type in an attribute position.

The generated virtual Roc module places each expression inside the component closure so the Roc compiler can infer props and report normal type errors. Segment maps translate those errors back to the expression in the `.rocci` file.

## Control flow

The recommended bounded control-flow grammar is specified in [Recommended: explicit components with a bounded template grammar](#recommended-explicit-components-with-a-bounded-template-grammar). It deliberately provides `@if`, list-oriented `@for`, Roc-pattern `@match`, and prefix-only `@let`, while leaving general computation in lexically bounded Roc expressions and ordinary module declarations.

## Styles

CSS colocation is useful, but it must not turn render components into runtime instances. Styles should compile to static artifacts and ordinary HTML attributes. `rocci-template` may parse, validate, rewrite, and return CSS; a caller remains responsible for writing, bundling, serving, and linking it.

### Design goals

Embedded CSS should:

- support several independently styled components in one `.rocci` module;
- produce no per-render `<style>` elements and no runtime style registry;
- preserve the normal CSS cascade, media queries, custom properties, pseudo-classes, and animations;
- make local versus global effects visible in source;
- generate stable output from source identity and content, not build order;
- source-map CSS diagnostics to the `.rocci` file;
- avoid silently styling a child component's private markup;
- allow external stylesheets and design-system classes alongside embedded CSS.

### Alternative 1: one module-level global block

The smallest syntax is an extracted CSS block applying to the entire document:

```rocci
styles global {
    :root {
        --space-unit: 0.25rem;
    }

    .counter-panel {
        display: grid;
        gap: calc(var(--space-unit) * 2);
    }
}

@component counterPanel = |{ count }| {
    <section class="counter-panel">
        <output>{count.to_str()}</output>
    </section>
}
```

This is easy to parse and exactly preserves CSS semantics. It works well for resets, tokens, typography, layouts, and small applications using BEM-like names. Its downside is equally clear: every selector is global and collisions are detected only by convention. The word `global` should therefore be mandatory rather than making an unmarked block global accidentally.

Only one `styles global` block should be allowed per module in v1. Multiple files may contribute global artifacts; the bundler defines their deterministic cascade order from the application import graph and reports cycles or ambiguous ordering.

### Alternative 2: CSS Modules as typed Roc records

The most Roc-like local-style model is an explicitly named CSS module:

```rocci
counterStyles = styles module {
    .root {
        display: grid;
        gap: 0.5rem;
    }

    .count {
        font-variant-numeric: tabular-nums;
    }

    .danger {
        color: var(--danger-color);
    }
}

@component counterPanel = |{ count, isDangerous }| {
    <section class={counterStyles.root}>
        <output class={if isDangerous {
            Css.classes([counterStyles.count, counterStyles.danger])
        } else {
            counterStyles.count
        }}>
            {count.to_str()}
        </output>
    </section>
}
```

The conditional above is illustrative; ordinary Roc should ideally construct the class list outside markup, or the template grammar can support a small class-list attribute form. The important semantic model is that the style declaration generates an ordinary Roc value resembling:

```roc
counterStyles = {
    root: "root_r7s3m",
    count: "count_r7s3m",
    danger: "danger_r7s3m",
}
```

and extracted CSS resembling:

```css
.root_r7s3m { display: grid; gap: 0.5rem; }
.count_r7s3m { font-variant-numeric: tabular-nums; }
.danger_r7s3m { color: var(--danger-color); }
```

The Roc compiler then catches `counterStyles.cout` as a missing record field. No component-style association is inferred: one style record can be shared by several local components, and a component can consume several style records. This matches the general Rocci rule that explicit values compose better than component-instance metadata.

Recommended v1 restrictions keep the generated record predictable:

- exported local class selectors must be one Roc-compatible lower-camel identifier such as `.resetButton`;
- hashes derive from the stable package/module identity, style declaration name, and compiler format version—not an absolute filesystem path or build order;
- every local class is rewritten consistently inside compound selectors and media/container rules;
- local IDs are rejected initially because IDs represent document identity, not reusable presentation;
- `composes`, Sass/Less, arbitrary CSS-module export values, and tree-shaking unused selectors are deferred;
- `:global(selector)` is the explicit escape for a selector portion which must not be renamed;
- classes referenced only through dynamic strings are unsupported; use the generated record.

This approach changes class selectors but does not need to inject a scope attribute into every DOM element. It preserves component boundaries naturally: a parent's class affects a child only when the parent explicitly passes that class as a prop and the child places it on an element.

### Alternative 3: automatically scoped component styles

Vue, Svelte, and Astro commonly scope selectors by adding generated attributes or classes to the template and rewriting selectors. A Rocci spelling could explicitly associate a block with a component:

```rocci
styles scoped counterPanel {
    .root > output {
        font-weight: 700;
    }
}

@component counterPanel = |{ count }| {
    <section class="root">
        <output>{count.to_str()}</output>
    </section>
}
```

Conceptually this could become:

```html
<section class="root" data-rocci-s="r7s3m">
    <output data-rocci-s="r7s3m">...</output>
</section>
```

```css
.root:where([data-rocci-s~="r7s3m"]) >
output:where([data-rocci-s~="r7s3m"]) {
    font-weight: 700;
}
```

This is convenient but has more semantic machinery than CSS Modules:

- the compiler must define lexical ownership for intrinsic nodes, fragments, passed bodies, and raw HTML;
- a component call has no guaranteed DOM root on which to place the parent's scope;
- parent layout styling of a child root needs attribute forwarding or an explicit class prop;
- selector rewriting must handle `:is`, `:where`, `:not`, pseudo-elements, nesting, at-rules, and keyframes with a real CSS parser;
- recursive components and deep descendant selectors can cross boundaries surprisingly;
- generated scope selectors can alter specificity unless `:where(...)` or an equivalent strategy is specified carefully.

If introduced later, scope should apply only to intrinsic markup lexically authored in that component. Child component internals should not receive the parent's scope. Content authored by a caller retains the caller's scope even when passed as a body value. `:global(...)` and perhaps `:deep(...)` would need explicit definitions. These rules are feasible, but they are too large for the first POC.

### Alternative 4: CSS nested inside a component declaration

A Vue-like spelling could put a non-rendering directive in the body:

```rocci
@component counterPanel = |{ count }| {
    @styles module counterStyles {
        .root { display: grid; }
    }

    <section class={counterStyles.root}>{count.to_str()}</section>
}
```

This provides tight visual colocation but creates a new declaration scope inside the template grammar, complicates the “template block produces `Html`” rule, and makes sharing styles across sibling components awkward. Because `.rocci` already permits multiple nearby top-level declarations, a named top-level `styles module` value gives nearly the same locality with simpler semantics. Nested style declarations should be rejected.

Literal HTML `<style>` remains available only when the author intentionally wants a rendered style element, for example a generated standalone document. It is not the mechanism for component styling and should be rejected inside patchable fragments by default to prevent repeated insertion and Content Security Policy complications.

### Alternative 5: inline style attributes and dynamic values

Static inline declarations need no new syntax:

```rocci
<div style="display: grid; gap: 0.5rem">...</div>
```

Dynamic styles should not be assembled by concatenating arbitrary strings. A typed helper can serialize an allowlisted property record:

```rocci
<progress
    style={Css.inline({
        inlineSize: Css.percent(progress),
        accentColor: theme.accent,
    })}
/>
```

For values shared across several declarations or pseudo-state rules, CSS custom properties are the better bridge:

```rocci
meterStyles = styles module {
    .root {
        color: var(--meter-color);
    }

    .root:hover {
        outline-color: var(--meter-color);
    }
}

@component meter = |{ color, value }| {
    <div
        class={meterStyles.root}
        style={Css.vars({ meterColor: color })}
    >
        {value.to_str()}
    </div>
}
```

`Css.vars` should generate escaped custom-property declarations and map `meterColor` deterministically to `--meter-color`. Values should use CSS value types or an explicitly trusted escape rather than accept arbitrary raw CSS. Unlike Vue's reactive CSS binding, Roc simply emits the current variable value during each server render; Datastar's DOM morph updates the attribute.

### Alternative 6: external CSS and utility classes

External stylesheets remain first-class:

```rocci
import styles "./counter.css"

@component counterPanel = |{ count }| {
    <section class="counter-panel">{count.to_str()}</section>
}
```

The exact import syntax should follow Rocci's eventual asset/module grammar rather than pretend to be an ordinary Roc import. It can either register a global stylesheet artifact or import a CSS-module record. Utility-class systems require no template feature beyond static and constructed class attributes, although build tooling may scan the template AST to discover literal classes.

External files are preferable for global design systems, generated framework CSS, large shared stylesheets, and editor workflows where CSS has independent ownership. Embedded modules are preferable for small styles tightly coupled to a few colocated render functions.

### Extraction and delivery contract

`rocci-template` should return style artifacts as data, for example:

```text
StyleArtifact {
    source_range
    kind: Global | Module
    logical_name
    css
    class_exports
    source_map
    dependencies
}
```

It must not write assets, decide public URLs, inject styles into running pages, or retain which components have mounted. The caller/bundler should:

1. collect artifacts reachable from the application module graph;
2. order global CSS deterministically and deduplicate module artifacts by identity;
3. validate CSS and rewrite module selectors with a standards-aware CSS parser;
4. produce content-hashed production assets and a CSS manifest;
5. insert one stylesheet link into complete documents;
6. use a stable development URL or full reload when CSS changes.

An HTML fragment patch should never carry the component CSS again. This keeps CSP straightforward, avoids duplicate rules, and allows browser caching.

### Recommendation and POC scope

Use two explicit style categories:

1. `styles global { ... }` for deliberate document-wide CSS.
2. `name = styles module { ... }` for local classes exposed as a typed Roc record.

CSS Modules should be the default recommendation for component-local styling. They fit Roc's explicit value model, do not depend on a component lifecycle or single-root element, and work naturally when several components share one `.rocci` file.

For the first POC, implement only a single named `styles module` block, basic local class selectors, deterministic hashing, extraction, a generated class record, source-mapped CSS parse errors, and one production/development stylesheet link. Keep global blocks as a small follow-up if needed for the demo reset. Defer automatic scoped attributes, `:deep`, CSS preprocessors, composition, CSS tree shaking, critical-CSS inlining, and CSS-only hot replacement.

## Dedicated implementation package

Template parsing and lowering should live in one narrowly scoped workspace package:

```text
crates/
  rocci-template/       .rocci parser, template AST, validation, lowering, segment maps
  rocci-cli/            orchestration, file watching, diagnostics presentation
  rocci-roc/            Roc toolchain/platform invocation and backend lifecycle
  rocci-core/           runtime-neutral backend/session contracts
```

The package name should be singular (`rocci-template`) because it implements one template language, even though a source module may contain many components.

### Responsibilities of `rocci-template`

- parse the selected `.rocci` Roc/markup grammar, whether full JSX or explicitly delimited markup islands;
- distinguish Roc expressions from HTML elements, component calls, attributes, text, and fragments;
- preserve byte offsets and UTF-16 positions for every source-backed node;
- validate template-local invariants such as balanced tags and legal attribute forms;
- resolve local component names and emit unresolved/imported references for later checking;
- lower every markup expression to ordinary Roc `Html` construction and direct function calls while preserving surrounding Roc semantics;
- produce deterministic generated Roc and bidirectional segment maps;
- optionally return extracted style artifacts without writing or serving them;
- expose parse/lower diagnostics as data rather than printing or exiting.

### Explicit non-responsibilities

`rocci-template` must not:

- start the Roc compiler or select/install a Roc toolchain;
- define routes, HTTP handlers, sessions, authorization, or Datastar SSE behavior;
- start or supervise backend processes;
- watch files or implement hot reload;
- write generated files, caches, or bundles itself;
- own application state, effects, or an Elm-style program runtime;
- depend on Wry, Axum, Tokio, or the Rocci desktop shell.

The CLI or a future `rocci-roc` package composes these stages:

```text
.rocci source
    -> rocci-template parse
    -> rocci-template validate/lower
    -> generated Roc + segment maps + optional CSS artifact
    -> rocci-roc invokes pinned roc check/build
    -> CLI remaps and presents diagnostics
    -> runtime package launches the built backend
```

Here “template compilation” means compiling the embedded template language into ordinary Roc. Invoking the Roc compiler and linking a backend executable are deliberately outside this package.

### Proposed public API shape

The exact Rust API should follow an implementation spike, but the boundary should resemble:

```rust
pub fn parse(source: SourceFile<'_>) -> ParseOutput;

pub fn lower(
    document: &Document,
    options: &LowerOptions,
) -> Result<LoweredModule, Vec<Diagnostic>>;

pub fn compile(
    source: SourceFile<'_>,
    options: &LowerOptions,
) -> CompileOutput;
```

`CompileOutput` should contain generated Roc, segment maps, discovered markup/component references, optional extracted styles, and all recoverable diagnostics. The API accepts source text and returns data; filesystem policy belongs to callers.

Suggested internal modules:

```text
src/
  lexer.rs             outer Roc tokens and bounded Roc regions
  parser.rs            error-tolerant component/template parser
  ast.rs               source-preserving component/template syntax tree
  validate.rs          template-local semantic checks
  resolve.rs           component tag/name resolution
  lower.rs             safe Html constructor generation
  source_map.rs        bidirectional generated/source segments
  diagnostic.rs        structured diagnostics and fix metadata
  lib.rs               stable package API
```

The parser and AST must not import runtime concepts. Datastar attributes are preserved as HTML attributes; optional spelling metadata can be supplied by tooling without coupling the parser to a particular Datastar release.

## Parsing strategy

Use the bounded component parser described in [Parser feasibility](#parser-feasibility): a token-aware outer scanner finds top-level `@component Name = |...| {` declarations, a recursive-descent template parser owns their bodies, and a Roc-aware lexer captures expressions using context-specific lexical terminators without recursively accepting markup.

The implementation must keep its modes explicit (`OuterRoc`, `ComponentSignature`, `Template`, `Tag`, `Attribute`, `BracedRoc`, `DirectiveHeader`, `LineRoc`, `Pattern`, and `DirectiveBody`) and version-lock Roc lexical assumptions. A raw search for delimiters or keywords is not sufficient. Full JSX remains a documented future alternative, not a second v1 parse mode.

## Lowering contract

Lowering produces a deterministic virtual Roc module. Conceptually:

```rocci
@component hello = |{ name }| {
    <p class="greeting">Hello, {name}</p>
}
```

becomes something like:

```roc
hello = |{ name }|
    Html.element(
        "p",
        [Html.attribute("class", "greeting")],
        [Html.text("Hello, "), Html.text(name)],
    )
```

The actual constructors must match the pinned Roc HTML package. Generated formatting, helper names, and import aliases must be stable for identical source and compiler-package versions.

Each generated segment records:

```text
generated start/end -> source URI + source start/end + origin kind
```

Origin kinds should distinguish ordinary Roc, component signatures, directives, component tags, text expressions, attribute expressions, static markup, and generated scaffolding. Diagnostics in scaffolding map to the nearest responsible directive or component declaration and may include a generated-code debug location.

## Tooling contract

The composite language server should consume `rocci-template` rather than implement a second parser. It can derive virtual documents from the same parse/lower result:

- generated Roc for Roc type diagnostics, completion, hover, and navigation;
- template HTML for HTML-aware tooling;
- extracted CSS for CSS tooling;
- the source AST for symbols, folding, semantic regions, and formatting boundaries.

CLI builds and editor sessions must generate byte-identical virtual Roc for the same input. Otherwise diagnostics and navigation will drift between tools.

Formatting should initially be conservative:

- format ordinary Roc ranges with the pinned Roc formatter;
- format markup with the template AST;
- preserve expression contents unless the Roc formatter can operate on an exact virtual range;
- decline transformations whose recovery parse is ambiguous;
- never format generated Roc and reverse-diff it back into source.

## Relationship to the block-format spike

A single `<roc>/<template>/<style>` document is still a useful experimental input for validating safe HTML lowering and segment maps. It is not the proposed stable language because it naturally implies one primary template per file and separates component signatures from their markup.

The dedicated package can temporarily support that input behind an explicit experimental mode, but its template AST and lowering stages should not assume one component per document. The block spike and stable component declarations should reuse the same HTML/component AST, validation, lowering, directive lowering, and source-map code after their different outer parsers locate a component body.

## Proof of concept

Before freezing the grammar:

1. Parse a `.rocci` module containing ordinary Roc plus at least five explicit component declarations.
2. Lower `<Hello name={person.name} />` to a direct call with a props record.
3. Nest `@if`, `@for`, and exhaustive `@match` constructs and verify their generated Roc semantics.
4. Support `@let`, one qualified imported component, fragments, text/attribute expressions, and an explicitly declared second body argument.
5. Exercise Roc regions containing nested records, lists, closures, strings, comments, `if`, `match`, and comparison operators—but no inline markup.
6. Verify that top-level record expressions in directive headers require parentheses while records nested inside calls work without extra syntax.
7. Map type, arity, non-`Bool` condition, non-list loop, and non-exhaustive match errors back to their exact source constructs.
8. Recover useful diagnostics when a tag, directive body, match arm, interpolation, or header is incomplete before another top-level Roc definition.
9. Verify whitespace output around directives with golden render tests.
10. Demonstrate that the package has no HTTP, async-runtime, desktop-shell, or process-management dependencies.
11. Use the same package output in both `rocci check` and the composite language server.
12. Prototype `` html`...` `` as an experimental expression form and compare parser recovery, source maps, formatting, and generated Roc against explicit `component` declarations.

## Open decisions

1. Is `@` the best directive marker, and should literal `@` escape as `@@` or require an HTML entity?
2. Should `@if` without `@else` lower to `Html.empty` as recommended, or require an explicit else branch like Roc?
3. Should the directive body brace be required on the same logical header as recommended?
4. Should `@let` use the proposed logical-line expression, retain `{expression}`, or be omitted from v1?
5. Is a single-identifier binder enough for v1 `@for` and `@let`?
6. Should v1 accept every pattern and guard form recognized by the pinned Roc lexer, or intentionally restrict any forms for better recovery?
7. Should component text accept only `Str`, or a small explicit `ToText`-like ability once Roc's exact type-system support is confirmed?
8. Should paired component tags always produce a second body argument, and should indentation-only content count as `Html.empty`?
9. Should self-closing calls to two-argument components fail through ordinary Roc arity checking, or should the compiler synthesize an empty body after consulting component metadata?
10. Are named `Html` and function props sufficient, or is a slot-record plus `<Fill>` syntax important enough for v1?
11. Which URL attributes require dedicated value types or validation?
12. Should the first stable release include both recommended style categories, or graduate typed CSS Modules before explicit global blocks?
13. How should the formatter handle multiline Roc regions and directive indentation?
14. What stable debug representation should `rocci inspect` expose for generated modules and segment maps?
15. Should tagged HTML literals replace explicit component declarations if the parser spike confirms that mixed-expression recovery remains bounded?

## Recommendation

Adopt `.rocci` as a multi-component Roc module format and make `rocci-template` its only parser and lowering implementation. Use explicit `@component Name = |params| { ... }` declarations with a bounded template grammar. Component names are PascalCase and lower to camelCase Roc functions. Keep `{Roc expression}` for HTML text and attribute interpolation, but use the simpler `@if expression { ... }`, `@for item in expression { ... }`, and `@match expression { Pattern => templateValue }` forms. The first depth-zero `{` opens each directive body; top-level record expressions must be parenthesized. A match arm returns one self-delimiting template value, with fragments for multiple siblings. Keep `@let` line-bounded or omit it until its ergonomics are proven.

Do not permit markup recursively inside Roc expressions and do not ship full JSX as an equivalent v1 syntax.

Run the tagged-literal spike before permanently freezing the declaration syntax. It retains ordinary Roc functions and has substantially stronger lexical boundaries than full JSX, but it must be presented honestly as `.rocci` syntax lowered by `rocci-template`, not as a library function enabled by constant folding.

Keep generated render functions pure, use HTML tags for direct typed function calls, and let records provide named props. Represent one nested body as an explicit second function argument with an arbitrary local name; represent named and scoped content as ordinary `Html` and function-valued props. Keep runtime state, routes, HTTP, Datastar, process management, filesystem writes, and Roc toolchain execution outside the package.

For colocated styling, use named `styles module` declarations which generate typed records of hashed class names. Support explicitly marked `styles global` blocks for document-wide rules. Extract both as build artifacts; do not inject styles per render or make style presence depend on component instances. Keep automatic attribute-scoped CSS out of the first POC.

This boundary allows the language to evolve and be tested independently while preventing a convenient template syntax from turning into a second application runtime.

## Primary sources

- templ, [Introduction](https://templ.guide/)
- templ, [Basic syntax](https://templ.guide/syntax-and-usage/basic-syntax/)
- templ, [Template composition and children](https://templ.guide/syntax-and-usage/template-composition/)
- templ, [If/else](https://templ.guide/syntax-and-usage/if-else/)
- templ, [Switch](https://templ.guide/syntax-and-usage/switch/)
- templ, [For loops](https://templ.guide/syntax-and-usage/loops/)
- Vue, [Slots](https://vuejs.org/guide/components/slots.html)
- Vue, [SFC CSS features](https://vuejs.org/api/sfc-css-features)
- Astro, [Components and slots](https://docs.astro.build/en/basics/astro-components/#slots)
- Astro, [Styles and CSS](https://docs.astro.build/en/guides/styling/)
- CSS Modules, [project documentation](https://github.com/css-modules/css-modules)
- W3C, [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/)
- MDN, [`<slot>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/slot)
- Svelte, [Svelte 5 migration guide: snippets instead of slots](https://svelte.dev/docs/svelte/v5-migration-guide#Snippets-instead-of-slots)
- React, [`Children` and alternatives](https://react.dev/reference/react/Children)
- TypeScript, [JSX](https://www.typescriptlang.org/docs/handbook/jsx.html)
- Roc, [Tutorial: records and modules](https://www.roc-lang.org/tutorial)
- Roc, [new compiler all-syntax fixture](https://github.com/roc-lang/roc/blob/main/test/echo/all_syntax_test.roc)
- Roc, [new compiler mini tutorial: constant folding and string interpolation](https://github.com/roc-lang/roc/blob/main/docs/mini-tutorial-new-compiler.md)

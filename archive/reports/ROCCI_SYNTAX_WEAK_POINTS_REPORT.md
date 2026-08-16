# Rocci syntax weak-points report

**Audit date:** 2026-08-14
**Scope:** the current working-tree implementation of `rocci-template`, its tests and documentation, the Roc Counter/Snake/Datastar examples, and the current official Datastar example/reference pages.

## Executive summary

The bounded `.rocci` grammar is a good first implementation boundary. It keeps ordinary Roc opaque, gives markup an unambiguous parser mode, lowers to a small `Html` API, and is already expressive enough to port Active Search, Click to Edit, TodoMVC, Lazy Tabs, and Inline Validation. The existing 16 `rocci-template` tests pass, and every `.rocci` file in `examples/datastar` lowers successfully.

The main risks are not missing convenience syntax. They are places where valid-looking source has a meaning different from what an author is likely to infer, or where lowering can emit invalid Roc:

1. `??` component defaults are implemented at the caller, not in the component, so defaults can resolve names in the wrong scope and do not work across module boundaries.
2. Attribute values mix three languages—static HTML text, Roc `Str`, and Datastar client expressions—without a visible type boundary. Server interpolation in a quoted attribute silently does nothing.
3. The custom quoted-attribute scanner consumes backslash escapes, which can silently change client expressions such as regular expressions and Windows-like paths.
4. Child `{expr}` is documented like a general interpolation but is actually `Str`-only, except for a name-based exception for bare body parameters. A computed `Html` value is wrapped in `Html.text`.
5. `<script>`, `<style>`, `<textarea>`, and `<title>` have no raw-text mode. JavaScript/CSS braces, `@`, and `<` are parsed as Rocci syntax.
6. There is no conditional or spread form for attributes. Dynamic HTML boolean attributes are especially hazardous because `disabled="false"` still means disabled in HTML.

Items 1, 3, 4, and 5 should be treated as pre-stabilization correctness work. The Datastar-specific sugar and richer loop syntax can wait.

## What was audited

- The parser and scanners in [`parser.rs`](../../crates/rocci-template/src/parser.rs) and [`lexer.rs`](../../crates/rocci-template/src/lexer.rs).
- Lowering and component metadata in [`lower.rs`](../../crates/rocci-template/src/lower.rs) and [`ast.rs`](../../crates/rocci-template/src/ast.rs).
- The implemented language reference in [`crates/rocci-template/README.md`](../../crates/rocci-template/README.md), plus the broader proposal in [`ROC_TEMPLATE.md`](../../ROC_TEMPLATE.md).
- The kitchen-sink fixture and compiler tests in [`crates/rocci-template/tests`](../../crates/rocci-template/tests).
- [`examples/counter`](../../examples/counter), [`examples/snake`](../../examples/snake), and the current [`examples/datastar`](../../examples/datastar) gallery.
- Official Datastar examples for [Active Search](https://data-star.dev/examples/active_search), [Click to Edit](https://data-star.dev/examples/click_to_edit), [TodoMVC](https://data-star.dev/examples/todomvc), [Lazy Tabs](https://data-star.dev/examples/lazy_tabs), and [Inline Validation](https://data-star.dev/examples/inline_validation), plus the [attribute](https://data-star.dev/reference/attributes), [action](https://data-star.dev/reference/actions), and [SSE event](https://data-star.dev/reference/sse_events) references.

The audit distinguishes three categories:

- **Correctness:** plausible syntax lowers incorrectly, corrupts content, or emits invalid Roc.
- **Ergonomics:** the result is correct but unnecessarily noisy or easy to misuse.
- **Adjacent runtime/tooling:** exposed by template examples, but not best fixed by expanding the grammar.

## Priority table

| Rank | Weak point | Class | Impact | Recommended timing |
| ---: | --- | --- | --- | --- |
| 1 | `??` defaults execute at the call site | Correctness | Can emit out-of-scope names; fails across modules | Before syntax stabilization |
| 2 | Static/Roc/client attribute boundary is invisible | Correctness + ergonomics | Dynamic URLs silently remain literal; client programs are opaque strings | Define the boundary now; add sugar later |
| 3 | Quoted attributes consume backslashes | Correctness | Silently changes regexes, escapes, and paths | Fix now |
| 4 | `{expr}` cannot insert general `Html` | Correctness + composability | Documented-looking composition emits a type error | Fix or document/restrict now |
| 5 | No raw-text element mode | Correctness | Inline JS, CSS, JSON-LD, and some text content misparse | Fix now or explicitly reject |
| 6 | No conditional/spread attributes | Correctness + ergonomics | Boolean-attribute trap; duplicated elements/branches | Near term |
| 7 | Component call shape is under-validated | Correctness + diagnostics | Multiple body parameters and non-record props lower to wrong arity/shape | Near term |
| 8 | Control flow is deliberately narrow | Ergonomics | Data must be pre-shaped; empty/index cases are verbose | Defer until repeated demand |
| 9 | Syntax sigils and header boundaries impose escape/formatting tax | Ergonomics | Text containing `@`, braces, or code samples is awkward | Document now; reconsider with formatter work |
| 10 | Datastar attributes and SSE variants are untyped | Ergonomics + tooling | Typos survive Roc compilation; helper coverage is incomplete | Library/LSP work before grammar work |
| 11 | Full-page and patch views can drift | Architecture | Duplicated patch subtrees and IDs | Component convention/library work |
| 12 | Documentation and test coverage do not yet define edge semantics | Tooling | Current behavior can stabilize accidentally | Before declaring the grammar stable |

## Detailed findings

### 1. `??` defaults are call-site rewrites, not component defaults

The source form suggests callee-scoped record-pattern defaults:

```rocci
labelled = component |{ x, label ?? makeLabel(x) }| {
    <p>{label}</p>
}

caller = component |{ value }| {
    <Labelled x={value} />
}
```

Current lowering strips the default from the function and inserts it into a local component call:

```roc
labelled = |{ x, label }| { ... }

caller = |{ value }| {
    labelled({ x: value, label: makeLabel(x) })
}
```

`x` is resolved in `caller`, where it does not exist; the intended value is the record field being passed as `value`. This is not just noisy generated code—it changes lexical scope.

The same mechanism stores defaults only for components declared in the current source document. An omitted default on `<Other.Labelled />`, or on a component called from ordinary Roc, cannot be filled. Defaults that reference private callee helpers also become inaccessible to callers.

**Recommendation:** remove or temporarily reject `??` in `.rocci` until Roc accepts the intended pattern syntax, or lower it into a callee-side wrapper with correct lexical scope. Do not describe the current rewrite as semantically equivalent. If a temporary subset is retained, restrict it to context-free literals and report cross-module omissions explicitly.

### 2. Attribute values cross three languages without a visible boundary

These forms look similar but execute in different places:

```rocci
data-on:click="@delete('/todos/${todo.id}')"
data-on:click={"@delete('/todos/${todo.id}')"}
```

The first is a static Rocci attribute. The browser receives the literal `${todo.id}`. The second is a Roc expression producing a `Str`, so Roc performs interpolation before rendering. The trap appears in per-row TodoMVC actions and generated Lazy Tabs URLs.

Datastar adds a third language inside the resulting string. Its attributes intentionally contain expressions, action calls, object literals, regular expressions, and modifiers. For example, the official reference uses values such as `{include: /user/}` and `$fetching`, while request actions use `@get(...)`/`@post(...)`. Rocci represents all of them as ordinary static text or `Str`, with no `ClientExpr` identity for highlighting, formatting, validation, or safe composition.

Multiline quoted attributes already parse, so capacity is not the problem. The problem is that a multiline client program still looks and behaves like undifferentiated static attribute text to Rocci's formatter, highlighter, and validator.

Object syntax makes the mode switch especially easy to misread:

```rocci
# Datastar object; must be static quoted text.
data-signals="{counter: 0}"

# Roc expression; braces are consumed by Rocci.
data-signals={Signals.initial(model)}
```

**Recommendation:** keep short client expressions in quoted attributes and larger behavior in JavaScript modules, as already recommended by [`SNAKE_DATASTAR_ARCHITECTURE_REPORT.md`](SNAKE_DATASTAR_ARCHITECTURE_REPORT.md). Add typed Roc builders for common actions and values before designing a client-side Roc dialect. A small `ClientExpr`/Datastar attribute boundary, plus LSP validation, is more valuable than making arbitrary JavaScript look like Roc.

Raw `data-*="..."` must remain an escape hatch.

### 3. The quoted-attribute scanner silently rewrites backslashes

`scan_quoted_string` treats the contents of every quoted HTML attribute as a custom escaped string. It converts `\n`, `\t`, and `\r`, accepts `\"`, and drops the backslash from every unknown escape.

For example:

```rocci
<div data-pattern="\d+" />
```

currently lowers as if the value were `d+`, not `\d+`. This matters directly to Datastar because its documented attribute expressions can contain JavaScript regular expressions. It also changes client string escapes and path-like text. The behavior is easy to miss because lowering succeeds.

**Recommendation:** define quoted attributes as HTML/template text, not as an underspecified Roc string literal. At minimum, preserve unknown backslash sequences and diagnose unterminated/invalid escapes. If `\"` and `\\` remain source escapes, document precisely which layer consumes them and add round-trip tests for regexes, JavaScript strings, newlines, and literal backslashes.

### 4. Child interpolation has a hidden, name-based type rule

The language reference says that text interpolation lowers to `Html.text(expr)`, except when the expression is a bare identifier matching an extra component/body parameter. That exception inserts the value directly as `Html`:

```rocci
page = component |{}, content| {
    <main>{content}</main>
}
```

But a general computed node does not work:

```rocci
view = component |{ value }| {
    <div>{render(value)}</div>
}
```

It lowers to `Html.text(render(value))`, even if `render(value)` returns `Html`. Aliasing a body value through `@let`, indexing a list of `Html`, or choosing a node with an ordinary Roc `if` has the same problem. This contradicts the broader design text in [`ROC_TEMPLATE.md`](../../ROC_TEMPLATE.md), which suggests calling named render functions from expressions.

The rule is also syntactic rather than typed: `content` is raw only because its spelling appears in `body_params`. Renaming it through a local binding changes lowering. Conversely, every extra component parameter is assumed to be an `Html` body parameter even though the parser does not enforce its type.

**Recommendation:** keep `{expr}` unambiguously text and add an explicit node/list insertion form, or make child interpolation type-directed through distinct constructors that Roc can check. Remove the bare-name heuristic once an explicit form exists. The syntax should make trusted/raw `Html` insertion visibly different from escaped text insertion.

### 5. Standard raw-text elements are parsed as Rocci template bodies

The parser uses the same child grammar for every non-void element. Consequently:

```rocci
<script>const config = { foo: 1 }</script>
```

lowers approximately to:

```roc
Html.text("const config = ")
Html.text(foo: 1)
```

JavaScript/CSS braces become Roc interpolation, `@` begins a directive, and `<` begins a tag. This affects:

- `<script>` and `<style>` raw-text content;
- JSON-LD and other inline data scripts;
- `<textarea>` and `<title>` escapable raw text;
- literal code samples and some `<pre>` content;
- whitespace-only content in preformatted contexts, because formatting whitespace is discarded without element context.

External JS and CSS files are a sound application convention, but they do not make the HTML grammar complete. The parser should not silently reinterpret a standard raw-text element.

**Recommendation:** implement element-aware raw-text scanning for the HTML categories, with an explicit and safe interpolation policy, or reject non-empty inline raw-text elements with a targeted diagnostic. Continue recommending modules/assets for non-trivial code.

### 6. Attribute syntax cannot express conditional presence

Rocci supports static strings, Roc `Str` expressions, and valueless attributes:

```rocci
<button disabled>
<a class={className}>
```

There is no dynamic boolean-attribute form. A plausible attempt such as `disabled={busy}` lowers to `Html.attribute("disabled", busy)`, which expects a string in the current `Html` API. Converting the boolean to `"false"` is semantically wrong: in HTML, the presence of `disabled`, `checked`, `selected`, `required`, and similar attributes makes them true regardless of their string value.

The Todo example therefore duplicates checkbox markup across `@if` branches. Conditional ARIA attributes, groups of `data-*` attributes, and optional IDs/classes have similar pressure. There is also no attribute spread/list form through which a typed helper could solve the problem.

**Recommendation:** add one narrow conditional-presence mechanism before a general spread feature. A form such as `disabled?={busy}` would be explicit and statically lowerable; an attribute-list spread is more general but expands the type/runtime surface. Whichever form is selected, validate duplicate attributes and define ordering/override behavior before adding spreads.

### 7. Component declarations accept shapes that tag calls cannot satisfy

The parser records whether the first parameter is a record, but component tag lowering always passes a props record. It also records every parameter after the first as a body parameter, while a paired component tag supplies exactly one body argument.

```rocci
modelView = component |model| { ... }
slots = component |{}, header, body| { ... }

caller = component |{}| {
    <Slots><p>Only one body value</p></Slots>
}
```

The first declaration cannot be called naturally through attributes because `<ModelView ...>` always produces a record. The second call lowers with two arguments even though `slots` requires three. Both are accepted by the template compiler and fail only later in generated Roc.

Related gaps include duplicate props, no spread props, and no compiler-level distinction between an HTML attribute and a component prop beyond capitalization.

**Recommendation:** validate the component subset the tag syntax can actually call: normally one record parameter plus zero or one body parameter. Treat other Roc functions as ordinary Roc rather than template components. If multiple slots are desired later, pass them as named `Html` or function-valued props instead of inferring them from positional parameters.

### 8. Control flow pushes collection shaping out of the template

The earlier findings are accurate:

- `@for` lowers only to `List.map`;
- the binder is one lowercase identifier;
- there is no loop index, destructuring, filter clause, or `@empty` arm;
- `@let` must precede render-producing siblings in its block;
- a `@match` arm must be one tag, fragment, interpolation, or directive, not bare text.

These constraints are coherent for v1, but they create recurring preparation code:

- empty lists require an outer `@if List.is_empty(...)`;
- indexes require `List.map_with_index` or pre-zipped records before the template;
- dictionaries/sets must become lists;
- destructuring moves into helpers or pre-shaped records;
- a value needed only after some markup cannot be introduced by `@let` at that point.

**Recommendation:** do not add all common template-loop features preemptively. The best first extension is likely an optional index binder or destructuring, but only after two or three real examples repeat the same pre-shaping. An `@empty` branch is convenient, not an expressiveness blocker.

### 9. Template sigils and directive boundaries are predictable but costly for literal content

Inside template text, `<`, `{`, `}`, and `@` are structural. `@@` emits one literal `@`, so email addresses and handles must be written with awareness of the template grammar. A text item beginning with `#` is treated as a Roc comment, and HTML comments are discarded rather than rendered. Braces in documentation/code samples require an interpolation workaround or another escape.

Directive headers add a separate boundary rule: the first depth-zero `{` opens the template body. Top-level record literals, record updates, `if` expressions, and other brace-bearing Roc expressions therefore need parentheses. The body opener must stay on the same logical line, and `@let` has a different newline-based termination rule.

These are reasonable costs of the bounded parser, but they mean `.rocci` cannot be formatted correctly by a generic HTML or Roc formatter.

**Recommendation:** document a complete literal-character table, including braces, `#`, entities, and raw-text elements. Add a real `.rocci` formatter before allowing more directive shapes; otherwise each extension multiplies boundary and recovery cases.

### 10. Datastar-aware validation is absent, but should not become a second client language

The attribute-name scanner is permissive enough for current Datastar forms such as:

```rocci
data-bind:search
data-indicator:_fetching
data-on:input__debounce.200ms="@get('/search/results')"
```

That compatibility is useful. It also means Rocci cannot catch:

- misspelled Datastar attributes/actions;
- keys that are forbidden for a particular attribute;
- malformed modifier order/durations;
- object syntax accidentally written as a Roc attribute expression;
- a request expression missing its `@` action marker;
- casing changes that alter signal names.

The official Datastar reference has a finite attribute/modifier vocabulary, but client expressions remain JavaScript-like and intentionally open-ended.

**Recommendation:** start with editor/schema diagnostics and typed action/value builders, not a client Roc dialect. A small `ds:*` alias may reduce noise later, but only if raw `data-*` forms remain first-class and source maps point diagnostics to the original attribute.

### 11. Page/patch reuse is a composition convention, not a syntax feature

Click to Edit, TodoMVC, Counter, and Snake all need both a full page and a smaller morphable subtree. Rocci has no template marker for a patch boundary, so it is easy to duplicate a full-page subtree and its patch rendering or let stable IDs drift.

The current mitigation is the right one: extract one inner render component and use it from both the page and the SSE response. A patch boundary is an element with a stable ID, not a special component lifecycle.

**Recommendation:** keep this out of the core grammar. Add a small library/convention that renders and patches the same component, and test that emitted patch roots have stable IDs.

### 12. SSE helper coverage and diagnostics are adjacent gaps

The example `Datastar.roc` helper emits only `datastar-patch-elements`. The official SSE format also supports signal patches, selectors, multiple morph modes, SVG/MathML namespaces, view transitions, and removal without element content. Infinite scroll and click-to-load examples need append/prepend plus selectors; title/signal updates need other event forms.

This is not a `.rocci` syntax defect. It becomes a syntax pressure point only if authors start hand-building wire strings in attributes or templates.

Likewise, `rocci-template` produces source-map segments but does not invoke Roc or remap Roc type errors itself. Many invalid combinations above are therefore accepted by the template pass and diagnosed later against generated code unless the surrounding CLI performs the mapping.

**Recommendation:** expand a typed Roc SSE helper and complete diagnostic remapping before adding transport directives to the template language.

## Datastar example impact

| Example | Current syntax pressure | Practical mitigation |
| --- | --- | --- |
| Active Search | Long modifier names; opaque `@get` expression | Keep the short quoted expression; LSP/schema validation later |
| Click to Edit | Full page vs `#contact` patch reuse; repeated loading/disabled attributes | Shared inner component; future conditional/spread attributes |
| TodoMVC | Dynamic per-row URLs; non-trivial Enter handler; empty list; repeated checked markup | Roc string builder for URLs, JS module for large handler, outer `@if`, conditional boolean attr later |
| Lazy Tabs | Generated URL and optional index | Put an ID/index in each pre-shaped tab record |
| Inline Validation | Repeated field structure; dynamic signal names/initial values; match-heavy notes | Component extraction and Roc helpers; Datastar schema diagnostics |
| File upload/dialog/key handling | Large client expressions and escape-heavy strings | Browser module/input island; do not grow a client Roc dialect |
| Click to Load/infinite scroll | Append/prepend/selector SSE options | Extend `Datastar.roc`, not `.rocci` syntax |

None of the recommended first gallery examples requires full JSX, attribute spreads, or a client-side Roc runtime. They do expose the need for safer boundaries.

## Recommended sequence

### Before calling the syntax stable

1. Remove/restrict `??` or implement callee-scoped defaults correctly.
2. Fix and specify quoted-attribute escaping; add regex/backslash round-trip tests.
3. Decide how escaped text, `Html`, and `List Html` are inserted as children; remove the bare-name heuristic.
4. Implement or reject raw-text element content explicitly.
5. Validate component parameter shapes, duplicate attributes/props, and paired/self-closing arity.
6. Add negative and lowering tests for all of the above.

### Near-term ergonomics

1. Add conditional boolean attributes.
2. Add Datastar-aware LSP diagnostics and typed request/SSE helpers.
3. Add a `.rocci` formatter that understands directive boundaries.
4. Establish a page/patch component convention in the gallery.

### Defer until examples justify them

1. General attribute/prop spreads.
2. `@for` destructuring, indexes, filters, and `@empty`.
3. Named slot syntax.
4. Dynamic component tags.
5. Full JSX or markup nested recursively inside Roc expressions.
6. A Datastar/client Roc expression language.

## Suggested acceptance tests

The current kitchen-sink fixture proves the happy path. Add focused cases for the semantic boundaries:

- a default that refers to another prop and a private callee helper;
- an omitted default on a qualified cross-module component;
- a Datastar regex containing `\d`, a literal backslash, and a client string escape;
- a computed `Html` node, a `List Html`, and a renamed body value;
- conditional `disabled`, `checked`, and `selected` attributes;
- inline `<script>`, `<style>`, `<textarea>`, `<title>`, and whitespace-only `<pre>` content;
- a component with a non-record first parameter and with two body parameters;
- duplicate HTML attributes and duplicate component props;
- literal `@`, `{`, `}`, `#`, and an email address in text;
- `@for` mixed with siblings and nested directives, ensuring list flattening remains ordered;
- compiler diagnostics mapped back from generated Roc to each source form.

## Conclusion

Rocci does not need a much larger template language to support the Datastar gallery. Its current bounded model is viable, and the deliberate loop/match limitations are tolerable. The more important work is to make every language boundary explicit and semantics-preserving:

- defaults must execute where their source says they execute;
- quoted client expressions must survive byte-for-byte unless an escape is explicit;
- text and `Html` insertion must not depend on an identifier's spelling;
- standard HTML raw-text and boolean-attribute rules must be represented deliberately;
- Datastar validation should be additive tooling/library support, not a second Roc-like language.

Addressing those points would make the existing small grammar safer to learn and safer to freeze, while preserving raw HTML/Datastar escape hatches and keeping application logic in ordinary Roc.

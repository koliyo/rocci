# Rocci landing-page prototype brief

**Exploratory implementation brief — 17 August 2026**

The proposed landing page is a dedicated orientation surface, not a replacement
for the article and reference shell. It should answer four questions within the
first screen: what Rocci is, whether it is ready for the visitor's use, how its
parts fit together, and what to try first.

## Information architecture

1. **Project chrome:** Rocci, Overview, Rocdown, Docs, Community, and GitHub.
2. **Status line:** public preview, independent project, and a direct path to
   compatibility and limitations.
3. **Hero:** “Build web and desktop interfaces in Roc,” a concrete one-sentence
   explanation, a five-minute primary action, and an architecture secondary
   action.
4. **Source-to-screen strip:** `.rocci / .rocdown → Roc + HTML → web / desktop`.
5. **Real example:** the current `Greeting` component syntax beside the exact
   lowering promise, avoiding fabricated output or benchmarks.
6. **Three task paths:** build a component, write a Rocdown page, and add a
   Datastar action. Each names an outcome rather than an internal crate.
7. **Relationship statement:** Rocci is independent; Roc and Datastar are
   ecosystem dependencies or integrations, not Rocci products.

## Visual direction

- Retain warm ivory, charcoal, coral, and white/dark surfaces from the current
  Rocs shell.
- Use violet only in the source-to-screen relationship line, not as a second
  action color.
- Use a wordmark-only header during identity testing. No placeholder or candidate
  symbol should acquire de facto status through the prototype.
- Keep the hero below roughly 3.4rem on ordinary desktop screens and let the
  code example become the strongest visual proof.
- Preserve a neutral system sans and monospace stack with no remote font
  dependency.

## Responsive behavior

- At laptop and desktop widths, the hero and code example form a balanced two
  column composition.
- Below 760px, content becomes one column, navigation wraps without a menu that
  does not yet exist, and actions remain full-text rather than icon-only.
- At 320px, long source-format labels wrap, code scrolls horizontally, and no
  status or independence language disappears.
- Light and dark appearance use the same hierarchy and accessible semantic
  colors.

## Content rules

- Show shipped or explicitly experimental behavior only.
- Do not use “foundation,” “official,” or “SDK” as a status claim.
- Do not expose `rocs` as a peer brand; use “Rocci Docs” and mention the command
  only in task documentation.
- Keep “without the framework tax” out of the H1. It may survive later as
  supporting editorial copy if message testing shows it helps rather than
  confuses.
- Link Community only when the feedback route and response expectations are
  actually public.

## Prototype acceptance checks

- A first-time visitor can paraphrase the product and brand hierarchy after one
  screen.
- Preview maturity and independent status are visible without opening a legal
  page.
- The primary action reaches a deterministic five-minute example.
- Keyboard focus, text zoom, 320px layout, dark mode, forced colors, reduced
  motion, and print are tested before implementation is called launch-ready.
- Page title, H1, description, visible project name, social card, and repository
  About text use the same descriptor.


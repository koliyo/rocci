# rocci-theme

CSS themes for Rocdown. A theme is a `.css` file that sets `--rd-*`
variables on `.rd-document`. Light and dark palettes use CSS
`light-dark()`; `color_scheme` `auto` | `light` | `dark` picks which side
applies.

## Selection

1. `@page { theme: "..." }`
2. `rocdown view --theme ...`
3. `ROCCI_THEME`
4. builtin `paper`

`--theme` / `@page.theme` is a **name** or a **path**:

```sh
rocdown view --theme paper foo.rocdown
rocdown view --theme path/to/theme.css foo.rocdown
rocdown view --color-scheme dark foo.rocdown
```

Named themes load from `~/.rocci/themes/{name}.css` or
`~/.rocci/themes/{name}/theme.css`. Builtins: `paper`, `rocci`, `none`.
The default `paper` palette follows One Light / One Dark Pro.

## Authoring

Copy a builtin and change variables:

```css
.rd-document {
  color-scheme: light dark;
  --rd-font-body: ui-sans-serif, system-ui, sans-serif;
  --rd-font-heading: ui-sans-serif, system-ui, sans-serif;
  --rd-font-code: ui-monospace, monospace;
  --rd-color-bg: light-dark(#fafafa, #282c34);
  --rd-color-text: light-dark(#383a42, #abb2bf);
  --rd-color-accent: light-dark(#4078f2, #61afef);
  --rd-header-1-color: var(--rd-color-text);
  --rd-paragraph-color: var(--rd-color-muted);
  --rd-link-color: var(--rd-color-accent);
}
```

Rocci always appends chrome that maps those variables onto Markdown classes
such as `rd-header-1` and `rd-paragraph`. The same chrome styles the
standalone left navigator (`.rd-toc`) when the default page shell emits one.
Below `48rem` the same heading IDs appear in a compact `<details class="rd-toc-menu">`
control; print still hides both. Wide tables scroll inside `.rd-table-wrap`.
A short script makes navigator clicks scroll
quickly instead of jumping.

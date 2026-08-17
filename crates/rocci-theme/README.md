# rocci-theme

CSS themes for Rocdown. A theme is a `.css` file that sets `--rd-*`
variables on `.rd-document`. Light and dark palettes use CSS
`light-dark()`; `color_scheme` `auto` | `light` | `dark` picks which side
applies.

## Selection

1. `@page { theme: "..." }`
2. `rocdown run --theme ...`
3. `ROCCI_THEME`
4. builtin `paper`

`--theme` / `@page.theme` is a **name** or a **path**:

```sh
rocdown run --theme paper foo.rocdown
rocdown run --theme path/to/theme.css foo.rocdown
rocdown run --color-scheme dark foo.rocdown
```

Named themes load from `~/.rocci/themes/{name}.css` or
`~/.rocci/themes/{name}/theme.css`. Builtins: `paper`, `rocci`, `none`.

## Authoring

Copy a builtin and change variables:

```css
.rd-document {
  color-scheme: light dark;
  --rd-font-body: ui-sans-serif, system-ui, sans-serif;
  --rd-font-heading: ui-sans-serif, system-ui, sans-serif;
  --rd-font-code: ui-monospace, monospace;
  --rd-color-bg: light-dark(#f7f7f5, #18181b);
  --rd-color-text: light-dark(#1c1917, #fafafa);
  --rd-color-accent: light-dark(#2563eb, #60a5fa);
  --rd-header-1-color: var(--rd-color-text);
  --rd-paragraph-color: var(--rd-color-muted);
  --rd-link-color: var(--rd-color-accent);
}
```

Rocci always appends chrome that maps those variables onto Markdown classes
such as `rd-header-1` and `rd-paragraph`. The same chrome styles the
standalone left navigator (`.rd-toc`) when the default page shell emits one,
and ships a short script so navigator clicks scroll quickly instead of jumping.

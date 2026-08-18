/**
 * Highlight bridge mapping PlaygroundHighlightSpan records to CSS classes and CodeMirror decorations.
 */

export interface PlaygroundHighlightSpan {
  from: number;
  to: number;
  kind: string;
  modifiers?: number;
}

export const MOD_DECLARATION = 1 << 0;
export const MOD_DEFAULT_LIBRARY = 1 << 1;
export const MOD_READONLY = 1 << 2;
export const MOD_DOCUMENTATION = 1 << 3;

/**
 * Returns the CSS class names for a given highlight span.
 */
export function getSpanCssClasses(span: PlaygroundHighlightSpan): string[] {
  const classes: string[] = [span.kind];
  const mods = span.modifiers || 0;
  if (mods & MOD_DECLARATION) classes.push("tok-definition");
  if (mods & MOD_DEFAULT_LIBRARY) classes.push("tok-default-library");
  if (mods & MOD_READONLY) classes.push("tok-readonly");
  if (mods & MOD_DOCUMENTATION) classes.push("tok-documentation");
  return classes;
}

/**
 * Sanitizes and validates highlight spans to ensure non-overlapping monotonic order.
 */
export function sanitizeSpans(spans: PlaygroundHighlightSpan[], maxLen: number): PlaygroundHighlightSpan[] {
  const valid = spans.filter(
    (s) => typeof s.from === "number" && typeof s.to === "number" && s.from >= 0 && s.from < s.to && s.to <= maxLen
  );
  valid.sort((a, b) => a.from - b.from || b.to - a.to);

  const result: PlaygroundHighlightSpan[] = [];
  let prevTo = 0;
  for (const span of valid) {
    if (span.from >= prevTo) {
      result.push(span);
      prevTo = span.to;
    }
  }
  return result;
}

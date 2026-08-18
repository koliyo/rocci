import { EditorState, StateEffect, StateField } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { type Diagnostic, setDiagnostics } from "@codemirror/lint";
import type { PlaygroundDiagnostic, PlaygroundHighlightSpan } from "./protocol";
import { getSpanCssClasses, sanitizeSpans } from "./highlighter";

export interface EditorOptions {
  parent: HTMLElement;
  doc?: string;
  readOnly?: boolean;
  onChange?: (value: string) => void;
}

// Effect for updating highlight decorations
const setHighlightEffect = StateEffect.define<PlaygroundHighlightSpan[]>();

// StateField managing syntax highlight decorations
const highlightField = StateField.define<DecorationSet>({
  create() {
    return Decoration.none;
  },
  update(decorations, tr) {
    decorations = decorations.map(tr.changes);
    for (const effect of tr.effects) {
      if (effect.is(setHighlightEffect)) {
        const spans = sanitizeSpans(effect.value, tr.newDoc.length);
        const decos = spans.map((span) => {
          const classes = getSpanCssClasses(span).join(" ");
          return Decoration.mark({ class: classes }).range(span.from, span.to);
        });
        decorations = Decoration.set(decos, true);
      }
    }
    return decorations;
  },
  provide: (f) => EditorView.decorations.from(f),
});

export class CodeEditor {
  private view: EditorView;
  private isReadOnly: boolean;

  constructor(options: EditorOptions) {
    this.isReadOnly = Boolean(options.readOnly);

    const extensions = [
      lineNumbers(),
      highlightActiveLineGutter(),
      highlightActiveLine(),
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      highlightField,
      EditorView.updateListener.of((update) => {
        if (update.docChanged && options.onChange && !this.isReadOnly) {
          options.onChange(update.state.doc.toString());
        }
      }),
    ];

    if (this.isReadOnly) {
      extensions.push(EditorState.readOnly.of(true));
      extensions.push(EditorView.editable.of(false));
    }

    const state = EditorState.create({
      doc: options.doc || "",
      extensions,
    });

    this.view = new EditorView({
      state,
      parent: options.parent,
    });
  }

  public getValue(): string {
    return this.view.state.doc.toString();
  }

  public setValue(text: string) {
    if (this.getValue() === text) return;
    this.view.dispatch({
      changes: { from: 0, to: this.view.state.doc.length, insert: text },
    });
  }

  public setHighlights(spans: PlaygroundHighlightSpan[]) {
    this.view.dispatch({
      effects: setHighlightEffect.of(spans),
    });
  }

  public setDiagnostics(diags: PlaygroundDiagnostic[]) {
    const cmDiags: Diagnostic[] = diags.map((d) => ({
      from: Math.min(d.from, this.view.state.doc.length),
      to: Math.min(d.to, this.view.state.doc.length),
      severity: d.severity === "error" ? "error" : "warning",
      message: d.message,
    }));
    this.view.dispatch(setDiagnostics(this.view.state, cmDiags));
  }

  public setCursor(from: number, to?: number) {
    const docLen = this.view.state.doc.length;
    const clampedFrom = Math.max(0, Math.min(from, docLen));
    const clampedTo = to !== undefined ? Math.max(0, Math.min(to, docLen)) : clampedFrom;

    this.view.dispatch({
      selection: { anchor: clampedFrom, head: clampedTo },
      scrollIntoView: true,
    });
    this.view.focus();
  }

  public focus() {
    this.view.focus();
  }

  public destroy() {
    this.view.destroy();
  }
}

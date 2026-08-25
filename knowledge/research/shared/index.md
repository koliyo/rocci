# Shared

Chrome that spans OKF, Rocdown, and Rocci products.

* [Mobile chrome for OKF, Rocdown, and rocci.dev](mobile-chrome.md) - Code-backed inventory: default Rocdown theme has a no-JS menu; rocci.dev hides the docs sidebar without replacement; OKF nests Home/Review inside a TOC that phone CSS discards. Implementation plan: [mobile chrome](/plans/shared/mobile-chrome.md). Exploratory; not shipped.
* [Editor preview for Rocci and Rocdown](editor-preview.md) - VS Code can host the CLI `--no-window` origin beside the source; Zed has no extension webview, so tasks should open the native preview window. Implementation plan: [editor preview](/plans/shared/editor-preview.md). Exploratory; v1 landed on branch `editor-preview`.
* [Hosted editor preview chrome and unbundled tools](editor-preview-host.md) - Full preview means the VS Code webview becomes the toolbar and inspector host; binaries come from Rocci GitHub releases, not the VSIX. Implementation plan: [hosted editor preview](/plans/shared/editor-preview-host.md). Exploratory; not shipped.

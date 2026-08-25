export type HostCommand =
  | 'back'
  | 'forward'
  | 'home'
  | 'reload'
  | 'toggle-live-reload'

export type IframeHistory = {
  entries: string[]
  index: number
}

export type HostState = {
  pageUrl: string
  title: string
  liveReload: boolean
  canBack: boolean
  canForward: boolean
}

export function createHistory(homeUrl: string): IframeHistory {
  return { entries: [homeUrl], index: 0 }
}

export function currentUrl(history: IframeHistory): string {
  return history.entries[history.index] ?? history.entries[0]
}

export function canGoBack(history: IframeHistory): boolean {
  return history.index > 0
}

export function canGoForward(history: IframeHistory): boolean {
  return history.index < history.entries.length - 1
}

export function goBack(history: IframeHistory): IframeHistory {
  if (!canGoBack(history)) {
    return history
  }
  return { entries: history.entries, index: history.index - 1 }
}

export function goForward(history: IframeHistory): IframeHistory {
  if (!canGoForward(history)) {
    return history
  }
  return { entries: history.entries, index: history.index + 1 }
}

export function goHome(history: IframeHistory): IframeHistory {
  return { entries: history.entries, index: 0 }
}

export function navigateTo(history: IframeHistory, url: string): IframeHistory {
  if (history.entries[history.index] === url) {
    return history
  }
  return {
    entries: [...history.entries.slice(0, history.index + 1), url],
    index: history.index + 1
  }
}

export function replaceCurrent(history: IframeHistory, url: string): IframeHistory {
  const entries = history.entries.slice()
  entries[history.index] = url
  return { entries, index: history.index }
}

export function applyLiveReloadFlag(url: string, enabled: boolean): string {
  const parsed = new URL(url)
  if (enabled) {
    parsed.searchParams.delete('reload')
  } else {
    parsed.searchParams.set('reload', '0')
  }
  return parsed.toString()
}

export function displayPath(url: string): string {
  const parsed = new URL(url)
  parsed.searchParams.delete('_r')
  const search = parsed.searchParams.toString()
  return search ? `${parsed.pathname}?${search}` : parsed.pathname
}

export function servingTitle(filePath: string): string {
  const slash = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'))
  return slash >= 0 ? filePath.slice(slash + 1) : filePath
}

export function parseHostCommand(message: unknown): HostCommand | undefined {
  if (!message || typeof message !== 'object') {
    return undefined
  }
  const type = (message as { type?: unknown }).type
  if (
    type === 'back' ||
    type === 'forward' ||
    type === 'home' ||
    type === 'reload' ||
    type === 'toggle-live-reload'
  ) {
    return type
  }
  return undefined
}

export function hostPreviewHtml(state: HostState): string {
  const page = escapeAttr(state.pageUrl)
  const path = escapeHtml(displayPath(state.pageUrl))
  const title = escapeHtml(state.title)
  const livePressed = state.liveReload ? 'true' : 'false'
  const liveClass = state.liveReload ? 'is-on' : ''
  return `<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; frame-src http: https:; style-src 'unsafe-inline'; script-src 'unsafe-inline';" />
    <style>
      :root {
        --rocci-toolbar: 48px;
        color-scheme: light dark;
      }
      html, body { margin: 0; height: 100%; }
      body {
        display: flex;
        flex-direction: column;
        font-family: var(--vscode-font-family, system-ui);
        color: var(--vscode-foreground, #ccc);
        background: var(--vscode-editor-background, #1e1e1e);
      }
      .toolbar {
        box-sizing: border-box;
        height: var(--rocci-toolbar);
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 0 10px;
        border-bottom: 1px solid var(--vscode-panel-border, #333);
      }
      .group { display: flex; gap: 2px; }
      .divider { width: 1px; height: 20px; background: var(--vscode-panel-border, #333); }
      button {
        background: transparent;
        color: inherit;
        border: 0;
        border-radius: 4px;
        padding: 4px 6px;
        cursor: pointer;
      }
      button:disabled { opacity: 0.35; cursor: default; }
      button:not(:disabled):hover { background: var(--vscode-toolbar-hoverBackground, #333); }
      button.is-on { color: var(--vscode-textLink-foreground, #4daafc); }
      .meta { flex: 1; min-width: 0; }
      .path, .title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
      .path { font-size: 12px; }
      .title { font-size: 11px; opacity: 0.7; }
      iframe { flex: 1; width: 100%; border: 0; background: #fff; }
    </style>
  </head>
  <body>
    <div class="toolbar" role="toolbar" aria-label="Rocci preview">
      <div class="group">
        <button type="button" data-cmd="back" aria-label="Back"${state.canBack ? '' : ' disabled'}>◀</button>
        <button type="button" data-cmd="forward" aria-label="Forward"${state.canForward ? '' : ' disabled'}>▶</button>
      </div>
      <div class="divider"></div>
      <div class="group">
        <button type="button" data-cmd="home" aria-label="Home">⌂</button>
        <button type="button" data-cmd="reload" aria-label="Reload">↻</button>
        <button type="button" data-cmd="toggle-live-reload" class="${liveClass}" aria-label="Live reload" aria-pressed="${livePressed}">⚡</button>
      </div>
      <div class="meta">
        <div class="path" id="path">${path}</div>
        <div class="title" id="title">${title}</div>
      </div>
    </div>
    <iframe id="page" src="${page}"></iframe>
    <script>
      const vscode = acquireVsCodeApi();
      for (const button of document.querySelectorAll('[data-cmd]')) {
        button.addEventListener('click', () => {
          vscode.postMessage({ type: button.getAttribute('data-cmd') });
        });
      }
    </script>
  </body>
</html>`
}

function escapeAttr(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;')
}

function escapeHtml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

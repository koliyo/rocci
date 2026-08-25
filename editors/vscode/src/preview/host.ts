import { dockClassNames, InspectorPrefs } from './inspector'

export type HostCommand =
  | 'back'
  | 'forward'
  | 'home'
  | 'reload'
  | 'toggle-live-reload'
  | 'toggle-dev'
  | 'dock-right'
  | 'dock-bottom'
  | 'open-as-page'
  | 'split'

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
  inspectorUrl?: string
  inspectorSrc?: string
  prefs: InspectorPrefs
  asPage: boolean
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
    type === 'toggle-live-reload' ||
    type === 'toggle-dev' ||
    type === 'dock-right' ||
    type === 'dock-bottom' ||
    type === 'open-as-page' ||
    type === 'split'
  ) {
    return type
  }
  return undefined
}

export function parseSplitSize(message: unknown): { dock: 'right' | 'bottom'; size: string } | undefined {
  if (!message || typeof message !== 'object') {
    return undefined
  }
  const value = message as { type?: unknown; dock?: unknown; size?: unknown }
  if (value.type !== 'split' || (value.dock !== 'right' && value.dock !== 'bottom')) {
    return undefined
  }
  if (typeof value.size !== 'string' || !value.size) {
    return undefined
  }
  return { dock: value.dock, size: value.size }
}

export function hostPreviewHtml(state: HostState): string {
  const page = escapeAttr(state.pageUrl)
  const path = escapeHtml(displayPath(state.pageUrl))
  const title = escapeHtml(state.title)
  const livePressed = state.liveReload ? 'true' : 'false'
  const liveClass = state.liveReload ? 'is-on' : ''
  const hasInspector = Boolean(state.inspectorUrl)
  const inspectorSrc = state.inspectorSrc ? escapeAttr(state.inspectorSrc) : ''
  const dock = dockClassNames(state.prefs, state.asPage)
  const devHidden = hasInspector ? '' : ' hidden'
  const devPressed = state.prefs.open && !state.asPage ? 'true' : 'false'
  return `<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; frame-src http: https:; style-src 'unsafe-inline'; script-src 'unsafe-inline';" />
    <style>
      :root {
        --rocci-toolbar: 48px;
        --rocci-dock-right: ${escapeAttr(state.prefs.right)};
        --rocci-dock-bottom: ${escapeAttr(state.prefs.bottom)};
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
      .stage { flex: 1; display: flex; min-height: 0; position: relative; }
      body.dock-bottom.dev-open .stage { flex-direction: column; }
      #page { flex: 1; width: 100%; border: 0; background: #fff; min-width: 0; min-height: 0; }
      .splitter {
        display: none;
        background: var(--vscode-panel-border, #333);
        flex: 0 0 6px;
        cursor: col-resize;
        touch-action: none;
        z-index: 1;
      }
      body.dock-bottom.dev-open .splitter { cursor: row-resize; }
      body.is-resizing, body.is-resizing * { user-select: none; }
      body.is-resizing iframe { pointer-events: none; }
      body.is-resizing { cursor: col-resize; }
      body.is-resizing.dock-bottom { cursor: row-resize; }
      .inspector {
        display: none;
        flex-direction: column;
        min-width: 12rem;
        min-height: 6rem;
        background: var(--vscode-editor-background, #1e1e1e);
      }
      body.dock-right.dev-open .inspector { width: var(--rocci-dock-right); flex: 0 0 var(--rocci-dock-right); }
      body.dock-bottom.dev-open .inspector { height: var(--rocci-dock-bottom); flex: 0 0 var(--rocci-dock-bottom); min-width: 0; }
      body.dev-open .inspector { display: flex; }
      body.dev-open .splitter { display: block; }
      body.as-page .inspector, body.as-page .splitter { display: none; }
      #inspector { width: 100%; height: 100%; border: 0; flex: 1; min-height: 0; }
      .inspector-bar { display: flex; gap: 4px; padding: 4px; }
    </style>
  </head>
  <body class="${dock}">
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
      <button type="button" data-cmd="toggle-dev" class="${state.prefs.open ? 'is-on' : ''}" aria-label="Developer panel" aria-pressed="${devPressed}"${devHidden}>Dev</button>
    </div>
    <div class="stage">
      <iframe id="page" src="${page}"></iframe>
      <div class="splitter" id="splitter" role="separator"></div>
      <div class="inspector" id="inspector-dock">
        <div class="inspector-bar">
          <button type="button" data-cmd="dock-right" aria-label="Dock right">Right</button>
          <button type="button" data-cmd="dock-bottom" aria-label="Dock bottom">Bottom</button>
          <button type="button" data-cmd="open-as-page" aria-label="Open as page">Open as page</button>
        </div>
        <iframe id="inspector"${inspectorSrc ? ` src="${inspectorSrc}"` : ''}></iframe>
      </div>
    </div>
    <script>
      const vscode = acquireVsCodeApi();
      for (const button of document.querySelectorAll('[data-cmd]')) {
        button.addEventListener('click', () => {
          vscode.postMessage({ type: button.getAttribute('data-cmd') });
        });
      }
      window.addEventListener('message', event => {
        const data = event.data;
        if (data && data.type === 'rocci-inspector') {
          vscode.postMessage({ type: 'inspector', tab: data.tab, view: data.view });
        }
      });
      const splitter = document.getElementById('splitter');
      const stage = document.querySelector('.stage');
      let drag = false;
      let lastSplit = null;
      function measureSplit(event) {
        const rect = stage.getBoundingClientRect();
        if (document.body.classList.contains('dock-right')) {
          const px = Math.min(rect.width - 80, Math.max(192, rect.right - event.clientX));
          const size = (px / 16) + 'rem';
          document.documentElement.style.setProperty('--rocci-dock-right', size);
          return { dock: 'right', size };
        }
        const px = Math.min(rect.height - 80, Math.max(128, rect.bottom - event.clientY));
        const size = (px / Math.max(1, window.innerHeight) * 100) + 'vh';
        document.documentElement.style.setProperty('--rocci-dock-bottom', size);
        return { dock: 'bottom', size };
      }
      function endDrag() {
        if (!drag) { return; }
        drag = false;
        document.body.classList.remove('is-resizing');
        if (lastSplit) {
          vscode.postMessage({ type: 'split', dock: lastSplit.dock, size: lastSplit.size });
          lastSplit = null;
        }
      }
      splitter.addEventListener('pointerdown', event => {
        if (!document.body.classList.contains('dev-open') || document.body.classList.contains('as-page')) {
          return;
        }
        drag = true;
        lastSplit = measureSplit(event);
        document.body.classList.add('is-resizing');
        splitter.setPointerCapture(event.pointerId);
        event.preventDefault();
      });
      splitter.addEventListener('pointermove', event => {
        if (!drag) { return; }
        lastSplit = measureSplit(event);
      });
      splitter.addEventListener('pointerup', endDrag);
      splitter.addEventListener('pointercancel', endDrag);
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

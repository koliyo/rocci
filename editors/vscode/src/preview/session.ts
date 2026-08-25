import { ChildProcess, spawn } from 'child_process'
import {
  commands,
  ExtensionContext,
  OutputChannel,
  StatusBarAlignment,
  StatusBarItem,
  ViewColumn,
  WebviewPanel,
  env,
  Uri,
  window,
  workspace
} from 'vscode'
import { resolvePreviewBinary } from './binaries'
import { canPreviewDocument } from './browser'
import { previewArgv, PreviewProduct } from './dispatch'
import {
  applyLiveReloadFlag,
  canGoBack,
  canGoForward,
  createHistory,
  currentUrl,
  goBack,
  goForward,
  goHome,
  hostPreviewHtml,
  IframeHistory,
  navigateTo,
  parseHostCommand,
  parseSplitSize,
  replaceCurrent,
  servingTitle
} from './host'
import {
  applyInspectorMessage,
  DEFAULT_INSPECTOR_PREFS,
  INSPECTOR_STATE_KEY,
  InspectorPrefs,
  InspectorTuple,
  inspectorHref,
  inspectorTuple,
  readInspectorPrefs,
  shouldAssignInspectorSrc
} from './inspector'
import { belongsToOrigin, navigateUrl, PreviewOrigin, previewOrigin, reuseDecision } from './origin'
import { parseInspectorUrl, parsePreviewUrl } from './parse'
import { countPreviewReadyLines, countRebuildLines, withReloadNonce } from './reload'
import { PreviewReloadStream } from './sse'

const READY_TIMEOUT_MS = 120_000
const ACTIVE_CONTEXT = 'rocci.preview.active'

export class PreviewSession {
  private child: ChildProcess | undefined
  private origin: PreviewOrigin | undefined
  private filePath: string | undefined
  private url: string | undefined
  private inspectorUrl: string | undefined
  private inspectorAssigned: InspectorTuple | undefined
  private inspectorSrc: string | undefined
  private asPage = false
  private prefs: InspectorPrefs = { ...DEFAULT_INSPECTOR_PREFS }
  private history: IframeHistory | undefined
  private liveReload = true
  private product: PreviewProduct | undefined
  private stopping = false
  private panel: WebviewPanel | undefined
  private readonly output: OutputChannel
  private readonly status: StatusBarItem
  private readonly reloadStream = new PreviewReloadStream(
    () => {
      if (this.liveReload) {
        void this.reload()
      }
    },
    message => this.log(message)
  )
  private reloadNonce = 0
  private reloadTimer: NodeJS.Timeout | undefined
  private readyLines = 0
  private rebuildLines = 0
  private saveTimer: NodeJS.Timeout | undefined
  private ignoreSavesUntil = 0

  constructor(private readonly context: ExtensionContext) {
    this.output = window.createOutputChannel('Rocci Preview')
    this.status = window.createStatusBarItem(StatusBarAlignment.Left, 50)
    this.status.command = 'rocci.stopPreview'
    this.status.tooltip = 'Stop Rocci preview'
    this.prefs = readInspectorPrefs(this.context.workspaceState.get(INSPECTOR_STATE_KEY))
    this.context.subscriptions.push(
      this.output,
      this.status,
      workspace.onDidSaveTextDocument(document => {
        void this.onSaved(document.uri.fsPath)
      })
    )
  }

  get running(): boolean {
    return this.child !== undefined && this.child.exitCode === null
  }

  async preview(): Promise<void> {
    const editor = window.activeTextEditor
    if (!editor) {
      await window.showErrorMessage('Open a .rocci or .rocdown file to preview.')
      return
    }
    if (!canPreviewDocument(editor.document.uri.scheme, editor.document.uri.fsPath)) {
      const message = 'Preview requires a saved file. Untitled buffers cannot be served.'
      this.log(message)
      await window.showErrorMessage(message)
      return
    }
    const filePath = editor.document.uri.fsPath
    const origin = previewOrigin(filePath)
    const argv = previewArgv(filePath)
    if (!origin || !argv) {
      await window.showErrorMessage('Preview supports .rocci and .rocdown files.')
      return
    }
    this.ignoreSavesUntil = Date.now() + 2000
    if (editor.document.isDirty) {
      await editor.document.save()
    }

    const action = reuseDecision(this.running ? this.origin : undefined, origin)
    if (action === 'reuse' && this.url && this.history && argv.product === 'rocdown') {
      const url = applyLiveReloadFlag(navigateUrl(this.url, filePath, origin), this.liveReload)
      this.url = url
      this.filePath = filePath
      this.history = navigateTo(this.history, url)
      this.log(`navigate ${url}`)
      await this.renderHost()
      return
    }

    await this.start(filePath, origin, argv.product)
  }

  async reload(): Promise<void> {
    if (!this.running || !this.url) {
      this.log('reload ignored (no preview)')
      return
    }
    if (this.reloadTimer) {
      clearTimeout(this.reloadTimer)
    }
    this.reloadTimer = setTimeout(() => {
      this.reloadTimer = undefined
      void this.flushReload()
    }, 150)
  }

  private async flushReload(): Promise<void> {
    if (!this.running || !this.url) {
      return
    }
    this.reloadNonce += 1
    const url = applyLiveReloadFlag(withReloadNonce(this.url, this.reloadNonce), this.liveReload)
    this.url = url
    if (this.history) {
      this.history = replaceCurrent(this.history, url)
    }
    this.log(`reload ${url}`)
    await this.renderHost()
  }

  async stop(): Promise<void> {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer)
      this.saveTimer = undefined
    }
    if (this.reloadTimer) {
      clearTimeout(this.reloadTimer)
      this.reloadTimer = undefined
    }
    this.reloadStream.stop()
    const child = this.child
    this.child = undefined
    this.origin = undefined
    this.filePath = undefined
    this.url = undefined
    this.inspectorUrl = undefined
    this.inspectorAssigned = undefined
    this.inspectorSrc = undefined
    this.asPage = false
    this.history = undefined
    this.liveReload = true
    this.product = undefined
    this.readyLines = 0
    this.rebuildLines = 0
    await this.setActive(false)
    if (!child || child.exitCode !== null) {
      return
    }
    this.stopping = true
    this.log(`stop pid ${child.pid ?? '?'}`)
    killProcessTree(child)
    await waitForExit(child, 2000)
  }

  dispose(): void {
    this.panel?.dispose()
    void this.stop()
  }

  private async start(
    filePath: string,
    origin: PreviewOrigin,
    product: PreviewProduct
  ): Promise<void> {
    const argv = previewArgv(filePath)
    if (!argv) {
      return
    }
    const binary = resolvePreviewBinary(this.context, argv.product)
    if (!binary) {
      const name = argv.product === 'rocci' ? 'rocci' : 'rocdown'
      const message = `${name} not found. Set rocci.preview.${argv.product}Path or add it to PATH.`
      this.log(message)
      await window.showErrorMessage(message)
      return
    }

    await this.stop()
    this.stopping = false
    this.ignoreSavesUntil = Date.now() + 2000
    this.output.show(true)
    this.log(`${binary} ${argv.args.join(' ')}`)

    const child = spawn(binary, argv.args, {
      detached: process.platform !== 'win32',
      env: process.env,
      stdio: ['ignore', 'pipe', 'pipe']
    })
    this.child = child
    this.origin = origin
    this.filePath = filePath
    this.product = product
    this.log(`spawn pid ${child.pid ?? '?'}`)

    let buffer = ''
    const onData = (chunk: Buffer) => {
      const text = chunk.toString('utf8')
      buffer += text
      this.output.append(text)
      this.noteCliProgress(buffer)
    }
    child.stdout?.on('data', onData)
    child.stderr?.on('data', onData)
    child.on('error', err => {
      this.log(String(err))
    })
    child.on('exit', (code, signal) => {
      if (this.child === child) {
        this.child = undefined
        this.origin = undefined
        this.url = undefined
        this.reloadStream.stop()
        void this.setActive(false)
      }
      if (!this.stopping) {
        this.log(`preview exited (${code ?? signal ?? 'unknown'})`)
      }
    })

    const url = await this.waitForUrl(child, () => buffer)
    if (!url) {
      await this.stop()
      return
    }
    this.url = applyLiveReloadFlag(url, this.liveReload)
    this.history = createHistory(this.url)
    this.noteInspector(buffer)
    this.readyLines = Math.max(1, countPreviewReadyLines(buffer))
    this.rebuildLines = countRebuildLines(buffer)
    this.syncWatch()
    if (product === 'rocci') {
      this.log('watch skipped (rocci run does not rebuild; save restarts preview)')
    }
    await this.setActive(true)
    await this.renderHost()
  }

  private async waitForUrl(
    child: ChildProcess,
    readBuffer: () => string
  ): Promise<string | undefined> {
    const deadline = Date.now() + READY_TIMEOUT_MS
    while (Date.now() < deadline) {
      const url = parsePreviewUrl(readBuffer())
      if (url) {
        this.log(`ready ${url}`)
        return url
      }
      if (child.exitCode !== null) {
        const message = 'Preview process exited before a listen URL was printed.'
        this.log(message)
        await window.showErrorMessage(message)
        return undefined
      }
      await delay(50)
    }
    const message = 'Timed out waiting for a preview URL on CLI output.'
    this.log(message)
    await window.showErrorMessage(message)
    return undefined
  }

  private async renderHost(): Promise<void> {
    if (!this.url || !this.history) {
      return
    }
    if (!this.panel) {
      this.panel = window.createWebviewPanel(
        'rocciPreview',
        'Rocci Preview',
        { viewColumn: ViewColumn.Beside, preserveFocus: true },
        { enableScripts: true, retainContextWhenHidden: true }
      )
      this.panel.webview.onDidReceiveMessage(message => {
        void this.onHostMessage(message)
      })
      this.panel.onDidDispose(() => {
        this.panel = undefined
      })
    }
    this.syncInspectorSrc(false)
    this.panel.webview.html = hostPreviewHtml({
      pageUrl: currentUrl(this.history),
      title: servingTitle(this.filePath ?? ''),
      liveReload: this.liveReload,
      canBack: canGoBack(this.history),
      canForward: canGoForward(this.history),
      inspectorUrl: this.inspectorUrl,
      inspectorSrc: this.prefs.open && !this.asPage ? this.inspectorSrc : undefined,
      prefs: this.prefs,
      asPage: this.asPage,
      canReveal: Boolean(this.filePath)
    })
    this.panel.reveal(ViewColumn.Beside, true)
  }

  private async onHostMessage(message: unknown): Promise<void> {
    if (message && typeof message === 'object' && (message as { type?: unknown }).type === 'inspector') {
      const applied = applyInspectorMessage(this.prefs, message as { tab?: unknown; view?: unknown })
      this.prefs = applied.prefs
      await this.persistPrefs()
      if (!applied.viewOnly) {
        this.syncInspectorSrc(true)
        await this.renderHost()
      }
      return
    }
    const split = parseSplitSize(message)
    if (split) {
      if (split.dock === 'right') {
        this.prefs.right = split.size
      } else {
        this.prefs.bottom = split.size
      }
      await this.persistPrefs()
      return
    }
    const command = parseHostCommand(message)
    if (!command || !this.history || !this.url) {
      return
    }
    if (command === 'reload') {
      await this.reload()
      return
    }
    if (command === 'toggle-dev') {
      if (!this.inspectorUrl) {
        return
      }
      this.asPage = false
      this.prefs.open = !this.prefs.open
      await this.persistPrefs()
      this.syncInspectorSrc(true)
      await this.renderHost()
      return
    }
    if (command === 'dock-right' || command === 'dock-bottom') {
      this.prefs.dock = command === 'dock-right' ? 'right' : 'bottom'
      await this.persistPrefs()
      await this.renderHost()
      return
    }
    if (command === 'open-as-page' && this.inspectorUrl) {
      this.asPage = true
      this.prefs.open = false
      await this.persistPrefs()
      this.history = navigateTo(this.history, this.inspectorUrl)
      this.url = this.inspectorUrl
      await this.renderHost()
      return
    }
    if (command === 'reveal' && this.filePath) {
      await commands.executeCommand('revealFileInOS', Uri.file(this.filePath))
      return
    }
    if (command === 'copy' && this.filePath) {
      const bytes = await workspace.fs.readFile(Uri.file(this.filePath))
      await env.clipboard.writeText(Buffer.from(bytes).toString('utf8'))
      return
    }
    if (command === 'toggle-live-reload') {
      this.liveReload = !this.liveReload
      const url = applyLiveReloadFlag(this.url, this.liveReload)
      this.url = url
      this.history = replaceCurrent(this.history, url)
      this.syncWatch()
      this.log(`live-reload ${this.liveReload ? 'on' : 'off'}`)
      await this.renderHost()
      return
    }
    if (command === 'back') {
      this.history = goBack(this.history)
    } else if (command === 'forward') {
      this.history = goForward(this.history)
    } else {
      this.history = goHome(this.history)
    }
    this.url = currentUrl(this.history)
    this.asPage = Boolean(
      this.inspectorUrl && new URL(this.url).pathname === new URL(this.inspectorUrl).pathname
    )
    this.log(`${command} ${this.url}`)
    await this.renderHost()
  }

  private syncWatch(): void {
    if (this.product === 'rocdown' && this.liveReload && this.url) {
      this.reloadStream.start(this.url)
      return
    }
    this.reloadStream.stop()
  }

  private noteCliProgress(buffer: string): void {
    const rebuilds = countRebuildLines(buffer)
    if (this.url && rebuilds > this.rebuildLines) {
      this.rebuildLines = rebuilds
      this.log('cli rebuild')
      void this.reload()
    }
    const ready = countPreviewReadyLines(buffer)
    if (this.url && ready > this.readyLines) {
      this.readyLines = ready
      this.log('cli preview_ready')
      void this.reload()
    }
    this.noteInspector(buffer)
  }

  private noteInspector(buffer: string): void {
    const url = parseInspectorUrl(buffer)
    if (!url || url === this.inspectorUrl) {
      return
    }
    this.inspectorUrl = url
    this.log(`inspector ${url}`)
    if (this.history) {
      void this.renderHost()
    }
  }

  private syncInspectorSrc(force: boolean): void {
    if (!this.inspectorUrl || !this.url || !this.prefs.open || this.asPage) {
      return
    }
    const next = inspectorTuple(this.inspectorUrl, this.url, this.prefs.tab)
    if (!force && !shouldAssignInspectorSrc(this.inspectorAssigned, next)) {
      return
    }
    this.inspectorAssigned = next
    this.inspectorSrc = inspectorHref(this.inspectorUrl, next, true, this.prefs.view)
  }

  private async persistPrefs(): Promise<void> {
    await this.context.workspaceState.update(INSPECTOR_STATE_KEY, this.prefs)
  }

  private async onSaved(filePath: string): Promise<void> {
    if (!this.running || !this.origin || Date.now() < this.ignoreSavesUntil) {
      return
    }
    if (!belongsToOrigin(filePath, this.origin)) {
      return
    }
    this.log(`saved ${filePath}`)
    if (this.product === 'rocci') {
      const origin = this.origin
      const product = this.product
      const target = this.filePath ?? filePath
      this.log('restarting rocci run after save')
      await this.start(target, origin, product)
      return
    }
    if (this.saveTimer) {
      clearTimeout(this.saveTimer)
    }
    this.saveTimer = setTimeout(() => {
      this.saveTimer = undefined
      void this.reload()
    }, 800)
  }

  private log(message: string): void {
    this.output.appendLine(`preview: ${message}`)
  }

  private async setActive(active: boolean): Promise<void> {
    await commands.executeCommand('setContext', ACTIVE_CONTEXT, active)
    if (active) {
      this.status.text = '$(radio-tower) Rocci Preview'
      this.status.show()
    } else {
      this.status.hide()
    }
  }
}

export function killProcessTree(child: ChildProcess): void {
  const pid = child.pid
  if (pid === undefined) {
    return
  }
  if (process.platform === 'win32') {
    spawn('taskkill', ['/pid', String(pid), '/t', '/f'])
    return
  }
  try {
    process.kill(-pid, 'SIGTERM')
  } catch {
    child.kill('SIGTERM')
  }
}

function waitForExit(child: ChildProcess, timeoutMs: number): Promise<void> {
  if (child.exitCode !== null) {
    return Promise.resolve()
  }
  return new Promise(resolve => {
    const timer = setTimeout(() => {
      child.kill('SIGKILL')
      resolve()
    }, timeoutMs)
    child.once('exit', () => {
      clearTimeout(timer)
      resolve()
    })
  })
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

export function registerPreviewCommands(
  context: ExtensionContext,
  session: PreviewSession
): void {
  context.subscriptions.push(
    session,
    commands.registerCommand('rocci.preview', () => session.preview()),
    commands.registerCommand('rocci.reloadPreview', () => session.reload()),
    commands.registerCommand('rocci.stopPreview', () => session.stop())
  )
}

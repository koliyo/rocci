import { ChildProcess, spawn } from 'child_process'
import {
  commands,
  ExtensionContext,
  StatusBarAlignment,
  StatusBarItem,
  ViewColumn,
  WebviewPanel,
  window,
  workspace
} from 'vscode'

import { wrappedOutput } from '../output-channels'
import { resolvePreviewBinary } from './binaries'
import { canPreviewDocument, chooseBrowserHost, iframePreviewHtml } from './browser'
import { previewArgv } from './dispatch'
import { navigateUrl, PreviewOrigin, previewOrigin, reuseDecision, sameOrigin } from './origin'
import { parsePreviewUrl } from './parse'
import { countPreviewReadyLines, withReloadNonce } from './reload'
import { PreviewReloadStream } from './sse'

const READY_TIMEOUT_MS = 120_000
const ACTIVE_CONTEXT = 'rocci.preview.active'

export class PreviewSession {
  private child: ChildProcess | undefined
  private origin: PreviewOrigin | undefined
  private url: string | undefined
  private stopping = false
  private panel: WebviewPanel | undefined
  private readonly status: StatusBarItem
  private readonly reloadStream = new PreviewReloadStream(() => {
    void this.reload()
  })
  private reloadNonce = 0
  private readyLines = 0
  private saveTimer: NodeJS.Timeout | undefined

  constructor(private readonly context: ExtensionContext) {
    this.status = window.createStatusBarItem(StatusBarAlignment.Left, 50)
    this.status.command = 'rocci.stopPreview'
    this.status.tooltip = 'Stop Rocci preview'
    this.context.subscriptions.push(
      this.status,
      workspace.onDidSaveTextDocument(document => this.onSaved(document.uri.fsPath))
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
      wrappedOutput.appendLine(message)
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
    if (editor.document.isDirty) {
      await editor.document.save()
    }

    const action = reuseDecision(this.running ? this.origin : undefined, origin)
    if (action === 'reuse' && this.url) {
      const url = navigateUrl(this.url, filePath, origin)
      this.url = url
      await this.openBrowser(url)
      return
    }

    const binary = resolvePreviewBinary(this.context, argv.product)
    if (!binary) {
      const name = argv.product === 'rocci' ? 'rocci' : 'rocdown'
      const message = `${name} not found. Set rocci.preview.${argv.product}Path or add it to PATH.`
      wrappedOutput.appendLine(message)
      await window.showErrorMessage(message)
      return
    }

    await this.stop()
    this.stopping = false
    wrappedOutput.appendLine(`${binary} ${argv.args.join(' ')}`)

    const child = spawn(binary, argv.args, {
      detached: process.platform !== 'win32',
      env: process.env,
      stdio: ['ignore', 'pipe', 'pipe']
    })
    this.child = child
    this.origin = origin

    let buffer = ''
    const onData = (chunk: Buffer) => {
      const text = chunk.toString('utf8')
      buffer += text
      wrappedOutput.append(text)
      this.noteReadyLines(buffer)
    }
    child.stdout?.on('data', onData)
    child.stderr?.on('data', onData)
    child.on('error', err => {
      wrappedOutput.appendLine(String(err))
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
        wrappedOutput.appendLine(`preview exited (${code ?? signal ?? 'unknown'})`)
      }
    })

    const url = await this.waitForUrl(child, () => buffer)
    if (!url) {
      await this.stop()
      return
    }
    this.url = url
    this.readyLines = Math.max(1, countPreviewReadyLines(buffer))
    this.reloadStream.start(url)
    await this.setActive(true)
    await this.openBrowser(url)
  }

  async reload(): Promise<void> {
    if (!this.running || !this.url) {
      return
    }
    this.reloadNonce += 1
    await this.openBrowser(withReloadNonce(this.url, this.reloadNonce))
  }

  async stop(): Promise<void> {
    if (this.saveTimer) {
      clearTimeout(this.saveTimer)
      this.saveTimer = undefined
    }
    this.reloadStream.stop()
    const child = this.child
    this.child = undefined
    this.origin = undefined
    this.url = undefined
    this.readyLines = 0
    await this.setActive(false)
    if (!child || child.exitCode !== null) {
      return
    }
    this.stopping = true
    killProcessTree(child)
    await waitForExit(child, 2000)
  }

  dispose(): void {
    this.panel?.dispose()
    void this.stop()
  }

  private async waitForUrl(
    child: ChildProcess,
    readBuffer: () => string
  ): Promise<string | undefined> {
    const deadline = Date.now() + READY_TIMEOUT_MS
    while (Date.now() < deadline) {
      const url = parsePreviewUrl(readBuffer())
      if (url) {
        return url
      }
      if (child.exitCode !== null) {
        const message = 'Preview process exited before a listen URL was printed.'
        wrappedOutput.appendLine(message)
        await window.showErrorMessage(message)
        return undefined
      }
      await delay(50)
    }
    const message = 'Timed out waiting for a preview URL on CLI output.'
    wrappedOutput.appendLine(message)
    await window.showErrorMessage(message)
    return undefined
  }

  private async openBrowser(url: string): Promise<void> {
    const known = await commands.getCommands(true)
    const host = chooseBrowserHost(known.includes('simpleBrowser.api.open'))
    if (host === 'simpleBrowser') {
      try {
        await commands.executeCommand('simpleBrowser.api.open', url, {
          viewColumn: ViewColumn.Beside,
          preserveFocus: true
        })
        return
      } catch (err) {
        wrappedOutput.appendLine(`Simple Browser unavailable: ${err}`)
      }
    }
    this.openIframe(url)
  }

  private openIframe(url: string): void {
    if (!this.panel) {
      this.panel = window.createWebviewPanel(
        'rocciPreview',
        'Rocci Preview',
        { viewColumn: ViewColumn.Beside, preserveFocus: true },
        { enableScripts: true, retainContextWhenHidden: true }
      )
      this.panel.onDidDispose(() => {
        this.panel = undefined
      })
    }
    this.panel.webview.html = iframePreviewHtml(url)
    this.panel.reveal(ViewColumn.Beside, true)
  }

  private noteReadyLines(buffer: string): void {
    const count = countPreviewReadyLines(buffer)
    if (this.url && count > this.readyLines) {
      this.readyLines = count
      void this.reload()
    }
  }

  private onSaved(filePath: string): void {
    if (!this.running || !this.origin) {
      return
    }
    const origin = previewOrigin(filePath)
    if (!origin || !sameOrigin(origin, this.origin)) {
      return
    }
    if (this.saveTimer) {
      clearTimeout(this.saveTimer)
    }
    this.saveTimer = setTimeout(() => {
      this.saveTimer = undefined
      void this.reload()
    }, 600)
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

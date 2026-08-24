import { ChildProcess, spawn } from 'child_process'
import { commands, ExtensionContext, ViewColumn, window } from 'vscode'

import { wrappedOutput } from '../output-channels'
import { resolvePreviewBinary } from './binaries'
import { previewArgv } from './dispatch'
import { parsePreviewUrl } from './parse'

const READY_TIMEOUT_MS = 120_000

export class PreviewSession {
  private child: ChildProcess | undefined
  private stopping = false

  constructor(private readonly context: ExtensionContext) {}

  get running(): boolean {
    return this.child !== undefined && this.child.exitCode === null
  }

  async preview(): Promise<void> {
    const editor = window.activeTextEditor
    if (!editor) {
      await window.showErrorMessage('Open a .rocci or .rocdown file to preview.')
      return
    }
    const filePath = editor.document.uri.scheme === 'file' ? editor.document.uri.fsPath : undefined
    if (!filePath) {
      await window.showErrorMessage('Save the file before previewing.')
      return
    }
    const argv = previewArgv(filePath)
    if (!argv) {
      await window.showErrorMessage('Preview supports .rocci and .rocdown files.')
      return
    }
    if (editor.document.isDirty) {
      await editor.document.save()
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

    let buffer = ''
    const onData = (chunk: Buffer) => {
      const text = chunk.toString('utf8')
      buffer += text
      wrappedOutput.append(text)
    }
    child.stdout?.on('data', onData)
    child.stderr?.on('data', onData)
    child.on('error', err => {
      wrappedOutput.appendLine(String(err))
    })
    child.on('exit', (code, signal) => {
      if (this.child === child) {
        this.child = undefined
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
    await this.openBrowser(url)
  }

  async stop(): Promise<void> {
    const child = this.child
    this.child = undefined
    if (!child || child.exitCode !== null) {
      return
    }
    this.stopping = true
    killProcessTree(child)
    await waitForExit(child, 2000)
  }

  dispose(): void {
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
    await commands.executeCommand('simpleBrowser.api.open', url, {
      viewColumn: ViewColumn.Beside,
      preserveFocus: true
    })
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
    commands.registerCommand('rocci.stopPreview', () => session.stop())
  )
}

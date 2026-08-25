import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { commands, ExtensionContext, window, workspace } from 'vscode'
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  RevealOutputChannelOn,
  ServerOptions,
  Trace,
  TransportKind
} from 'vscode-languageclient/node'

import { createOutputChannels, wrappedOutput } from './output-channels'
import { PreviewSession, registerPreviewCommands } from './preview/session'

let client: LanguageClient | undefined
let previewSession: PreviewSession | undefined
const isDebug = process.env.VSCODE_DEBUG_MODE !== undefined

function lspExecutableName(): string {
  return os.type() === 'Windows_NT' ? 'rocci-language-server.exe' : 'rocci-language-server'
}

function findOnPath(name: string): string | undefined {
  const envPath = process.env.PATH ?? ''
  for (const dir of envPath.split(path.delimiter)) {
    if (!dir) {
      continue
    }
    const candidate = path.join(dir, name)
    if (fs.existsSync(candidate)) {
      return candidate
    }
  }
  return undefined
}

function resolveServerPath(context: ExtensionContext): string | undefined {
  const configured = workspace.getConfiguration('rocci').get<string>('lsp.serverPath')?.trim()
  if (configured) {
    return configured
  }

  const exe = lspExecutableName()

  if (isDebug) {
    const debugPath = path.join(context.extensionPath, '..', '..', 'target', 'debug', exe)
    if (fs.existsSync(debugPath)) {
      return debugPath
    }
  }

  const bundled = path.join(context.extensionPath, 'dist', 'bin', exe)
  if (fs.existsSync(bundled)) {
    return bundled
  }

  return findOnPath(exe)
}

function traceFromConfig(): Trace {
  const value = workspace.getConfiguration('rocci').get<string>('lsp.trace.server', 'off')
  switch (value) {
    case 'verbose':
      return Trace.Verbose
    case 'messages':
      return Trace.Messages
    default:
      return Trace.Off
  }
}

async function startClient(context: ExtensionContext) {
  const serverPath = resolveServerPath(context)
  if (!serverPath) {
    const message = 'rocci-language-server not found. Build it with `cargo build -p rocci-rocdown-lsp` or set rocci.lsp.serverPath.'
    wrappedOutput.appendLine(message)
    await window.showErrorMessage(message)
    return
  }

  wrappedOutput.appendLine(`Language server: ${serverPath}`)

  const executable: Executable = {
    command: serverPath,
    args: [],
    transport: TransportKind.stdio
  }

  const serverOptions: ServerOptions = {
    run: executable,
    debug: executable
  }

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { language: 'rocci' },
      { pattern: '**/*.rocci' },
      { language: 'rocdown' },
      { pattern: '**/*.rocdown' }
    ],
    synchronize: {
      configurationSection: 'rocci',
      fileEvents: [
        workspace.createFileSystemWatcher('**/*.rocci'),
        workspace.createFileSystemWatcher('**/*.rocdown')
      ]
    },
    outputChannel: wrappedOutput,
    revealOutputChannelOn: RevealOutputChannelOn.Info
  }

  client = new LanguageClient('rocci', 'Rocci', serverOptions, clientOptions)
  client.setTrace(traceFromConfig())
  context.subscriptions.push(client)

  try {
    await client.start()
  } catch (reason) {
    wrappedOutput.appendLine(`Client error: ${reason}`)
  }
}

function registerCommands(context: ExtensionContext) {
  context.subscriptions.push(
    commands.registerCommand('rocci.restartLspServer', async () => {
      if (client) {
        await client.restart()
      }
    })
  )
  previewSession = new PreviewSession(context)
  registerPreviewCommands(context, previewSession)
}

export async function activate(context: ExtensionContext) {
  createOutputChannels(isDebug)
  wrappedOutput.appendLine(`Activate LSP client in ${context.extensionPath}`)
  registerCommands(context)
  await startClient(context)
}

export function deactivate() {
  const stopPreview = previewSession?.stop()
  previewSession = undefined
  if (!client) {
    return stopPreview
  }
  wrappedOutput.appendLine('Stop client')
  if (!stopPreview) {
    return client.stop()
  }
  return Promise.all([stopPreview, client.stop()]).then(() => undefined)
}

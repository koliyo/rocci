import * as fs from 'fs'
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
import { extractTarGz, installTools, nodeGithubClient } from './tools/install'
import { resolveTool } from './tools/resolve'

let client: LanguageClient | undefined
let previewSession: PreviewSession | undefined
const isDebug = process.env.VSCODE_DEBUG_MODE !== undefined

function resolveServerPath(context: ExtensionContext): string | undefined {
  const configured = workspace.getConfiguration('rocci').get<string>('lsp.serverPath')
  return resolveTool(context, 'rocci-language-server', configured, isDebug)
}

function lspVerbose(): boolean {
  const config = workspace.getConfiguration('rocci')
  return config.get<boolean>('lsp.verbose', false) || config.get<string>('lsp.trace.server', 'off') === 'verbose'
}

function traceFromConfig(): Trace {
  if (lspVerbose()) {
    return Trace.Verbose
  }
  const value = workspace.getConfiguration('rocci').get<string>('lsp.trace.server', 'off')
  switch (value) {
    case 'messages':
      return Trace.Messages
    default:
      return Trace.Off
  }
}

function resolveRocPath(): string | undefined {
  const rocci = workspace.getConfiguration('rocci').get<string>('roc.path')?.trim()
  if (rocci) {
    return rocci
  }
  const fromEnv = process.env.ROCCI_ROC_PATH?.trim()
  if (fromEnv) {
    return fromEnv
  }
  const vscodeRoc = workspace.getConfiguration('roc').get<string>('path')?.trim()
  return vscodeRoc || undefined
}

async function stopClient() {
  if (!client) {
    return
  }
  const current = client
  client = undefined
  await current.stop()
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

  const rocPath = resolveRocPath()
  const env = { ...process.env }
  if (rocPath) {
    env.ROCCI_ROC_PATH = rocPath
  }
  if (lspVerbose()) {
    env.ROCCI_LSP_VERBOSE = '1'
    wrappedOutput.appendLine('Verbose language-server logging enabled (rocci.lsp.verbose)')
    wrappedOutput.show(true)
  }
  if (rocPath) {
    wrappedOutput.appendLine(`Roc compiler: ${rocPath}`)
  } else {
    wrappedOutput.appendLine('Roc compiler: roc on PATH (set rocci.roc.path or roc.path if hover is empty)')
  }

  const executable: Executable = {
    command: serverPath,
    args: [],
    transport: TransportKind.stdio,
    options: { env }
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
    revealOutputChannelOn: lspVerbose() ? RevealOutputChannelOn.Info : RevealOutputChannelOn.Error
  }

  client = new LanguageClient('rocci', 'Rocci', serverOptions, clientOptions)
  client.setTrace(traceFromConfig())

  try {
    await client.start()
  } catch (reason) {
    wrappedOutput.appendLine(`Client error: ${reason}`)
    client = undefined
  }
}

async function restartClient(context: ExtensionContext) {
  await stopClient()
  await startClient(context)
}

function registerCommands(context: ExtensionContext) {
  context.subscriptions.push(
    workspace.onDidChangeConfiguration(async event => {
      if (
        event.affectsConfiguration('rocci.roc.path') ||
        event.affectsConfiguration('roc.path') ||
        event.affectsConfiguration('rocci.lsp.verbose') ||
        event.affectsConfiguration('rocci.lsp.trace.server')
      ) {
        await restartClient(context)
      }
    }),
    commands.registerCommand('rocci.restartLspServer', async () => {
      await restartClient(context)
    }),
    commands.registerCommand('rocci.updateTools', async () => {
      await updateTools(context, true)
      await restartClient(context)
    })
  )
  previewSession = new PreviewSession(context)
  registerPreviewCommands(context, previewSession)
}

async function updateTools(context: ExtensionContext, overwriteDev: boolean): Promise<void> {
  const config = workspace.getConfiguration('rocci')
  const channel = config.get<string>('tools.channel') === 'dev' ? 'dev' : 'stable'
  try {
    fs.mkdirSync(context.globalStorageUri.fsPath, { recursive: true })
    await installTools({
      storageRoot: context.globalStorageUri.fsPath,
      channel,
      overwriteDev,
      platform: process.platform,
      arch: process.arch,
      client: nodeGithubClient('rocci-vscode'),
      extract: extractTarGz,
      log: message => wrappedOutput.appendLine(message)
    })
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    wrappedOutput.appendLine(message)
    await window.showErrorMessage(message)
  }
}

export async function activate(context: ExtensionContext) {
  createOutputChannels(isDebug)
  wrappedOutput.appendLine(`Activate LSP client in ${context.extensionPath}`)
  registerCommands(context)
  const autoUpdate = workspace.getConfiguration('rocci').get<boolean>('tools.autoUpdate', true)
  if (!isDebug && autoUpdate) {
    await updateTools(context, false)
  }
  await startClient(context)
}

export function deactivate() {
  const stopPreview = previewSession?.stop()
  previewSession = undefined
  if (!client) {
    return stopPreview
  }
  wrappedOutput.appendLine('Stop client')
  const stopClientPromise = stopClient()
  if (!stopPreview) {
    return stopClientPromise
  }
  return Promise.all([stopPreview, stopClientPromise]).then(() => undefined)
}

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
    }),
    commands.registerCommand('rocci.updateTools', async () => {
      await updateTools(context, true)
      if (client) {
        await client.restart()
      } else {
        await startClient(context)
      }
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
  if (!stopPreview) {
    return client.stop()
  }
  return Promise.all([stopPreview, client.stop()]).then(() => undefined)
}

import { OutputChannel, window } from 'vscode'

export let wrappedOutput: OutputChannel

export function createOutputChannels(isDebug: boolean) {
  const channel = window.createOutputChannel('Rocci')
  if (!isDebug) {
    wrappedOutput = channel
    return
  }

  const appendLine = channel.appendLine.bind(channel)
  channel.appendLine = (value: string) => {
    appendLine(value)
    console.log(value)
  }
  wrappedOutput = channel
}

import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { runTests } from '@vscode/test-electron'

function findLocalVsCodeExecutable(): string | undefined {
  if (process.env.VSCODE_PATH && fs.existsSync(process.env.VSCODE_PATH)) {
    return process.env.VSCODE_PATH
  }

  if (os.type() === 'Darwin') {
    const defaultMacPath = '/Applications/Visual Studio Code.app/Contents/MacOS/Code'
    if (fs.existsSync(defaultMacPath)) {
      return defaultMacPath
    }
  }

  return undefined
}

async function main() {
  try {
    const extensionDevelopmentPath = path.resolve(__dirname, '../../')
    const extensionTestsPath = path.resolve(__dirname, './suite/index')
    const testWorkspace = path.resolve(__dirname, '../../../../test')

    process.env.VSCODE_DEBUG_MODE = '1'

    const localExecutable = findLocalVsCodeExecutable()
    if (localExecutable) {
      console.log(`Using local VS Code executable: ${localExecutable}`)
    }

    await runTests({
      vscodeExecutablePath: localExecutable,
      extensionDevelopmentPath,
      extensionTestsPath,
      launchArgs: [testWorkspace, '--disable-extensions']
    })
  } catch (err) {
    console.error('Failed to run tests', err)
    process.exit(1)
  }
}

main()

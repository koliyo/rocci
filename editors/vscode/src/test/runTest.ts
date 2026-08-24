import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { runTests } from '@vscode/test-electron'

function findLocalVsCodeExecutable(): string | undefined {
  if (process.env.VSCODE_PATH && fs.existsSync(process.env.VSCODE_PATH)) {
    return process.env.VSCODE_PATH
  }

  return undefined
}

async function main() {
  try {
    // Cursor and other Electron hosts inherit this; leaving it set makes
    // downloaded VS Code run as Node instead of launching the editor.
    delete process.env.ELECTRON_RUN_AS_NODE

    const extensionDevelopmentPath = path.resolve(__dirname, '../../')
    const extensionTestsPath = path.resolve(__dirname, './suite/index')

    process.env.VSCODE_DEBUG_MODE = '1'

    const localExecutable = findLocalVsCodeExecutable()
    if (localExecutable) {
      console.log(`Using local VS Code executable: ${localExecutable}`)
    }

    const userDataDir = path.join(os.tmpdir(), 'rocci-vscode-test')
    await runTests({
      ...(localExecutable ? { vscodeExecutablePath: localExecutable } : {}),
      extensionDevelopmentPath,
      extensionTestsPath,
      launchArgs: ['--disable-extensions', `--user-data-dir=${userDataDir}`]
    })
  } catch (err) {
    console.error('Failed to run tests', err)
    process.exit(1)
  }
}

main()

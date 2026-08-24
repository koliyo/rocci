import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { ExtensionContext, workspace } from 'vscode'

import { PreviewProduct } from './dispatch'

function exeName(base: string): string {
  return os.type() === 'Windows_NT' ? `${base}.exe` : base
}

export function findOnPath(name: string): string | undefined {
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

function debugBinary(context: ExtensionContext, exe: string): string | undefined {
  const fromExtension = path.join(context.extensionPath, '..', '..', 'target', 'debug', exe)
  if (fs.existsSync(fromExtension)) {
    return fromExtension
  }
  for (const folder of workspace.workspaceFolders ?? []) {
    const candidate = path.join(folder.uri.fsPath, 'target', 'debug', exe)
    if (fs.existsSync(candidate)) {
      return candidate
    }
  }
  return undefined
}

export function resolvePreviewBinary(
  context: ExtensionContext,
  product: PreviewProduct
): string | undefined {
  const setting =
    product === 'rocci' ? 'preview.rocciPath' : 'preview.rocdownPath'
  const configured = workspace.getConfiguration('rocci').get<string>(setting)?.trim()
  if (configured) {
    return configured
  }

  const exe = exeName(product === 'rocci' ? 'rocci' : 'rocdown')
  const bundled = path.join(context.extensionPath, 'dist', 'bin', exe)
  if (fs.existsSync(bundled)) {
    return bundled
  }

  return findOnPath(exe) ?? debugBinary(context, exe)
}

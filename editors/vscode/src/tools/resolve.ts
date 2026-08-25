import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { ExtensionContext, workspace } from 'vscode'

import { releaseExtractDir, releaseTag, ReleaseManifest } from './release'

export type ToolName = 'rocci' | 'rocdown' | 'rocci-language-server'

function exeName(base: ToolName): string {
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

export function debugBinary(context: ExtensionContext, exe: string): string | undefined {
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

export function extractedBinary(storageRoot: string, tag: string, tool: ToolName): string | undefined {
  const exe = exeName(tool)
  const dir = releaseExtractDir(storageRoot, tag)
  const nested = path.join(dir, exe)
  if (fs.existsSync(nested)) {
    return nested
  }
  const entries = fs.existsSync(dir) ? fs.readdirSync(dir) : []
  for (const entry of entries) {
    const candidate = path.join(dir, entry, exe)
    if (fs.existsSync(candidate)) {
      return candidate
    }
  }
  return undefined
}

export function latestExtractedBinary(storageRoot: string, tool: ToolName): string | undefined {
  const releases = path.join(storageRoot, 'releases')
  if (!fs.existsSync(releases)) {
    return undefined
  }
  const tags = fs.readdirSync(releases).sort()
  for (let i = tags.length - 1; i >= 0; i -= 1) {
    const found = extractedBinary(storageRoot, tags[i], tool)
    if (found) {
      return found
    }
  }
  return undefined
}

export function readCachedManifest(storageRoot: string): ReleaseManifest | undefined {
  const manifestPath = path.join(storageRoot, 'manifest.json')
  if (!fs.existsSync(manifestPath)) {
    return undefined
  }
  try {
    return JSON.parse(fs.readFileSync(manifestPath, 'utf8')) as ReleaseManifest
  } catch {
    return undefined
  }
}

export function resolveTool(
  context: ExtensionContext,
  tool: ToolName,
  settingValue: string | undefined,
  isDebug: boolean
): string | undefined {
  const configured = settingValue?.trim()
  if (configured) {
    return configured
  }
  const exe = exeName(tool)
  if (isDebug) {
    const debug = debugBinary(context, exe)
    if (debug) {
      return debug
    }
  }
  const storageRoot = context.globalStorageUri.fsPath
  const cached = readCachedManifest(storageRoot)
  if (cached) {
    const fromRelease = extractedBinary(storageRoot, releaseTag(cached), tool)
    if (fromRelease) {
      return fromRelease
    }
  }
  return latestExtractedBinary(storageRoot, tool) ?? findOnPath(exe)
}

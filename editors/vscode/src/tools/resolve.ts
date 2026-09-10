import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { ExtensionContext, workspace } from 'vscode'

import {
  localCargoBinary,
  pickResolvedTool,
  ToolsChannel,
  workspaceCargoRoots
} from './local-build'
import { parseReleaseManifest, releaseExtractDir, releaseTag, ReleaseManifest } from './release'

export type { ToolsChannel }
export { newestLocalCargoBuild, workspaceCargoRoots } from './local-build'

export type ToolName = 'rocci' | 'rocdown' | 'rocci-language-server'

export type ResolveToolOptions = {
  isDebug: boolean
  channel: ToolsChannel
}

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
  return localCargoBinary(workspaceCargoRoots(context.extensionPath, workspace.workspaceFolders), exe)?.path
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
    return parseReleaseManifest(JSON.parse(fs.readFileSync(manifestPath, 'utf8')))
  } catch {
    return undefined
  }
}

export function resolveTool(
  context: ExtensionContext,
  tool: ToolName,
  settingValue: string | undefined,
  options: ResolveToolOptions
): string | undefined {
  const exe = exeName(tool)
  const local = localCargoBinary(
    workspaceCargoRoots(context.extensionPath, workspace.workspaceFolders),
    exe
  )
  const storageRoot = context.globalStorageUri.fsPath
  const cached = readCachedManifest(storageRoot)
  const fromRelease = cached ? extractedBinary(storageRoot, releaseTag(cached), tool) : undefined
  return pickResolvedTool({
    configured: settingValue,
    isDebug: options.isDebug,
    channel: options.channel,
    local,
    release: fromRelease && cached ? { path: fromRelease, publishedAt: cached.publishedAt } : undefined,
    fallback: latestExtractedBinary(storageRoot, tool) ?? findOnPath(exe)
  })
}

import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'

import { isNewerTimestamp } from './release'

export type ToolsChannel = 'stable' | 'dev'

export type LocalCargoBinary = {
  path: string
  mtime: Date
}

const CARGO_PROFILES = ['debug', 'release'] as const
export const LOCAL_TOOL_BINARIES = ['rocci', 'rocdown', 'rocci-language-server', 'rocci-okf'] as const

function exeName(base: string): string {
  return os.type() === 'Windows_NT' ? `${base}.exe` : base
}

export function workspaceCargoRoots(
  extensionPath: string,
  workspaceFolders: readonly { uri: { fsPath: string } }[] | undefined
): string[] {
  const roots = [path.join(extensionPath, '..', '..')]
  for (const folder of workspaceFolders ?? []) {
    roots.push(folder.uri.fsPath)
  }
  return roots
}

export function localCargoBinary(roots: readonly string[], exe: string): LocalCargoBinary | undefined {
  let best: LocalCargoBinary | undefined
  for (const root of roots) {
    for (const profile of CARGO_PROFILES) {
      const candidate = path.join(root, 'target', profile, exe)
      if (!fs.existsSync(candidate)) {
        continue
      }
      const mtime = fs.statSync(candidate).mtime
      if (!best || mtime.getTime() > best.mtime.getTime()) {
        best = { path: candidate, mtime }
      }
    }
  }
  return best
}

export function newestLocalCargoBuild(
  roots: readonly string[],
  names: readonly string[] = LOCAL_TOOL_BINARIES
): LocalCargoBinary | undefined {
  let best: LocalCargoBinary | undefined
  for (const name of names) {
    const found = localCargoBinary(roots, exeName(name))
    if (found && (!best || found.mtime.getTime() > best.mtime.getTime())) {
      best = found
    }
  }
  return best
}

export function pickResolvedTool(options: {
  configured?: string
  isDebug: boolean
  channel: ToolsChannel
  local?: LocalCargoBinary
  release?: { path: string; publishedAt: string }
  fallback?: string
}): string | undefined {
  const configured = options.configured?.trim()
  if (configured) {
    return configured
  }
  if (options.isDebug && options.local) {
    return options.local.path
  }
  if (options.channel === 'dev' && options.local) {
    if (!options.release || isNewerTimestamp(options.local.mtime, options.release.publishedAt)) {
      return options.local.path
    }
    return options.release.path
  }
  return options.release?.path ?? options.fallback
}

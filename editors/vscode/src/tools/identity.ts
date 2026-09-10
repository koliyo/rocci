import * as path from 'path'

export type ExtensionMode = 'F5' | 'installed'

export function formatExtensionIdentity(options: {
  publisher: string
  name: string
  version: string
  git?: string
  mode: ExtensionMode
}): string {
  const id = `${options.publisher}.${options.name}`
  const git = options.git?.trim() ? options.git.trim() : 'unknown'
  return `Extension: ${id} ${options.version} git ${git} (${options.mode})`
}

export function cargoProfileFromPath(binaryPath: string): 'debug' | 'release' | undefined {
  const debug = `${path.sep}target${path.sep}debug${path.sep}`
  const release = `${path.sep}target${path.sep}release${path.sep}`
  if (binaryPath.includes(debug)) {
    return 'debug'
  }
  if (binaryPath.includes(release)) {
    return 'release'
  }
  return undefined
}

export function formatLanguageServerIdentity(options: {
  binaryPath: string
  cargoMtime?: string
  cargoProfile?: 'debug' | 'release'
  github?: { tagName: string; name: string; id: number; publishedAt: string }
}): string {
  if (options.cargoProfile && options.cargoMtime) {
    return `Language server: local Cargo ${options.cargoProfile} ${options.cargoMtime}`
  }
  if (options.github) {
    return `Language server: GitHub ${options.github.tagName} (${options.github.name}, id ${options.github.id}, published ${options.github.publishedAt})`
  }
  return `Language server: ${options.binaryPath}`
}

export function parseBuildInfo(data: unknown): { git?: string; builtAt?: string } | undefined {
  if (!data || typeof data !== 'object') {
    return undefined
  }
  const value = data as Record<string, unknown>
  const git = typeof value.git === 'string' && value.git ? value.git : undefined
  const builtAt = typeof value.builtAt === 'string' && value.builtAt ? value.builtAt : undefined
  if (!git && !builtAt) {
    return undefined
  }
  return { git, builtAt }
}

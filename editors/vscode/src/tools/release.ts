import { createHash } from 'crypto'
import * as path from 'path'

export const SUPPORTED_TRIPLES = ['aarch64-apple-darwin', 'x86_64-unknown-linux-gnu'] as const

export const UNKNOWN_TARGET_MESSAGE =
  `Unsupported platform. Rocci GitHub releases currently publish ${SUPPORTED_TRIPLES.join(' and ')}.`

export type ReleaseManifest = {
  id: number
  name: string
  tagName: string
  publishedAt: string
}

export type ReleaseAsset = {
  name: string
  downloadUrl: string
}

export function rustTriple(platform: string, arch: string): string {
  if (platform === 'darwin' && arch === 'arm64') {
    return 'aarch64-apple-darwin'
  }
  if (platform === 'linux' && arch === 'x64') {
    return 'x86_64-unknown-linux-gnu'
  }
  throw new Error(UNKNOWN_TARGET_MESSAGE)
}

export function releaseAssetName(version: string, triple: string): string {
  return `rocci-${version}-${triple}.tar.gz`
}

export function releaseChecksumName(version: string, triple: string): string {
  return `${releaseAssetName(version, triple)}.sha256`
}

export function manifestsEqual(left: ReleaseManifest | undefined, right: ReleaseManifest): boolean {
  return Boolean(
    left &&
      left.id === right.id &&
      left.name === right.name &&
      new Date(left.publishedAt).getTime() === new Date(right.publishedAt).getTime()
  )
}

function asRecord(data: unknown): Record<string, unknown> {
  if (!data || typeof data !== 'object') {
    throw new Error('Invalid GitHub release JSON')
  }
  return data as Record<string, unknown>
}

function readString(data: Record<string, unknown>, keys: string[]): string | undefined {
  for (const key of keys) {
    const value = data[key]
    if (typeof value === 'string' && value) {
      return value
    }
  }
  return undefined
}

export function parseReleaseManifest(data: unknown): ReleaseManifest {
  const value = asRecord(data)
  if (typeof value.id !== 'number') {
    throw new Error('GitHub release JSON is missing id')
  }
  const tagName = readString(value, ['tagName', 'tag_name', 'name'])
  const name = readString(value, ['name']) ?? tagName
  const publishedAt = readString(value, ['publishedAt', 'published_at'])
  if (!name || !tagName) {
    throw new Error('GitHub release JSON is missing name')
  }
  if (!publishedAt) {
    throw new Error('GitHub release JSON is missing publishedAt')
  }
  return { id: value.id, name, tagName, publishedAt }
}

export function parseReleaseAssets(data: unknown): ReleaseAsset[] {
  const value = asRecord(data)
  if (!Array.isArray(value.assets)) {
    return []
  }
  const assets: ReleaseAsset[] = []
  for (const item of value.assets) {
    if (!item || typeof item !== 'object') {
      continue
    }
    const asset = item as Record<string, unknown>
    const name = typeof asset.name === 'string' ? asset.name : undefined
    const downloadUrl = readString(asset, ['downloadUrl', 'browser_download_url'])
    if (name && downloadUrl) {
      assets.push({ name, downloadUrl })
    }
  }
  return assets
}

export function findReleaseArchive(
  assets: { name: string }[],
  triple: string
): { archive: string; checksum: string } | undefined {
  const suffix = `-${triple}.tar.gz`
  const archive = assets.find(
    item => item.name.startsWith('rocci-') && item.name.endsWith(suffix)
  )?.name
  if (!archive) {
    return undefined
  }
  const checksum = `${archive}.sha256`
  if (!assets.some(item => item.name === checksum)) {
    return undefined
  }
  return { archive, checksum }
}

export function sha256Hex(buffer: Buffer): string {
  return createHash('sha256').update(buffer).digest('hex')
}

export function parseSha256Line(text: string): string {
  const hex = text.trim().split(/\s+/)[0]
  if (!/^[0-9a-f]{64}$/i.test(hex)) {
    throw new Error('Checksum file is not a sha256 hex digest')
  }
  return hex.toLowerCase()
}

export function verifySha256(buffer: Buffer, expectedHex: string): void {
  const actual = sha256Hex(buffer)
  if (actual !== expectedHex.toLowerCase()) {
    throw new Error(`sha256 mismatch: expected ${expectedHex}, got ${actual}`)
  }
}

export function githubReleaseApiUrl(channel: 'stable' | 'dev'): string {
  return channel === 'dev'
    ? 'https://api.github.com/repos/koliyo/rocci/releases/tags/dev'
    : 'https://api.github.com/repos/koliyo/rocci/releases/latest'
}

export function githubRequestHeaders(
  userAgent: string,
  kind: 'json' | 'asset'
): Record<string, string> {
  const headers: Record<string, string> = {}
  headers.accept =
    kind === 'json' ? 'application/vnd.github+json' : 'application/octet-stream'
  headers['user-agent'] = userAgent
  if (kind === 'json') {
    headers['x-github-api-version'] = '2022-11-28'
  }
  return headers
}

export function releaseTag(manifest: ReleaseManifest): string {
  return manifest.tagName === 'dev' ? 'dev' : manifest.tagName
}

export function isDevRelease(manifest: ReleaseManifest): boolean {
  return manifest.tagName === 'dev'
}

export function releaseExtractDir(storageRoot: string, tag: string): string {
  return path.join(storageRoot, 'releases', tag)
}

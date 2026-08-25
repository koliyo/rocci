import { createHash } from 'crypto'
import * as path from 'path'

export const SUPPORTED_TRIPLES = ['aarch64-apple-darwin', 'x86_64-unknown-linux-gnu'] as const

export const UNKNOWN_TARGET_MESSAGE =
  `Unsupported platform. Rocci GitHub releases currently publish ${SUPPORTED_TRIPLES.join(' and ')}.`

export type ReleaseManifest = {
  id: number
  name: string
  tag_name: string
  published_at: string
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
      new Date(left.published_at).getTime() === new Date(right.published_at).getTime()
  )
}

export function parseReleaseManifest(data: unknown): ReleaseManifest {
  if (!data || typeof data !== 'object') {
    throw new Error('Invalid GitHub release JSON')
  }
  const value = data as { id?: unknown; name?: unknown; published_at?: unknown; tag_name?: unknown }
  if (typeof value.id !== 'number') {
    throw new Error('GitHub release JSON is missing id')
  }
  const tag_name =
    typeof value.tag_name === 'string' && value.tag_name
      ? value.tag_name
      : typeof value.name === 'string'
        ? value.name
        : ''
  const name = typeof value.name === 'string' && value.name ? value.name : tag_name
  if (!name || !tag_name) {
    throw new Error('GitHub release JSON is missing name')
  }
  if (typeof value.published_at !== 'string') {
    throw new Error('GitHub release JSON is missing published_at')
  }
  return { id: value.id, name, tag_name, published_at: value.published_at }
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

export function releaseTag(manifest: ReleaseManifest): string {
  return manifest.tag_name === 'dev' ? 'dev' : manifest.tag_name
}

export function isDevRelease(manifest: ReleaseManifest): boolean {
  return manifest.tag_name === 'dev'
}

export function releaseExtractDir(storageRoot: string, tag: string): string {
  return path.join(storageRoot, 'releases', tag)
}

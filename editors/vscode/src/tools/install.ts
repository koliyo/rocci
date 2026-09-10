import * as fs from 'fs'
import * as https from 'https'
import * as path from 'path'
import { spawn } from 'child_process'
import { URL } from 'url'

import {
  findReleaseArchive,
  githubReleaseApiUrl,
  githubRequestHeaders,
  isDevRelease,
  isNewerRelease,
  isNewerTimestamp,
  manifestsEqual,
  parseReleaseAssets,
  parseReleaseManifest,
  parseSha256Line,
  releaseExtractDir,
  ReleaseAsset,
  ReleaseManifest,
  rustTriple,
  verifySha256
} from './release'

export type GithubClient = {
  getJson(url: string): Promise<unknown>
  getBuffer(url: string): Promise<Buffer>
}

export type InstallLog = (message: string) => void

const TOOL_BINARIES = ['rocci', 'rocdown', 'rocci-language-server', 'rocci-okf'] as const

function describeManifest(manifest: ReleaseManifest): string {
  return `${manifest.tagName} (${manifest.name}, id ${manifest.id}, published ${manifest.publishedAt})`
}

type RemoteRelease = {
  manifest: ReleaseManifest
  assets: ReleaseAsset[]
}

async function fetchRemote(
  client: GithubClient,
  channel: 'stable' | 'dev',
  log: InstallLog
): Promise<RemoteRelease | undefined> {
  const api = githubReleaseApiUrl(channel)
  log(`Check remote release: ${api}`)
  try {
    const payload = await client.getJson(api)
    const manifest = parseReleaseManifest(payload)
    const assets = parseReleaseAssets(payload)
    log(`Remote found: ${describeManifest(manifest)}`)
    return { manifest, assets }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    log(`Remote ${channel} unavailable: ${message}`)
    return undefined
  }
}

function selectRemote(
  channel: 'stable' | 'dev',
  versioned: RemoteRelease | undefined,
  rolling: RemoteRelease | undefined,
  log: InstallLog
): RemoteRelease {
  const preferred = channel === 'dev' ? rolling : versioned
  if (versioned && isNewerRelease(versioned.manifest, rolling?.manifest)) {
    if (!preferred || isNewerRelease(versioned.manifest, preferred.manifest)) {
      log(
        `GitHub candidate: versioned ${versioned.manifest.tagName} (newer than ${
          rolling ? `rolling dev ${rolling.manifest.publishedAt}` : 'missing rolling dev'
        })`
      )
      return versioned
    }
  }
  const selected = preferred ?? versioned ?? rolling
  if (!selected) {
    throw new Error('Could not read GitHub releases for rocci tools')
  }
  return selected
}

export async function installTools(options: {
  storageRoot: string
  channel: 'stable' | 'dev'
  overwriteDev: boolean
  localBuiltAt?: Date
  platform: string
  arch: string
  client: GithubClient
  extract: (archive: Buffer, dest: string) => Promise<void>
  log: InstallLog
}): Promise<ReleaseManifest | undefined> {
  options.log(
    `Update tools: channel=${options.channel} overwriteDev=${options.overwriteDev} platform=${options.platform} arch=${options.arch}`
  )
  const versioned = await fetchRemote(options.client, 'stable', options.log)
  const rolling = await fetchRemote(options.client, 'dev', options.log)
  const { manifest: latest, assets } = selectRemote(
    options.channel,
    versioned,
    rolling,
    options.log
  )
  const manifestPath = path.join(options.storageRoot, 'manifest.json')
  const local = fs.existsSync(manifestPath)
    ? parseReleaseManifest(JSON.parse(fs.readFileSync(manifestPath, 'utf8')))
    : undefined

  options.log(local ? `Current installed: ${describeManifest(local)}` : 'Current installed: none')

  if (
    options.channel === 'dev' &&
    options.localBuiltAt &&
    isNewerTimestamp(options.localBuiltAt, latest.publishedAt)
  ) {
    options.log(
      `Skip install: local Cargo build newer than ${latest.tagName} (${latest.publishedAt})`
    )
    return local
  }

  if (
    options.channel !== 'dev' &&
    !options.overwriteDev &&
    local &&
    isDevRelease(local) &&
    !isNewerRelease(latest, local)
  ) {
    options.log(
      `Skip install: keeping local ${local.tagName} on channel=${options.channel} (overwriteDev=false)`
    )
    return local
  }
  if (manifestsEqual(local, latest)) {
    options.log(`Skip install: already at ${latest.tagName}`)
    return latest
  }

  if (local) {
    options.log(`Install: ${local.tagName} -> ${latest.tagName}`)
  } else {
    options.log(`Install: none -> ${latest.tagName}`)
  }

  const triple = rustTriple(options.platform, options.arch)
  const names = findReleaseArchive(assets, triple)
  if (!names) {
    throw new Error(
      `Could not find a rocci-*-${triple}.tar.gz asset on ${latest.tagName}`
    )
  }
  const asset = assets.find(item => item.name === names.archive)
  const checksum = assets.find(item => item.name === names.checksum)
  if (!asset || !checksum) {
    throw new Error(`Could not find release assets ${names.archive} and ${names.checksum}`)
  }

  options.log(`Download ${names.archive} (${triple})`)
  const archive = await options.client.getBuffer(asset.downloadUrl)
  options.log(`Downloaded ${names.archive} (${archive.length} bytes)`)
  const digestText = (await options.client.getBuffer(checksum.downloadUrl)).toString('utf8')
  const expectedSha = parseSha256Line(digestText)
  verifySha256(archive, expectedSha)
  options.log(`Checksum ok: ${expectedSha}`)

  const dest = releaseExtractDir(options.storageRoot, latest.tagName)
  fs.rmSync(dest, { recursive: true, force: true })
  fs.mkdirSync(dest, { recursive: true })
  await options.extract(archive, dest)
  const installed: string[] = []
  for (const name of TOOL_BINARIES) {
    const found = findExtracted(dest, name)
    if (found) {
      fs.chmodSync(found, 0o755)
      installed.push(name)
    }
  }
  fs.mkdirSync(options.storageRoot, { recursive: true })
  fs.writeFileSync(manifestPath, JSON.stringify(latest, null, 2))
  options.log(`Installed ${describeManifest(latest)} into ${dest}`)
  options.log(
    installed.length > 0
      ? `Installed binaries: ${installed.join(', ')}`
      : `Installed binaries: none of ${TOOL_BINARIES.join(', ')} found in archive`
  )
  return latest
}

function findExtracted(dest: string, name: string): string | undefined {
  const direct = path.join(dest, name)
  if (fs.existsSync(direct)) {
    return direct
  }
  for (const entry of fs.readdirSync(dest)) {
    const nested = path.join(dest, entry, name)
    if (fs.existsSync(nested)) {
      return nested
    }
  }
  return undefined
}

export function nodeGithubClient(userAgent: string): GithubClient {
  return {
    getJson: async url => JSON.parse((await httpsGet(url, userAgent, 'json')).toString('utf8')),
    getBuffer: url => httpsGet(url, userAgent, 'asset')
  }
}

export function extractTarGz(archive: Buffer, dest: string): Promise<void> {
  const archivePath = path.join(dest, 'release.tar.gz')
  fs.writeFileSync(archivePath, archive)
  return new Promise((resolve, reject) => {
    const child = spawn('tar', ['-xzf', archivePath, '-C', dest], { stdio: 'ignore' })
    child.on('error', reject)
    child.on('exit', code => {
      fs.rmSync(archivePath, { force: true })
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`tar exited with ${code}`))
      }
    })
  })
}

function httpsGet(
  url: string,
  userAgent: string,
  kind: 'json' | 'asset',
  redirects = 0
): Promise<Buffer> {
  if (redirects > 5) {
    return Promise.reject(new Error('Too many redirects'))
  }
  return new Promise((resolve, reject) => {
    const parsed = new URL(url)
    const req = https.get(
      {
        hostname: parsed.hostname,
        path: `${parsed.pathname}${parsed.search}`,
        headers: githubRequestHeaders(userAgent, kind)
      },
      res => {
        if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume()
          resolve(httpsGet(res.headers.location, userAgent, kind, redirects + 1))
          return
        }
        if (res.statusCode !== 200) {
          res.resume()
          reject(new Error(`HTTP ${res.statusCode} for ${url}`))
          return
        }
        const chunks: Buffer[] = []
        res.on('data', chunk => chunks.push(chunk as Buffer))
        res.on('end', () => resolve(Buffer.concat(chunks)))
      }
    )
    req.on('error', reject)
  })
}

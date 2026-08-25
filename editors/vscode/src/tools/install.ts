import * as fs from 'fs'
import * as https from 'https'
import * as path from 'path'
import { spawn } from 'child_process'
import { URL } from 'url'

import {
  findReleaseArchive,
  githubReleaseApiUrl,
  isDevRelease,
  manifestsEqual,
  parseReleaseManifest,
  parseSha256Line,
  releaseExtractDir,
  ReleaseManifest,
  rustTriple,
  verifySha256
} from './release'

export type GithubClient = {
  getJson(url: string): Promise<unknown>
  getBuffer(url: string): Promise<Buffer>
}

export type InstallLog = (message: string) => void

export type ReleasePayload = {
  id: number
  name?: string
  tag_name?: string
  published_at: string
  assets: { name: string; browser_download_url: string }[]
}

export async function installTools(options: {
  storageRoot: string
  channel: 'stable' | 'dev'
  overwriteDev: boolean
  platform: string
  arch: string
  client: GithubClient
  extract: (archive: Buffer, dest: string) => Promise<void>
  log: InstallLog
}): Promise<ReleaseManifest | undefined> {
  const api = githubReleaseApiUrl(options.channel)
  options.log(`Check for tools: ${api}`)
  const payload = (await options.client.getJson(api)) as ReleasePayload
  const latest = parseReleaseManifest(payload)
  const manifestPath = path.join(options.storageRoot, 'manifest.json')
  const local = fs.existsSync(manifestPath)
    ? parseReleaseManifest(JSON.parse(fs.readFileSync(manifestPath, 'utf8')))
    : undefined

  if (options.channel !== 'dev' && !options.overwriteDev && local && isDevRelease(local)) {
    options.log(`Dev version detected: ${local.tag_name}`)
    return local
  }
  if (manifestsEqual(local, latest)) {
    options.log(`Installed tools are up to date: ${latest.tag_name}`)
    return latest
  }

  const triple = rustTriple(options.platform, options.arch)
  const names = findReleaseArchive(payload.assets, triple)
  if (!names) {
    throw new Error(
      `Could not find a rocci-*-${triple}.tar.gz asset on ${latest.tag_name}`
    )
  }
  const asset = payload.assets.find(item => item.name === names.archive)
  const checksum = payload.assets.find(item => item.name === names.checksum)
  if (!asset || !checksum) {
    throw new Error(`Could not find release assets ${names.archive} and ${names.checksum}`)
  }

  const archive = await options.client.getBuffer(asset.browser_download_url)
  const digestText = (await options.client.getBuffer(checksum.browser_download_url)).toString('utf8')
  verifySha256(archive, parseSha256Line(digestText))

  const dest = releaseExtractDir(options.storageRoot, latest.tag_name)
  fs.rmSync(dest, { recursive: true, force: true })
  fs.mkdirSync(dest, { recursive: true })
  await options.extract(archive, dest)
  for (const name of ['rocci', 'rocdown', 'rocci-language-server', 'rocci-okf']) {
    const found = findExtracted(dest, name)
    if (found) {
      fs.chmodSync(found, 0o755)
    }
  }
  fs.mkdirSync(options.storageRoot, { recursive: true })
  fs.writeFileSync(manifestPath, JSON.stringify(latest, null, 2))
  options.log(`Installed ${latest.name} into ${dest}`)
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
    getJson: async url => JSON.parse((await httpsGet(url, userAgent)).toString('utf8')),
    getBuffer: url => httpsGet(url, userAgent)
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

function httpsGet(url: string, userAgent: string, redirects = 0): Promise<Buffer> {
  if (redirects > 5) {
    return Promise.reject(new Error('Too many redirects'))
  }
  return new Promise((resolve, reject) => {
    const parsed = new URL(url)
    const req = https.get(
      {
        hostname: parsed.hostname,
        path: `${parsed.pathname}${parsed.search}`,
        headers: { 'User-Agent': userAgent, Accept: 'application/octet-stream' }
      },
      res => {
        if (res.statusCode && res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume()
          resolve(httpsGet(res.headers.location, userAgent, redirects + 1))
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

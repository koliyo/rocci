import * as assert from 'assert'
import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'

import { installTools } from '../../tools/install'

import {
  findReleaseArchive,
  githubReleaseApiUrl,
  githubRequestHeaders,
  manifestsEqual,
  parseReleaseManifest,
  parseSha256Line,
  releaseAssetName,
  rustTriple,
  sha256Hex,
  UNKNOWN_TARGET_MESSAGE,
  verifySha256
} from '../../tools/release'

suite('Rocci tools release contract (offline)', () => {
  test('maps supported triples and refuses unknown targets', () => {
    assert.strictEqual(rustTriple('darwin', 'arm64'), 'aarch64-apple-darwin')
    assert.strictEqual(rustTriple('linux', 'x64'), 'x86_64-unknown-linux-gnu')
    assert.strictEqual(
      releaseAssetName('0.1.0', 'aarch64-apple-darwin'),
      'rocci-0.1.0-aarch64-apple-darwin.tar.gz'
    )
    assert.throws(() => rustTriple('darwin', 'x64'), /aarch64-apple-darwin/)
    assert.throws(() => rustTriple('win32', 'x64'), new RegExp(UNKNOWN_TARGET_MESSAGE.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')))
  })

  test('compares Hylo-style id/name/date manifests', () => {
    const latest = parseReleaseManifest({
      id: 42,
      name: 'v0.1.0',
      publishedAt: '2026-08-25T00:00:00Z'
    })
    assert.ok(
      manifestsEqual(latest, {
        id: 42,
        name: 'v0.1.0',
        tagName: 'v0.1.0',
        publishedAt: '2026-08-25T00:00:00.000Z'
      })
    )
    const fromGithub = parseReleaseManifest(
      JSON.parse('{"id":42,"name":"v0.1.0","tag_name":"v0.1.0","published_at":"2026-08-25T00:00:00Z"}')
    )
    assert.strictEqual(fromGithub.tagName, 'v0.1.0')
    assert.ok(!manifestsEqual({ ...latest, id: 7 }, latest))
    assert.strictEqual(githubReleaseApiUrl('stable'), 'https://api.github.com/repos/koliyo/rocci/releases/latest')
    assert.strictEqual(githubReleaseApiUrl('dev'), 'https://api.github.com/repos/koliyo/rocci/releases/tags/dev')
    const jsonHeaders = githubRequestHeaders('rocci-vscode', 'json')
    assert.strictEqual(jsonHeaders.accept, 'application/vnd.github+json')
    assert.notStrictEqual(jsonHeaders.accept, 'application/octet-stream')
    assert.strictEqual(githubRequestHeaders('rocci-vscode', 'asset').accept, 'application/octet-stream')
  })

  test('verifies sha256 against a fixture buffer and rejects mismatch', () => {
    const buffer = Buffer.from('rocci-release-fixture')
    const digest = sha256Hex(buffer)
    assert.strictEqual(parseSha256Line(`${digest}  rocci-0.1.0.tar.gz\n`), digest)
    verifySha256(buffer, digest)
    assert.throws(() => verifySha256(buffer, '0'.repeat(64)), /sha256 mismatch/)
  })

  test('installs a mocked GitHub release after sha256 verify', async () => {
    const archive = Buffer.from('fixture-archive-bytes')
    const digest = sha256Hex(archive)
    const storage = fs.mkdtempSync(path.join(os.tmpdir(), 'rocci-tools-'))
    const logs: string[] = []
    await installTools({
      storageRoot: storage,
      channel: 'stable',
      overwriteDev: false,
      platform: 'darwin',
      arch: 'arm64',
      client: {
        getJson: async () => ({
          id: 9,
          name: 'v0.1.0',
          tagName: 'v0.1.0',
          publishedAt: '2026-08-25T00:00:00Z',
          assets: [
            {
              name: 'rocci-v0.1.0-aarch64-apple-darwin.tar.gz',
              downloadUrl: 'https://example.test/archive'
            },
            {
              name: 'rocci-v0.1.0-aarch64-apple-darwin.tar.gz.sha256',
              downloadUrl: 'https://example.test/sha'
            }
          ]
        }),
        getBuffer: async url =>
          url.endsWith('/sha') ? Buffer.from(`${digest}  rocci-v0.1.0-aarch64-apple-darwin.tar.gz\n`) : archive
      },
      extract: async (buffer, dest) => {
        fs.writeFileSync(path.join(dest, 'rocci-language-server'), buffer)
      },
      log: message => logs.push(message)
    })
    assert.ok(fs.existsSync(path.join(storage, 'releases', 'v0.1.0', 'rocci-language-server')))
    assert.ok(fs.existsSync(path.join(storage, 'manifest.json')))
    assert.ok(logs.some(line => line.includes('Current installed: none')))
    assert.ok(logs.some(line => line.includes('Remote found: v0.1.0')))
    assert.ok(logs.some(line => line.includes('Install: none -> v0.1.0')))
    assert.ok(logs.some(line => line.includes('Installed binaries: rocci-language-server')))
  })

  test('installs the rolling GitHub tag dev archive (dev-<sha> assets)', async () => {
    const archive = Buffer.from('dev-archive-bytes')
    const digest = sha256Hex(archive)
    const storage = fs.mkdtempSync(path.join(os.tmpdir(), 'rocci-tools-dev-'))
    const names = findReleaseArchive(
      [
        { name: 'rocci-dev-abcdef0-aarch64-apple-darwin.tar.gz' },
        { name: 'rocci-dev-abcdef0-aarch64-apple-darwin.tar.gz.sha256' }
      ],
      'aarch64-apple-darwin'
    )
    assert.deepStrictEqual(names, {
      archive: 'rocci-dev-abcdef0-aarch64-apple-darwin.tar.gz',
      checksum: 'rocci-dev-abcdef0-aarch64-apple-darwin.tar.gz.sha256'
    })
    await installTools({
      storageRoot: storage,
      channel: 'dev',
      overwriteDev: false,
      platform: 'darwin',
      arch: 'arm64',
      client: {
        getJson: async url => {
          assert.strictEqual(url, githubReleaseApiUrl('dev'))
          return JSON.parse(
            '{"id":11,"name":"Development Build (abcdef0)","tag_name":"dev","published_at":"2026-08-25T12:00:00Z","assets":[{"name":"rocci-dev-abcdef0-aarch64-apple-darwin.tar.gz","browser_download_url":"https://example.test/archive"},{"name":"rocci-dev-abcdef0-aarch64-apple-darwin.tar.gz.sha256","browser_download_url":"https://example.test/sha"}]}'
          )
        },
        getBuffer: async url =>
          url.endsWith('/sha')
            ? Buffer.from(`${digest}  rocci-dev-abcdef0-aarch64-apple-darwin.tar.gz\n`)
            : archive
      },
      extract: async (buffer, dest) => {
        fs.writeFileSync(path.join(dest, 'rocci-language-server'), buffer)
      },
      log: () => undefined
    })
    assert.ok(fs.existsSync(path.join(storage, 'releases', 'dev', 'rocci-language-server')))
    const manifest = parseReleaseManifest(
      JSON.parse(fs.readFileSync(path.join(storage, 'manifest.json'), 'utf8'))
    )
    assert.strictEqual(manifest.tagName, 'dev')
  })

  test('logs current and remote versions when already up to date', async () => {
    const archive = Buffer.from('fixture-archive-bytes')
    const digest = sha256Hex(archive)
    const storage = fs.mkdtempSync(path.join(os.tmpdir(), 'rocci-tools-current-'))
    const payload = {
      id: 9,
      name: 'v0.1.0',
      tagName: 'v0.1.0',
      publishedAt: '2026-08-25T00:00:00Z',
      assets: [
        {
          name: 'rocci-v0.1.0-aarch64-apple-darwin.tar.gz',
          downloadUrl: 'https://example.test/archive'
        },
        {
          name: 'rocci-v0.1.0-aarch64-apple-darwin.tar.gz.sha256',
          downloadUrl: 'https://example.test/sha'
        }
      ]
    }
    const client = {
      getJson: async () => payload,
      getBuffer: async (url: string) =>
        url.endsWith('/sha') ? Buffer.from(`${digest}  rocci-v0.1.0-aarch64-apple-darwin.tar.gz\n`) : archive
    }
    const extract = async (buffer: Buffer, dest: string) => {
      fs.writeFileSync(path.join(dest, 'rocci-language-server'), buffer)
    }
    await installTools({
      storageRoot: storage,
      channel: 'stable',
      overwriteDev: false,
      platform: 'darwin',
      arch: 'arm64',
      client,
      extract,
      log: () => undefined
    })
    const logs: string[] = []
    await installTools({
      storageRoot: storage,
      channel: 'stable',
      overwriteDev: false,
      platform: 'darwin',
      arch: 'arm64',
      client,
      extract,
      log: message => logs.push(message)
    })
    assert.ok(logs.some(line => line.includes('Current installed: v0.1.0')))
    assert.ok(logs.some(line => line.includes('Remote found: v0.1.0')))
    assert.ok(logs.some(line => line.includes('Skip install: already at v0.1.0')))
  })
})

import * as assert from 'assert'

import {
  githubReleaseApiUrl,
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
      published_at: '2026-08-25T00:00:00Z'
    })
    assert.ok(
      manifestsEqual(latest, {
        id: 42,
        name: 'v0.1.0',
        published_at: '2026-08-25T00:00:00.000Z'
      })
    )
    assert.ok(!manifestsEqual({ ...latest, id: 7 }, latest))
    assert.strictEqual(githubReleaseApiUrl('stable'), 'https://api.github.com/repos/koliyo/rocci/releases/latest')
    assert.strictEqual(githubReleaseApiUrl('dev'), 'https://api.github.com/repos/koliyo/rocci/releases/tags/dev')
  })

  test('verifies sha256 against a fixture buffer and rejects mismatch', () => {
    const buffer = Buffer.from('rocci-release-fixture')
    const digest = sha256Hex(buffer)
    assert.strictEqual(parseSha256Line(`${digest}  rocci-0.1.0.tar.gz\n`), digest)
    verifySha256(buffer, digest)
    assert.throws(() => verifySha256(buffer, '0'.repeat(64)), /sha256 mismatch/)
  })
})

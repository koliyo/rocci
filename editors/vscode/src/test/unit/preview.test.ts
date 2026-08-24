import * as assert from 'assert'
import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'

import { canPreviewDocument, chooseBrowserHost } from '../../preview/browser'
import { previewArgv } from '../../preview/dispatch'
import { previewOrigin, reuseDecision } from '../../preview/origin'
import { parsePreviewUrl } from '../../preview/parse'

const PREVIEW_WHEN = 'editorLangId == rocci || editorLangId == rocdown'

suite('Rocci preview (offline)', () => {
  test('parses rocci Serving loopback URLs', () => {
    assert.strictEqual(
      parsePreviewUrl('Serving Counter at http://127.0.0.1:8000/'),
      'http://127.0.0.1:8000/'
    )
  })

  test('parses rocdown serving loopback URLs with a path', () => {
    assert.strictEqual(
      parsePreviewUrl('rocdown: serving Guide at http://127.0.0.1:8000/guide/'),
      'http://127.0.0.1:8000/guide/'
    )
  })

  test('prefers preview_ready over Serving lines', () => {
    const text = [
      'Serving Counter at http://127.0.0.1:8000/',
      'preview_ready http://127.0.0.1:9000/guide/'
    ].join('\n')
    assert.strictEqual(parsePreviewUrl(text), 'http://127.0.0.1:9000/guide/')
  })

  test('dispatches .rocci to rocci run --no-window --port auto', () => {
    const argv = previewArgv('/tmp/App.rocci')
    assert.deepStrictEqual(argv, {
      product: 'rocci',
      args: ['run', '/tmp/App.rocci', '--no-window', '--port', 'auto']
    })
  })

  test('dispatches .rocdown to rocdown view --no-window --port auto', () => {
    const argv = previewArgv('/tmp/site/Guide.rocdown')
    assert.deepStrictEqual(argv, {
      product: 'rocdown',
      args: ['view', '/tmp/site/Guide.rocdown', '--no-window', '--port', 'auto']
    })
  })

  test('contributes preview commands and language when clauses', () => {
    const manifest = JSON.parse(
      fs.readFileSync(path.resolve(__dirname, '../../../package.json'), 'utf8')
    ) as {
      contributes: {
        commands: { command: string }[]
        menus: {
          'editor/title': { command: string; when: string }[]
          'editor/title/run': { command: string; when: string }[]
        }
      }
    }
    const commands = manifest.contributes.commands.map(entry => entry.command)
    assert.ok(commands.includes('rocci.preview'))
    assert.ok(commands.includes('rocci.stopPreview'))

    const title = manifest.contributes.menus['editor/title']
    const run = manifest.contributes.menus['editor/title/run']
    const titlePreview = title.find(entry => entry.command === 'rocci.preview')
    const runPreview = run.find(entry => entry.command === 'rocci.preview')
    assert.ok(titlePreview, 'editor/title contributes rocci.preview')
    assert.ok(runPreview, 'editor/title/run contributes rocci.preview')
    assert.strictEqual(titlePreview?.when, PREVIEW_WHEN)
    assert.strictEqual(runPreview?.when, PREVIEW_WHEN)
    assert.ok(!title.some(entry => entry.command === 'rocci.stopPreview'))
    const stop = manifest.contributes.commands.find(entry => entry.command === 'rocci.stopPreview') as
      | { enablement?: string }
      | undefined
    assert.strictEqual(stop?.enablement, 'rocci.preview.active')
  })

  test('refuses untitled and unsaved schemes', () => {
    assert.strictEqual(canPreviewDocument('untitled', undefined), false)
    assert.strictEqual(canPreviewDocument('untitled', ''), false)
    assert.strictEqual(canPreviewDocument('file', '/tmp/App.rocci'), true)
  })

  test('reuses the same site or app origin and restarts on product change', () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'rocci-preview-'))
    fs.writeFileSync(path.join(root, 'rocdown.toml'), '')
    const pages = path.join(root, 'pages')
    fs.mkdirSync(pages)
    const guide = path.join(pages, 'Guide.rocdown')
    const note = path.join(pages, 'Note.rocdown')
    fs.writeFileSync(guide, '')
    fs.writeFileSync(note, '')
    const first = previewOrigin(guide)
    const second = previewOrigin(note)
    assert.ok(first && second)
    assert.strictEqual(reuseDecision(undefined, first), 'start')
    assert.strictEqual(reuseDecision(first, second), 'reuse')

    const app = path.join(root, 'App.rocci')
    fs.writeFileSync(app, '')
    const rocci = previewOrigin(app)
    assert.ok(rocci)
    assert.strictEqual(reuseDecision(first, rocci), 'restart')
  })

  test('chooses Simple Browser when present and iframe otherwise', () => {
    assert.strictEqual(chooseBrowserHost(true), 'simpleBrowser')
    assert.strictEqual(chooseBrowserHost(false), 'iframe')
  })
})

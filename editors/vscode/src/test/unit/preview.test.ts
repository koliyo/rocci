import * as assert from 'assert'
import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'

import { canPreviewDocument, chooseBrowserHost } from '../../preview/browser'
import { previewArgv } from '../../preview/dispatch'
import { belongsToOrigin, previewOrigin, reuseDecision } from '../../preview/origin'
import { parsePreviewUrl } from '../../preview/parse'
import {
  countPreviewReadyLines,
  countRebuildLines,
  hasSseReloadEvent,
  liveReloadEventsUrl,
  withReloadNonce
} from '../../preview/reload'

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
      args: ['run', '--no-window', '--port', 'auto', '--verbose', '/tmp/App.rocci']
    })
  })

  test('dispatches .rocdown to rocdown view --no-window --port auto', () => {
    const argv = previewArgv('/tmp/site/Guide.rocdown')
    assert.deepStrictEqual(argv, {
      product: 'rocdown',
      args: ['view', '--no-window', '--port', 'auto', '--verbose', '/tmp/site/Guide.rocdown']
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
    assert.ok(commands.includes('rocci.reloadPreview'))
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
    const reload = manifest.contributes.commands.find(
      entry => entry.command === 'rocci.reloadPreview'
    ) as { enablement?: string } | undefined
    assert.strictEqual(stop?.enablement, 'rocci.preview.active')
    assert.strictEqual(reload?.enablement, 'rocci.preview.active')
    const titleReload = title.find(entry => entry.command === 'rocci.reloadPreview')
    assert.ok(titleReload?.when?.includes('rocci.preview.active'))
  })

  test('builds a cache-busting reload URL and detects SSE reload events', () => {
    assert.strictEqual(
      liveReloadEventsUrl('http://127.0.0.1:8000/guide/'),
      'http://127.0.0.1:8000/__rocci/events'
    )
    assert.strictEqual(
      withReloadNonce('http://127.0.0.1:8000/guide/', 3),
      'http://127.0.0.1:8000/guide/?_r=3'
    )
    assert.ok(hasSseReloadEvent('event: reload\ndata: 2\n\n'))
    assert.ok(!hasSseReloadEvent('event: log\ndata: ok\n\n'))
    assert.strictEqual(
      countPreviewReadyLines('preview_ready http://127.0.0.1:8000/\npreview_ready http://127.0.0.1:8000/\n'),
      2
    )
    assert.strictEqual(countRebuildLines('rocdown: rebuilding\nrocdown: rebuilt\n'), 1)
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
    assert.ok(belongsToOrigin(note, first))
    assert.ok(belongsToOrigin(app, first))
    const outside = path.join(os.tmpdir(), 'rocci-preview-outside.rocci')
    assert.ok(!belongsToOrigin(outside, first))
  })

  test('chooses Simple Browser when present and iframe otherwise', () => {
    assert.strictEqual(chooseBrowserHost(true), 'simpleBrowser')
    assert.strictEqual(chooseBrowserHost(false), 'iframe')
  })
})

import * as assert from 'assert'
import * as fs from 'fs'
import * as path from 'path'

import { previewArgv } from '../../preview/dispatch'
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
  })
})

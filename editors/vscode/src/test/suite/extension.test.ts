import * as assert from 'assert'
import * as path from 'path'
import * as vscode from 'vscode'

const LEGEND_TYPES = [
  'keyword',     // 0
  'function',    // 1
  'type',        // 2
  'namespace',   // 3
  'property',    // 4
  'string',      // 5
  'parameter',   // 6
  'operator',    // 7
  'variable',    // 8
  'number',      // 9
  'comment',     // 10
  'enumMember',  // 11
  'struct',      // 12
  'macro',       // 13
  'decorator'    // 14
]

interface DecodedToken {
  line: number
  character: number
  length: number
  type: string
  modifiers: number
}

function decodeSemanticTokens(tokens: vscode.SemanticTokens): DecodedToken[] {
  const data = tokens.data
  const decoded: DecodedToken[] = []

  let currentLine = 0
  let currentChar = 0

  for (let i = 0; i < data.length; i += 5) {
    const deltaLine = data[i]
    const deltaStartChar = data[i + 1]
    const length = data[i + 2]
    const tokenTypeIndex = data[i + 3]
    const tokenModifiers = data[i + 4]

    currentLine += deltaLine
    if (deltaLine > 0) {
      currentChar = deltaStartChar
    } else {
      currentChar += deltaStartChar
    }

    const type = LEGEND_TYPES[tokenTypeIndex] || `unknown_${tokenTypeIndex}`
    decoded.push({
      line: currentLine,
      character: currentChar,
      length,
      type,
      modifiers: tokenModifiers
    })
  }

  return decoded
}

function assertValidTokenStream(tokens: DecodedToken[]) {
  assert.ok(tokens.length > 0, 'Expected non-empty token stream')

  for (let i = 0; i < tokens.length; i++) {
    const t = tokens[i]
    assert.ok(t.length > 0, `Token at line ${t.line}, col ${t.character} has length <= 0`)

    if (i > 0) {
      const prev = tokens[i - 1]
      if (prev.line === t.line) {
        assert.ok(
          prev.character + prev.length <= t.character,
          `Token overlap on line ${t.line}: prev [${prev.character}..${prev.character + prev.length}), curr [${t.character}..${t.character + t.length})`
        )
      } else {
        assert.ok(prev.line < t.line, `Tokens not sorted by line: prev line ${prev.line}, curr line ${t.line}`)
      }
    }
  }
}

async function waitForServerReady(timeoutMs = 15000): Promise<void> {
  const start = Date.now()
  while (Date.now() - start < timeoutMs) {
    // Check if the extension is activated
    const ext = vscode.extensions.getExtension('koliyo.rocci')
    if (ext && ext.isActive) {
      return
    }
    await new Promise(r => setTimeout(r, 200))
  }
}

async function requestSemanticTokensWithRetry(
  uri: vscode.Uri,
  retries = 10,
  delayMs = 500
): Promise<vscode.SemanticTokens> {
  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      const tokens = await vscode.commands.executeCommand<vscode.SemanticTokens>(
        'vscode.provideDocumentSemanticTokens',
        uri
      )
      if (tokens && tokens.data && tokens.data.length > 0) {
        return tokens
      }
    } catch (e) {
      if (attempt === retries) {
        throw e
      }
    }
    await new Promise(r => setTimeout(r, delayMs))
  }
  throw new Error(`Failed to retrieve semantic tokens for ${uri.toString()} after ${retries} attempts`)
}

suite('Rocci VS Code Extension Integration Tests', () => {
  const rootWorkspace = path.resolve(__dirname, '../../../../../test')

  suiteSetup(async () => {
    const ext = vscode.extensions.getExtension('koliyo.rocci')
    assert.ok(ext, 'Rocci extension koliyo.rocci should be present')
    if (!ext.isActive) {
      await ext.activate()
    }
    await waitForServerReady()
  })

  test('Extension is active and registered languages', () => {
    const ext = vscode.extensions.getExtension('koliyo.rocci')
    assert.ok(ext?.isActive, 'Extension should be active')
  })

  test('Preview commands are registered', async () => {
    const commands = await vscode.commands.getCommands(true)
    assert.ok(commands.includes('rocci.preview'))
    assert.ok(commands.includes('rocci.stopPreview'))
  })

  test('EmbeddedLanguages.rocci semantic tokens and embedded highlighting', async () => {
    const fixturePath = path.join(rootWorkspace, 'EmbeddedLanguages.rocci')
    const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(fixturePath))
    await vscode.window.showTextDocument(doc)

    const rawTokens = await requestSemanticTokensWithRetry(doc.uri)
    const tokens = decodeSemanticTokens(rawTokens)

    assertValidTokenStream(tokens)

    // Token classifications present across host, Roc, CSS, HTML
    const typesFound = new Set(tokens.map(t => t.type))
    assert.ok(typesFound.has('keyword'), 'Should contain keyword tokens')
    assert.ok(typesFound.has('type'), 'Should contain type tokens')
    assert.ok(typesFound.has('function'), 'Should contain function tokens')
    assert.ok(typesFound.has('property'), 'Should contain property tokens')
    assert.ok(typesFound.has('variable'), 'Should contain variable tokens')
    assert.ok(typesFound.has('string'), 'Should contain string tokens')
    assert.ok(typesFound.has('number'), 'Should contain number tokens')
    assert.ok(typesFound.has('operator'), 'Should contain operator tokens')
    assert.ok(typesFound.has('enumMember'), 'Should contain enumMember tokens')

    // Check specific line tokens:
    // Line 0: module EmbeddedLanguages exposing [view, Status, main]
    const line0Tokens = tokens.filter(t => t.line === 0)
    assert.ok(line0Tokens.some(t => t.type === 'keyword'), 'Line 0 has keyword module/exposing')

    // Lines 18-35: @css block has property and string/number/variable tokens
    const cssTokens = tokens.filter(t => t.line >= 18 && t.line <= 35)
    assert.ok(cssTokens.some(t => t.type === 'property'), '@css contains property tokens')

    // Lines 78-85: HTML element and component template
    const templateTokens = tokens.filter(t => t.line >= 78 && t.line <= 85)
    assert.ok(templateTokens.length > 0, 'Template contains tokens')
  })

  test('EmbeddedLanguages.rocci range semantic tokens', async () => {
    const fixturePath = path.join(rootWorkspace, 'EmbeddedLanguages.rocci')
    const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(fixturePath))

    const fullTokens = decodeSemanticTokens(await requestSemanticTokensWithRetry(doc.uri))

    // Request sub-range for CSS block (lines 18 to 35)
    const range = new vscode.Range(18, 0, 35, 1)
    const rawRangeTokens = await vscode.commands.executeCommand<vscode.SemanticTokens>(
      'vscode.provideDocumentRangeSemanticTokens',
      doc.uri,
      range
    )
    assert.ok(rawRangeTokens, 'Range semantic tokens returned')
    const rangeTokens = decodeSemanticTokens(rawRangeTokens)

    assertValidTokenStream(rangeTokens)

    // Expected tokens from full list intersecting the range
    const expectedTokens = fullTokens.filter(t => t.line >= 18 && t.line <= 35)
    assert.strictEqual(
      rangeTokens.length,
      expectedTokens.length,
      `Range token count (${rangeTokens.length}) matches full tokens in range (${expectedTokens.length})`
    )
  })

  test('EmbeddedLanguages.rocdown semantic tokens and display fences', async () => {
    const fixturePath = path.join(rootWorkspace, 'EmbeddedLanguages.rocdown')
    const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(fixturePath))
    await vscode.window.showTextDocument(doc)

    const rawTokens = await requestSemanticTokensWithRetry(doc.uri)
    const tokens = decodeSemanticTokens(rawTokens)

    assertValidTokenStream(tokens)

    const typesFound = new Set(tokens.map(t => t.type))
    assert.ok(typesFound.has('keyword'), 'Rocdown has keywords')
    assert.ok(typesFound.has('property'), 'Rocdown has properties')
    assert.ok(typesFound.has('string'), 'Rocdown has strings')

    // Check display fences (lines 149-174):
    // Roc fence (lines 149-152), HTML fence (lines 154-158), CSS fence (lines 160-165)
    const rocHostFenceTokens = tokens.filter(t => t.line >= 150 && t.line <= 152)
    assert.ok(rocHostFenceTokens.length > 0, 'Fenced roc block contains tokens')

    const htmlFenceTokens = tokens.filter(t => t.line >= 154 && t.line <= 158)
    assert.ok(htmlFenceTokens.length > 0, 'Fenced html block contains tokens')

    const cssFenceTokens = tokens.filter(t => t.line >= 160 && t.line <= 165)
    assert.ok(cssFenceTokens.length > 0, 'Fenced css block contains tokens')
  })

  test('Same-file component definition jumps to component declaration', async () => {
    const fixturePath = path.join(rootWorkspace, 'EmbeddedLanguages.rocci')
    const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(fixturePath))

    // Line 139 has <UserCard user={u} ...>
    // Col 27 is on UserCard
    const position = new vscode.Position(138, 27)
    const locations = await vscode.commands.executeCommand<vscode.Location[] | vscode.LocationLink[]>(
      'vscode.executeDefinitionProvider',
      doc.uri,
      position
    )

    assert.ok(locations && locations.length > 0, 'Definition provider returned locations')
    const firstLoc = locations[0]
    const targetRange = 'targetRange' in firstLoc ? firstLoc.targetRange : (firstLoc as vscode.Location).range
    // UserCard is declared at line 49 (0-indexed: 48)
    assert.strictEqual(targetRange.start.line, 48, 'Definition points to line 49 of EmbeddedLanguages.rocci')
  })

  test('Syntax diagnostics publication on malformed document', async () => {
    const invalidContent = '@component BadComp = |{}| {\n  <div unclosed="yes">\n}\n'
    const doc = await vscode.workspace.openTextDocument({
      language: 'rocci',
      content: invalidContent
    })
    await vscode.window.showTextDocument(doc)

    // Wait for diagnostics with retry
    let diagnostics: vscode.Diagnostic[] = []
    for (let i = 0; i < 15; i++) {
      diagnostics = vscode.languages.getDiagnostics(doc.uri)
      if (diagnostics.length > 0) {
        break
      }
      await new Promise(r => setTimeout(r, 300))
    }

    assert.ok(diagnostics.length > 0, 'Expected syntax error diagnostics for unclosed tag')
  })
})

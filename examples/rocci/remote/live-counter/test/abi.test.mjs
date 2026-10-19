import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { host } from './support.mjs';
const initial = () => ({ count: '0', revision: '0', clicks: [] });
const increment = (origin = '', tz = '') => ({ v: 1, id: crypto.randomUUID(), command: 'increment', origin, tz });
const reset = () => ({ v: 1, id: crypto.randomUUID(), command: 'reset' });
test('real Roc decisions, origin fallbacks, reset, sequence and trimming', () => {
  const roc = new host.Roc();
  let s = initial();
  for (const [origin, tz, expected] of [['', '', 'UTC'], ['', 'Europe/Oslo', 'Oslo, Norway'], ['Europe/Stockholm', 'UTC', 'Stockholm, Sweden'], ['Unknown/<>&"🌍\r\n東京', '', '&lt;&gt;&amp;&quot;🌍\r\n東京']]) {
    const next = roc.call(s, increment(origin, tz), '100');
    assert.ok(next.html.includes(expected), next.html);
    s = next.state;
  }
  const before = structuredClone(s.clicks);
  s = roc.call(s, reset(), '100').state;
  assert.equal(s.count, '0'); assert.deepEqual(s.clicks, before);
  for (let i = 0; i < 60; i++) s = roc.call(s, increment('Europe/Oslo'), '100').state;
  assert.equal(s.count, '60'); assert.equal(s.clicks.length, 50);
  assert.equal(s.revision, '65'); assert.equal(s.clicks[0].seq, '65'); assert.equal(s.clicks[49].seq, '16');
  assert.throws(() => roc.call({ ...s, revision: host.MAX.toString() }, increment(), '100'), /overflow/);
  assert.throws(() => roc.call({ count: host.MAX.toString(), revision: '100', clicks: [] }, increment(), '100'), /overflow/);
  const fresh = new host.Roc().call(s, undefined, '100');
  assert.deepEqual(fresh, roc.call(s, undefined, '100'));
});
test('native and independent Wasm share byte-identical decisions and HTML', () => {
  const roc = new host.Roc();
  const full = { count: '50', revision: '50', clicks: Array.from({ length: 50 }, (_, i) => ({ seq: String(50 - i), ts: String(100 - i), chosen: i % 2 === 0, tz: i % 2 ? 'Europe/Stockholm' : 'Unknown/東京 &<"\r\n' })) };
  for (const input of [initial(), full]) for (const cmd of [undefined, reset(), increment('', 'Europe/Oslo'), increment('Unknown/東京 &<"\r\n')]) for (const document of [false, true]) {
    const text = host.encode(input, cmd, '3700', document);
    const native = spawnSync('./dist/native', [], { input: text, encoding: 'utf8' });
    assert.equal(native.status, 0, native.stderr);
    assert.deepEqual(host.decode(native.stdout), roc.call(input, cmd, '3700', document));
    if (document) assert.match(host.decode(native.stdout).html, /retry: (?:&#39;|')always(?:&#39;|')/,
      'Remote subscriptions must reconnect after clean EOF as well as network errors');
  }
});
test('relative time minute/hour/day boundaries and presentation without mutations', () => {
  const roc = new host.Roc();
  const s = roc.call(initial(), increment(), '100').state;
  for (const [delta, label] of [[0, 'just now'], [59, 'just now'], [60, '1 minute ago'], [3599, '59 minutes ago'], [3600, '1 hour ago'], [86400, '1 day ago'], [172800, '2 days ago']]) {
    const result = roc.call(s, undefined, String(100 + delta));
    assert.ok(result.html.includes(label)); assert.deepEqual(result.state, s);
  }
});
test('bounded ABI, traps, malformed values, no imports, 10000 calls with full feeds', async () => {
  const bytes = await readFile('dist/actor.wasm');
  const module = new WebAssembly.Module(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  const e = new WebAssembly.Instance(module).exports;
  assert.throws(() => e.invoke(256 * 1024 + 1), WebAssembly.RuntimeError);
  const raw = new WebAssembly.Instance(module).exports;
  for (const malformed of ['', '2\nlive', '1\nlive\n9223372036854775808\n0\n100\nread\n\n', '1\nlive\n0\n0\n100\nincrement\nff\n']) {
    const input = new TextEncoder().encode(malformed);
    new Uint8Array(raw.memory.buffer).set(input, raw.input_ptr()); raw.invoke(input.length);
    assert.equal(new TextDecoder().decode(new Uint8Array(raw.memory.buffer, raw.output_ptr(), raw.output_len())), '1\ninvalid\n\n');
  }
  const maxText = '<&"\'\r\n🌍東京'.repeat(8);
  let s = { count: '50', revision: '50', clicks: Array.from({ length: 50 }, (_, i) => ({ seq: String(50 - i), ts: '100', chosen: true, tz: maxText })) };
  const a = new host.Roc(), b = new host.Roc();
  const pages = a.pages, started = performance.now();
  for (let i = 0; i < 10000; i++) {
    const result = (i % 2 ? a : b).call(s, undefined, '100');
    assert.ok(result.html.includes('just now'));
    assert.ok(result.html.includes('&lt;&amp;&quot;&#39;'));
    assert.deepEqual(result.state, s);
    if (i % 1000 === 0) assert.equal(a.pages, pages);
  }
  assert.equal(a.pages, pages); assert.equal(b.pages, pages);
  const metrics = { calls: 10000, totalMs: performance.now() - started, wasmBytes: bytes.length,
    pagesPerActor: pages, combinedLinearMemoryBytes: (a.pages + b.pages) * 65536,
    peakArenaBytes: Math.max(a.peakArena, b.peakArena), nodeRssBytes: process.memoryUsage().rss };
  await writeFile('dist/abi-metrics.json', JSON.stringify(metrics, null, 2) + '\n');
  console.log(metrics);
});

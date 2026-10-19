import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';
const source = await readFile('assets/remote.js', 'utf8');
function browser(respond, remembered = '0') {
  const status = { textContent: '' }, requests = [], storage = new Map([['rocci-counter-pending', remembered]]);
  const context = { window: {}, document: { querySelector: () => status }, location: { href: 'https://counter.example/c/demo/' },
    sessionStorage: { getItem: k => storage.get(k), setItem: (k, v) => storage.set(k, v), removeItem: k => storage.delete(k) },
    crypto, AbortSignal, URL, setTimeout: fn => { fn(); return 0; },
    fetch: async (url, request) => { requests.push(JSON.parse(request.body)); return respond(requests.length); },
  };
  vm.runInNewContext(source, context);
  return { adapter: context.window.rocciCounter, status, requests, storage };
}
test('one envelope per intent and same UUID/body through transient retries', async () => {
  const b = browser(attempt => new Response('', { status: attempt === 1 ? 503 : 200 }));
  await b.adapter.command('increment', 'Europe/Oslo', '');
  assert.equal(b.requests.length, 2); assert.deepEqual(b.requests[0], b.requests[1]);
  await Promise.all([b.adapter.command('increment'), b.adapter.command('increment')]);
  assert.equal(new Set(b.requests.map(r => r.id)).size, 3);
  assert.equal(b.status.textContent, ''); assert.equal(b.storage.get('rocci-counter-pending'), '0');
});
test('definite failures show HTTP errors; reload after ambiguity never resubmits', async () => {
  const failed = browser(() => new Response('Counter frozen', { status: 423 }));
  await failed.adapter.command('reset'); assert.match(failed.status.textContent, /423.*Counter frozen/);
  assert.equal(failed.requests.length, 1);
  const unknown = browser(() => { throw new Error('connection lost'); });
  await unknown.adapter.command('increment'); assert.equal(unknown.requests.length, 5);
  assert.equal(new Set(unknown.requests.map(r => r.id)).size, 1); assert.match(unknown.status.textContent, /outcome unknown/);
  const reloaded = browser(() => { throw new Error('should not submit'); }, unknown.storage.get('rocci-counter-pending'));
  assert.equal(reloaded.requests.length, 0); assert.match(reloaded.status.textContent, /has not been resubmitted/);
});

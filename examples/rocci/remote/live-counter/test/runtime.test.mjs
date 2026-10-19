import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { configure } from '../tools/configure.mjs';
// Keep logs and configuration writes in the derived local workspace.
process.env.WRANGLER_LOG_PATH = `${process.cwd()}/.wrangler/logs/tests.log`;
process.env.XDG_CONFIG_HOME = `${process.cwd()}/.wrangler/config`;
process.env.CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV = 'false';
const { createTestHarness } = await import('wrangler');
await configure(false);
const command = (tag = 'increment', extra = {}) => ({ v: 1, id: crypto.randomUUID(), command: tag, ...(tag === 'increment' ? { origin: 'Europe/Oslo', tz: '' } : {}), ...extra });
test('real workerd SQL, fan-out, retry, rollback, reconstruction, export and compatible upgrade', { timeout: 60000 }, async () => {
  const options = { workers: [{ configPath: 'dist/wrangler.local.json', secrets: {} }] };
  const server = createTestHarness(options);
  let restore;
  const readers = [];
  try {
    const { url } = await server.listen();
    const request = (path, value, extra = {}) => server.fetch(path, value === undefined ? undefined : {
      method: 'POST', headers: { 'Content-Type': 'application/json', Origin: url.origin, ...extra }, body: JSON.stringify(value),
    });
    const base = '/c/demo/';
    const manifest = async () => (await request(base + 'manifest')).json();
    const post = (c, extra) => request(base + 'command', c, extra);
    const doc = await request(base); assert.equal(doc.status, 200);
    const html = await doc.text(); assert.ok(html.includes('window.rocciCounter.command')); assert.ok(html.includes('sse') && html.includes('openWhenHidden'));
    assert.ok(doc.headers.get('Content-Security-Policy').includes("'unsafe-eval'"));
    assert.equal((await request('/assets/datastar.js')).status, 200);
    const verification = await request('/assets/verify.html');
    assert.equal(verification.status, 200);
    assert.ok((await verification.text()).includes('Run 100-command check'));
    assert.equal((await request('/c/unknown/')).status, 404);
    assert.equal((await (await request('/c/isolation/manifest')).json()).runtime.revision, '0');
    const abortA = new AbortController(), abortB = new AbortController();
    const stream = await fetch(new URL(base + 'sse', url), { signal: abortA.signal }); assert.equal(stream.status, 200);
    const a = stream.body.getReader(); readers.push(a);
    assert.ok(new TextDecoder().decode((await a.read()).value).includes('id: 0:1'));
    const second = await fetch(new URL(base + 'sse', url), { signal: abortB.signal }), b = second.body.getReader(); readers.push(b);
    await b.read();
    const unique = Array.from({ length: 100 }, () => command());
    const responses = await Promise.all(unique.map(c => post(c)));
    assert.ok(responses.every(r => r.status === 204), responses.map(r => r.status).join(','));
    assert.equal((await manifest()).runtime.revision, '100');
    assert.ok((await manifest()).runtime.sql.rowsWritten >= 200);
    const sql = await server.getWorker().getDurableObjectStorage('COUNTERS', { name: 'demo' });
    let stored = JSON.parse((await sql.exec('SELECT snapshot FROM counter_state'))[0].snapshot);
    assert.equal(stored.count, '100'); assert.equal(stored.clicks.length, 50);
    const all = await Promise.all(Array.from({ length: 15 }, () => post(unique[0])));
    assert.ok(all.every(r => r.status === 204));
    assert.equal((await post({ ...unique[0], origin: 'UTC' })).status, 409);
    assert.equal((await post(command(), { Origin: 'https://evil.example' })).status, 403);
    assert.equal((await post({ ...command(), extra: true })).status, 400);
    assert.equal((await post(command('increment', { origin: 'x'.repeat(8193) }))).status, 413);
    const reset = command('reset'); const resetResponse = await post(reset, { 'Datastar-Request': 'true' }); assert.equal(resetResponse.status, 200, await resetResponse.text());
    assert.equal((await post(reset, { 'Datastar-Request': 'true' })).headers.get('Content-Type'), 'text/event-stream');
    stored = JSON.parse((await sql.exec('SELECT snapshot FROM counter_state'))[0].snapshot);
    assert.equal(stored.count, '0'); assert.equal(stored.clicks.length, 50);
    const replyLost = command(); assert.equal((await post(replyLost, { 'x-test-drop-reply': '1' })).status, 503);
    assert.equal((await post(replyLost)).status, 204);
    assert.equal((await manifest()).runtime.revision, '102');
    // A real SQL abort after state UPDATE must roll back state and receipt together.
    await sql.exec("CREATE TRIGGER fail_receipt BEFORE INSERT ON operations BEGIN SELECT RAISE(ABORT,'fixture rollback'); END");
    const rollback = command(); assert.equal((await post(rollback)).status, 500);
    assert.equal((await manifest()).runtime.revision, '102');
    await sql.exec('DROP TRIGGER fail_receipt');
    assert.equal((await post(rollback)).status, 204);
    const nextPatch = async reader => {
      let accumulated = '';
      const deadline = Date.now() + 8000;
      while (Date.now() < deadline) {
        let timer;
        const { value, done } = await Promise.race([reader.read(), new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('Timed out waiting for revision 103')), deadline - Date.now()); })]).finally(() => clearTimeout(timer));
        if (done) throw new Error('SSE ended before revision 103: ' + accumulated.slice(-300));
        accumulated = (accumulated + new TextDecoder().decode(value)).slice(-1024 * 1024);
        const text = accumulated;
        if (text.includes('id: 103:')) { assert.ok(text.includes('id="counter-feed"')); return text; }
      }
      throw new Error('Viewer did not converge');
    };
    await Promise.all([nextPatch(a), nextPatch(b)]);
    await a.cancel(); await b.cancel(); abortA.abort(); abortB.abort(); readers.length = 0;
    for (let i = 0; i < 80 && (await manifest()).runtime.viewers; i++) await new Promise(resolve => setTimeout(resolve, 250));
    assert.equal((await manifest()).runtime.viewers, 0, 'HTTP disconnect must reclaim subscribers and timers');
    await server.getWorker().evictDurableObject('COUNTERS', { name: 'demo' });
    assert.equal((await post(replyLost)).headers.get('X-Counter-Revision'), '102');
    assert.equal((await manifest()).runtime.revision, '103');
    const reconnect = await request(base + 'sse'), r = reconnect.body.getReader();
    assert.ok(new TextDecoder().decode((await r.read()).value).includes('id: 103:1')); await r.cancel();
    assert.equal((await request(base + 'operator/freeze', {})).status, 204);
    assert.equal((await post(command())).status, 423);
    assert.equal((await post(replyLost)).status, 204);
    const exported = await (await request(base + 'operator/export')).json();
    assert.equal(exported.data.operations.length, 103);
    const local = JSON.parse(await readFile('dist/wrangler.local.json', 'utf8'));
    await writeFile('dist/wrangler.restore.json', JSON.stringify({ ...local, name: 'rocci-counter-restore-fixture' }));
    restore = createTestHarness({ workers: [{ configPath: 'dist/wrangler.restore.json', secrets: {} }] });
    const restoredUrl = (await restore.listen()).url;
    const restoreRequest = (value) => restore.fetch(base + 'operator/import', { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: restoredUrl.origin }, body: JSON.stringify(value) });
    assert.equal((await restoreRequest({ ...exported, checksum: 'bad' })).status, 400);
    assert.equal((await restoreRequest(exported)).status, 204);
    const restoredSql = await restore.getWorker().getDurableObjectStorage('COUNTERS', { name: 'demo' });
    assert.equal((await restoredSql.exec('SELECT snapshot FROM counter_state'))[0].snapshot, JSON.stringify(exported.data.state));
    assert.equal((await restoreRequest(exported)).status, 409);
    await restore.getWorker().evictDurableObject('COUNTERS', { name: 'demo' });
    assert.equal((await (await restore.fetch(base + 'manifest')).json()).runtime.revision, '103');
    // Additive schema version then reload compatible prior configuration, same namespace.
    await server.update({ workers: [{ ...options.workers[0], vars: { STORAGE_SCHEMA: '2' } }] });
    assert.equal((await (await server.fetch(base + 'manifest')).json()).runtime.schema, 2);
    await server.update(options);
    assert.equal((await (await server.fetch(base + 'manifest')).json()).runtime.revision, '103');
    assert.equal((await (await server.fetch(base + 'manifest')).json()).runtime.schema, 2);
    // Receipt cap is tested without spending 10000 HTTP requests. Retry lookup precedes the cap.
    const currentSql = await server.getWorker().getDurableObjectStorage('COUNTERS', { name: 'demo' });
    await currentSql.exec('UPDATE counter_state SET frozen=0 WHERE id=1');
    await currentSql.exec("WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i+1 FROM n WHERE i<9897) INSERT INTO operations SELECT printf('00000000-0000-4000-8000-%012x',i),'fixture-owner','fixture','{}','0','0' FROM n");
    const origin = (await server.listen()).url.origin;
    const sendCap = c => server.fetch(base + 'command', { method: 'POST', headers: { 'Content-Type': 'application/json', Origin: origin }, body: JSON.stringify(c) });
    assert.equal((await sendCap(command())).status, 429);
    assert.equal((await sendCap(replyLost)).status, 204);
    await writeFile('dist/runtime-metrics.json', JSON.stringify({ concurrentCommands: 100, finalRevision: '103', rollback: true, replyLossRetry: true, reconstruction: true, exportRestore: true, additiveUpgradeRollback: true }, null, 2));
  } catch (error) {
    console.error(server.getLogs().slice(-5));
    throw error;
  } finally {
    for (const reader of readers) await reader.cancel().catch(() => {});
    if (restore) await restore.close();
    await server.close();
  }
});
test('production entry denies every unauthenticated path before routing or assets', { timeout: 30000 }, async () => {
  const server = createTestHarness({ workers: [{ configPath: 'dist/wrangler.json', prebuiltWorkerDir: 'dist/bundle', secrets: {} }] });
  try {
    await server.listen();
    const config = JSON.parse(await readFile('dist/wrangler.json', 'utf8'));
    for (const path of ['/', '/c/demo/', '/c/demo/command', '/c/demo/sse', '/c/demo/operator/export', '/assets/datastar.js']) {
      const r = await server.fetch(`https://${config.vars.COUNTER_HOSTNAME}${path}`);
      assert.ok([401, 403].includes(r.status), path);
    }
    assert.equal((await server.fetch('https://alternate.workers.dev/')).status, 403);
  } finally { await server.close(); }
});

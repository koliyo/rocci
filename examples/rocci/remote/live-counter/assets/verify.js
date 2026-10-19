// Run from the authenticated staging browser console. No service-token identity.
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
const expect = (ok, message) => { if (!ok) throw new Error(message); };
export async function api(path, value, base = '/c/demo/') {
  const response = await fetch(base + path, value === undefined ? { cache: 'no-store' } : {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(value),
  });
  if (!response.ok) throw new Error(`${path}: HTTP ${response.status}: ${await response.text()}`);
  return response;
}
export async function exportCounter(base = '/c/demo/') {
  await api('operator/freeze', {}, base);
  const response = await api('operator/export', undefined, base);
  const blob = await response.blob();
  const href = URL.createObjectURL(blob), link = document.createElement('a');
  link.href = href; link.download = `counter-export-${Date.now()}.json`; link.click();
  setTimeout(() => URL.revokeObjectURL(href), 1000);
  return JSON.parse(await blob.text());
}
export async function restoreCounter(exported, base = '/c/demo/') {
  // Target must be an empty counter in the separate disposable restore Worker.
  await api('operator/import', exported, base);
  return (await api('manifest', undefined, base)).json();
}
function subscription(base) {
  const abort = new AbortController();
  const start = fetch(base + 'sse', { signal: abort.signal }).then(response => {
    expect(response.ok, `SSE HTTP ${response.status}`);
    return response.body.getReader();
  });
  let pending = '';
  return {
    async until(revision) {
      const reader = await start, deadline = Date.now() + 10000;
      while (Date.now() < deadline) {
        const chunk = await Promise.race([reader.read(), pause(10000).then(() => { throw new Error('SSE timed out'); })]);
        expect(!chunk.done, 'Stream closed before snapshot');
        pending = (pending + new TextDecoder().decode(chunk.value)).slice(-1024 * 1024);
        const events = pending.split('\n\n'); pending = events.pop();
        for (const event of events) {
          const id = /^id: (\d+):/m.exec(event);
          if (id && BigInt(id[1]) >= BigInt(revision)) {
            expect(event.includes('id="counter"') && event.includes('id="counter-feed"'), 'Incoherent update batch');
            return event;
          }
        }
      }
      throw new Error('No current snapshot');
    },
    async close() { abort.abort(); await (await start).cancel().catch(() => {}); },
  };
}
export async function run(base = '/c/demo/') {
  const a = subscription(base), b = subscription(base), samples = [];
  const envelope = (command, extra = {}) => ({ v: 1, id: crypto.randomUUID(), command, ...extra });
  try {
    const first = await (await api('manifest', undefined, base)).json();
    await Promise.all([a.until(first.runtime.revision), b.until(first.runtime.revision)]);
    await api('command', envelope('reset'), base);
    const commands = Array.from({ length: 100 }, () => envelope('increment', { origin: 'Europe/Oslo', tz: '' }));
    const responses = await Promise.all(commands.map(async command => {
      const start = performance.now(); const r = await api('command', command, base); samples.push(performance.now() - start); return r;
    }));
    const revision = (BigInt(first.runtime.revision) + 101n).toString();
    const patches = await Promise.all([a.until(revision), b.until(revision)]);
    expect(patches.every(p => /<output[^>]*>100<\/output>/.test(p)), 'Concurrent count mismatch');
    await api('command', commands[0], base);
    const after = await (await api('manifest', undefined, base)).json();
    expect(after.runtime.revision === revision, 'Duplicate operation changed state');
    const conflict = await fetch(base + 'command', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ ...commands[0], origin: 'UTC' }) });
    expect(conflict.status === 409, 'Mismatched retry was accepted');
    await b.close();
    const command = envelope('increment', { origin: 'Unknown/東京 &<"', tz: '' });
    const accepted = await api('command', command, base);
    const reopened = subscription(base);
    try { await reopened.until(accepted.headers.get('X-Counter-Revision')); } finally { await reopened.close(); }
    const isolation = await (await api('manifest', undefined, '/c/isolation/')).json();
    samples.sort((a, b) => a - b);
    const evidence = { origin: location.origin, observed: new Date().toISOString(), before: first, after,
      final: await (await api('manifest', undefined, base)).json(), retryEnvelope: commands[0],
      isolation, responseStatuses: [...new Set(responses.map(r => r.status))], latencyMs: { p50: samples[50], p95: samples[95], max: samples.at(-1) } };
    console.log('Owner-browser conformance passed', evidence);
    return evidence;
  } finally { await a.close(); await b.close(); }
}
export async function idle(milliseconds = 600000, base = '/c/demo/') {
  const abort = new AbortController(), started = Date.now();
  const response = await fetch(base + 'sse', { signal: abort.signal });
  expect(response.ok, `SSE HTTP ${response.status}`);
  const reader = response.body.getReader();
  let heartbeats = 0, bytes = 0;
  try {
    while (Date.now() - started < milliseconds) {
      const { value, done } = await Promise.race([reader.read(), pause(30000).then(() => { throw new Error('Heartbeat stopped for 30s'); })]);
      expect(!done, 'Idle stream closed'); bytes += value.length;
      if (new TextDecoder().decode(value).includes(': heartbeat')) heartbeats++;
    }
    return { elapsedMs: Date.now() - started, heartbeats, bytes };
  } finally { abort.abort(); await reader.cancel().catch(() => {}); }
}

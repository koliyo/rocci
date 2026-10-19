// Opt-in local workerd measurement. This does not measure a hosted isolate or a bill.
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { createServer } from 'node:net';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import WebSocket from 'ws'; // Pinned in the Wrangler dependency lockfile.
import { dir } from './configure.mjs';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
async function port() {
  const server = createServer(); server.listen(0, '127.0.0.1'); await once(server, 'listening');
  const value = server.address().port; await new Promise(resolve => server.close(resolve)); return value;
}
const httpPort = await port(), inspectorPort = await port();
const storage = await mkdtemp(path.join(tmpdir(), 'rocci-actor-resources-'));
const child = spawn(process.execPath, ['node_modules/wrangler/bin/wrangler.js', 'dev', '--config', 'dist/wrangler.local.json',
  '--ip', '127.0.0.1', '--port', String(httpPort), '--inspector-port', String(inspectorPort), '--persist-to', storage], {
  cwd: dir, env: { ...process.env, CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV: 'false', WRANGLER_SEND_METRICS: 'false',
    WRANGLER_LOG_PATH: path.join(dir, '.wrangler/logs/resources.log'), XDG_CONFIG_HOME: path.join(dir, '.wrangler/config') },
  stdio: ['ignore', 'pipe', 'pipe'], detached: true,
});
let log = ''; child.stdout.on('data', d => { log = (log + d).slice(-16000); }); child.stderr.on('data', d => { log = (log + d).slice(-16000); });
const origin = `http://127.0.0.1:${httpPort}`, subscriptions = [];
let socket;
try {
  let targets;
  for (let n = 0; n < 120; n++) {
    if (child.exitCode !== null) throw new Error(log);
    try {
      // The inspector discovery proxy starts before its target Worker is ready.
      const ready = await fetch(origin, { redirect: 'manual' });
      if (ready.status === 303) {
        targets = await (await fetch(`http://127.0.0.1:${inspectorPort}/json`)).json();
        if (targets.length) break;
      }
    } catch {}
    await pause(250);
  }
  if (!targets?.length) throw new Error('Inspector did not start\n' + log);
  socket = new WebSocket(targets[0].webSocketDebuggerUrl, { origin: 'http://localhost' });
  await Promise.race([
    new Promise((resolve, reject) => {
      socket.addEventListener('open', resolve, { once: true });
      socket.addEventListener('error', () => reject(new Error('Inspector WebSocket connection failed')), { once: true });
    }),
    pause(10000).then(() => { throw new Error('Inspector WebSocket did not open\n' + log); }),
  ]);
  let id = 0; const pending = new Map();
  socket.addEventListener('message', event => {
    const data = JSON.parse(event.data), p = pending.get(data.id);
    if (p) { pending.delete(data.id); data.error ? p.reject(new Error(JSON.stringify(data.error))) : p.resolve(data.result); }
  });
  async function cdp(method) {
    const call = ++id;
    return Promise.race([new Promise((resolve, reject) => { pending.set(call, { resolve, reject }); socket.send(JSON.stringify({ id: call, method })); }),
      pause(10000).then(() => { throw new Error(`Inspector timeout: ${method}`); })]);
  }
  const measure = () => cdp('Runtime.getHeapUsage');
  const baseline = await measure(), latencies = [];
  async function command(key) {
    const start = performance.now();
    const r = await fetch(`${origin}/c/${key}/command`, { method: 'POST', headers: { Origin: origin, 'Content-Type': 'application/json' },
      body: JSON.stringify({ v: 1, id: crypto.randomUUID(), command: 'increment', origin: 'é'.repeat(64), tz: 'é'.repeat(64) }) });
    await r.arrayBuffer(); if (r.status !== 204) throw new Error(`Command failed: ${r.status}`);
    latencies.push(performance.now() - start);
  }
  for (let i = 0; i < 50; i++) await Promise.all(['demo', 'isolation'].map(command));
  const fullFeeds = await measure();
  for (const key of ['demo', 'isolation']) for (let i = 0; i < 20; i++) {
    const abort = new AbortController(); const r = await fetch(`${origin}/c/${key}/sse`, { signal: abort.signal });
    if (!r.ok) throw new Error(`Subscription failed: ${r.status}`);
    subscriptions.push({ abort, reader: r.body.getReader() }); // Deliberately do not drain.
  }
  for (let i = 0; i < 10; i++) await Promise.all(['demo', 'isolation'].map(command));
  const stalledReaders = await measure();
  const actors = await Promise.all(['demo', 'isolation'].map(async key => ({ key, ...(await (await fetch(`${origin}/c/${key}/manifest`)).json()).runtime })));
  latencies.sort((a, b) => a - b);
  const result = { observed: new Date().toISOString(), runtime: 'local workerd, inspector attached',
    inspectorTarget: { title: targets[0].title, url: targets[0].url }, baseline, fullFeeds, stalledReaders, actors,
    latencyMs: { p50: latencies[Math.floor(latencies.length * .5)], p95: latencies[Math.floor(latencies.length * .95)], max: latencies.at(-1) },
    limitation: 'V8 heap/backing-store telemetry; excludes unreported native/provider overhead. Stalled clients may still have OS/network buffering. Not whole hosted-isolate memory.' };
  await writeFile(path.join(dir, 'dist/resource-metrics.json'), JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify(result, null, 2));
} finally {
  for (const s of subscriptions) { s.abort.abort(); void s.reader.cancel().catch(() => {}); }
  socket?.close();
  if (child.exitCode === null) {
    process.kill(-child.pid, 'SIGTERM');
    await Promise.race([once(child, 'exit'), pause(5000).then(() => {
      if (child.exitCode === null) process.kill(-child.pid, 'SIGKILL');
    })]);
  }
  await rm(storage, { recursive: true, force: true });
}

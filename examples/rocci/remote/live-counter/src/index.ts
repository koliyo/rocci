import { DurableObject } from 'cloudflare:workers';
import { authenticate, csrf, AuthEnv } from './auth';
import { Roc } from './roc';
import { Streams, frame } from './streams';
import { body, command, canonical, checksum, exactKeys, integer, operationId, state,
  HttpError, State, Command, Snapshot, SCHEMA, RECEIPT_CAP } from './protocol';
import manifest from '../dist/manifest.json';
export interface Env extends AuthEnv {
  COUNTERS: DurableObjectNamespace<Counter>;
  ASSETS: Fetcher;
  COUNTER_KEYS: string;
  STORAGE_SCHEMA: string;
  CF_VERSION_METADATA?: { id: string; tag: string; timestamp: string };
}
type Receipt = { id: string; owner: string; digest: string; result: string; revision: string; accepted: string };
type Stored = { schema_version: number; snapshot: string; revision: string; frozen: number };
const headers = { 'Cache-Control': 'no-store', 'X-Content-Type-Options': 'nosniff' };
function failure(error: unknown): Response {
  if (error instanceof HttpError) return new Response(error.message, { status: error.status, headers });
  console.error('counter host failure', error instanceof Error ? error.message : 'unknown');
  return new Response('Counter unavailable; retry the same operation ID', { status: 500, headers });
}
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    try {
      const owner = await authenticate(request, env);
      csrf(request);
      return await route(request, env, owner);
    } catch (error) { return failure(error); }
  },
};
// Shared with the local identity-fixture entry point; never bypassed by the production entry point.
export async function route(request: Request, env: Env, owner: string): Promise<Response> {
  const url = new URL(request.url);
  if (url.pathname.startsWith('/assets/')) {
    if (request.method !== 'GET' && request.method !== 'HEAD') throw new HttpError(405, 'Method not allowed');
    const assetUrl = new URL(request.url);
    assetUrl.pathname = assetUrl.pathname.slice('/assets'.length);
    const response = await env.ASSETS.fetch(new Request(assetUrl, request));
    const result = new Response(response.body, response);
    result.headers.set('Cache-Control', 'private, no-store');
    result.headers.set('X-Content-Type-Options', 'nosniff');
    return result;
  }
  const keys = env.COUNTER_KEYS.split(',');
  if (keys.length !== 2 || keys.some(k => !/^[a-z0-9-]{1,32}$/.test(k)) || new Set(keys).size !== 2)
    throw new Error('Exactly two distinct counter keys required');
  if (url.pathname === '/' && request.method === 'GET')
    return new Response(null, { status: 303, headers: { ...headers, Location: `/c/${keys[0]}/` } });
  const match = /^\/c\/([a-z0-9-]+)\/(.*)$/.exec(url.pathname);
  if (!match || !keys.includes(match[1])) throw new HttpError(404, 'Unknown counter');
  const forwarded = new Request(request);
  forwarded.headers.set('x-counter-owner', owner);
  const response = await env.COUNTERS.getByName(match[1]).fetch(forwarded);
  if (match[2] !== 'sse' || !response.ok || !response.body) return response;
  // Explicitly forward cancellation across the ingress -> object fetch hop.
  const reader = response.body.getReader();
  const abort = () => { void reader.cancel().catch(() => {}); };
  request.signal.addEventListener('abort', abort, { once: true });
  const stream = new ReadableStream<Uint8Array>({
    async pull(controller) {
      try {
        const chunk = await reader.read();
        if (chunk.done) { request.signal.removeEventListener('abort', abort); controller.close(); }
        else controller.enqueue(chunk.value);
      } catch (error) { request.signal.removeEventListener('abort', abort); controller.error(error); }
    },
    async cancel(reason) {
      request.signal.removeEventListener('abort', abort);
      await reader.cancel(reason);
    },
  }, { highWaterMark: 1 });
  return new Response(stream, response);
}
export class Counter extends DurableObject<Env> {
  private roc = new Roc();
  private sqlMetrics = { statements: 0, rowsRead: 0, rowsWritten: 0 };
  private exec<T extends Record<string, SqlStorageValue> = Record<string, SqlStorageValue>>(query: string, ...params: SqlStorageValue[]) {
    const cursor = this.ctx.storage.sql.exec<T>(query, ...params), rows = cursor.toArray();
    this.sqlMetrics.statements++;
    this.sqlMetrics.rowsRead += cursor.rowsRead;
    this.sqlMetrics.rowsWritten += cursor.rowsWritten;
    return {
      toArray: () => rows,
      one: () => {
        if (rows.length !== 1) throw new Error('Expected exactly one SQL row');
        return rows[0];
      },
    };
  }
  private sequence = 0;
  private capture(input: State, cmd?: Command, now?: string, document = false): Snapshot {
    return { ...this.roc.call(input, cmd, now, document), order: ++this.sequence };
  }
  private streams = new Streams(() => {
    // Presentation ticks use the last completed durable snapshot, never a new SQL read.
    return this.capture(this.streams.latest!.state);
  }, () => {
    // One row per object heartbeat, shared by every viewer. Accessing storage
    // detects a replaced actor even when its old HTTP streams remain alive.
    this.exec('SELECT revision FROM counter_state WHERE id=1').one();
  });
  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    ctx.blockConcurrencyWhile(async () => {
      ctx.storage.transactionSync(() => {
        const sql = { exec: this.exec.bind(this) };
        sql.exec('CREATE TABLE IF NOT EXISTS counter_state (id INTEGER PRIMARY KEY CHECK(id=1), schema_version INTEGER NOT NULL, revision TEXT NOT NULL, snapshot TEXT NOT NULL, frozen INTEGER NOT NULL DEFAULT 0)');
        sql.exec('CREATE TABLE IF NOT EXISTS operations (id TEXT PRIMARY KEY, owner TEXT NOT NULL, digest TEXT NOT NULL, result TEXT NOT NULL, revision TEXT NOT NULL, accepted TEXT NOT NULL)');
        sql.exec('INSERT OR IGNORE INTO counter_state (id,schema_version,revision,snapshot) VALUES (1,?,?,?)', SCHEMA, '0', JSON.stringify({ count: '0', revision: '0', clicks: [] }));
        const stored = this.stored();
        if (![1, 2].includes(stored.schema_version)) throw new Error('Unsupported stored schema');
        if (env.STORAGE_SCHEMA === '2' && stored.schema_version === 1) {
          sql.exec("ALTER TABLE counter_state ADD COLUMN upgrade_note TEXT NOT NULL DEFAULT ''");
          sql.exec('UPDATE counter_state SET schema_version=2 WHERE id=1');
        }
        this.load();
      });
      await ctx.storage.sync();
    });
  }
  private stored(): Stored { return this.exec<Stored>('SELECT schema_version,revision,snapshot,frozen FROM counter_state WHERE id=1').one(); }
  private load(): State {
    const row = this.stored(), value = state(JSON.parse(row.snapshot));
    if (row.revision !== value.revision) throw new Error('State revision mismatch');
    return value;
  }
  private async durable(snapshot: Snapshot): Promise<Snapshot> {
    await this.ctx.storage.sync();
    this.streams.publish(snapshot);
    return this.streams.latest!;
  }
  async fetch(request: Request): Promise<Response> {
    try { return await this.respond(request); } catch (error) { return failure(error); }
  }
  private async respond(request: Request): Promise<Response> {
    const owner = request.headers.get('x-counter-owner');
    if (!owner) throw new HttpError(401, 'Missing verified owner');
    const path = new URL(request.url).pathname.replace(/^\/c\/[^/]+\//, '');
    if (path === 'sse' && request.method === 'GET') {
      if (this.streams.size >= 20) throw new HttpError(429, 'Viewer capacity reached');
      // Registration and capture are one synchronous actor turn.
      const { response, viewer } = this.streams.subscribe();
      try {
        const snapshot = this.capture(this.load());
        await this.durable(snapshot);
        if (viewer.serial === 0) this.streams.deliver(viewer, this.streams.latest!);
        if (request.signal.aborted) this.streams.remove(viewer);
        request.signal.addEventListener('abort', () => this.streams.remove(viewer), { once: true });
        return response;
      } catch (error) { this.streams.remove(viewer); throw error; }
    }
    if (path === '' && request.method === 'GET') {
      const snapshot = this.capture(this.load(), undefined, undefined, true);
      await this.ctx.storage.sync();
      return new Response(snapshot.html, { headers: { ...headers, 'Content-Type': 'text/html; charset=utf-8',
        'Content-Security-Policy': "default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self' data:; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
        'Referrer-Policy': 'no-referrer', 'X-Counter-Revision': snapshot.state.revision } });
    }
    if (path === 'command' && request.method === 'POST') {
      const cmd = command(await body(request)), digest = await checksum(canonical(cmd));
      const accepted = Math.floor(Date.now() / 1000).toString();
      const result = this.ctx.storage.transactionSync(() => {
        const sql = { exec: this.exec.bind(this) };
        const found = sql.exec<Receipt>('SELECT * FROM operations WHERE id=?', cmd.id).toArray()[0];
        if (found) {
          if (found.owner !== owner || found.digest !== digest) throw new HttpError(409, 'Operation ID payload or owner mismatch');
          return { revision: found.revision, snapshot: this.capture(this.load()) };
        }
        if (this.stored().frozen) throw new HttpError(423, 'Counter is frozen for export');
        if (sql.exec<{ n: number }>('SELECT COUNT(*) AS n FROM operations').one().n >= RECEIPT_CAP)
          throw new HttpError(429, 'Receipt capacity reached; export this experiment');
        const snapshot = this.capture(this.load(), cmd, accepted);
        frame(snapshot, 1); // Include wire expansion in pre-commit renderability validation.
        sql.exec('UPDATE counter_state SET revision=?,snapshot=? WHERE id=1', snapshot.state.revision, JSON.stringify(snapshot.state));
        const result = JSON.stringify({ revision: snapshot.state.revision });
        sql.exec('INSERT INTO operations VALUES (?,?,?,?,?,?)', cmd.id, owner, digest, result, snapshot.state.revision, accepted);
        return { revision: snapshot.state.revision, snapshot };
      });
      await this.durable(result.snapshot);
      const datastar = request.headers.get('Datastar-Request') === 'true';
      return new Response(datastar ? '' : null, { status: datastar ? 200 : 204,
        headers: { ...headers, 'Content-Type': 'text/event-stream', 'X-Counter-Revision': result.revision } });
    }
    if (path.startsWith('operator/')) return this.operator(path.slice(9), request, owner);
    if (path === 'manifest' && request.method === 'GET') {
      return Response.json({ ...manifest, deployment: this.env.CF_VERSION_METADATA ?? null,
        runtime: { wasmPages: this.roc.pages, peakArenaBytes: this.roc.peakArena, viewers: this.streams.size,
          revision: this.load().revision, schema: this.stored().schema_version,
          sql: { ...this.sqlMetrics }, streams: { ...this.streams.metrics } } }, { headers });
    }
    throw new HttpError(404, 'Unknown route or method');
  }
  private async operator(path: string, request: Request, owner: string): Promise<Response> {
    const sql = { exec: this.exec.bind(this) };
    if (['freeze', 'unfreeze'].includes(path) && request.method === 'POST') {
      exactKeys(await body(request), []);
      sql.exec('UPDATE counter_state SET frozen=? WHERE id=1', path === 'freeze' ? 1 : 0);
      await this.ctx.storage.sync();
      return new Response(null, { status: 204, headers });
    }
    if (path === 'export' && request.method === 'GET') {
      if (!this.stored().frozen) throw new HttpError(409, 'Freeze writes before exporting');
      const data = { schema: this.stored().schema_version, state: this.load(),
        operations: sql.exec<Receipt>('SELECT * FROM operations ORDER BY id').toArray() };
      await this.ctx.storage.sync();
      return Response.json({ v: 1, data, checksum: await checksum(JSON.stringify(data)) }, { headers });
    }
    if (path === 'import' && request.method === 'POST') {
      const envelope = await body(request, 8 * 1024 * 1024);
      exactKeys(envelope, ['v', 'data', 'checksum']);
      if (envelope.v !== 1 || typeof envelope.checksum !== 'string') throw new HttpError(400, 'Invalid export');
      const data = envelope.data;
      exactKeys(data, ['schema', 'state', 'operations']);
      if (![1, 2].includes(data.schema as number) || !Array.isArray(data.operations) || data.operations.length > RECEIPT_CAP)
        throw new HttpError(400, 'Unsupported export');
      const imported = state(data.state);
      const seen = new Set<string>();
      const revisions = new Set<string>();
      const operations = data.operations.map((r: unknown): Receipt => {
        exactKeys(r, ['id', 'owner', 'digest', 'result', 'revision', 'accepted']);
        const id = operationId(r.id), revision = integer(r.revision), accepted = integer(r.accepted);
        if (seen.has(id) || revisions.has(revision) || revision === '0' || r.owner !== owner || typeof r.digest !== 'string' || !/^[0-9a-f]{64}$/.test(r.digest)
          || r.result !== JSON.stringify({ revision }) || BigInt(revision) > BigInt(imported.revision)) throw new HttpError(400, 'Invalid receipt');
        seen.add(id);
        revisions.add(revision);
        return { id, owner, digest: r.digest, result: r.result as string, revision, accepted };
      });
      operations.sort((a, b) => a.id.localeCompare(b.id));
      if (BigInt(operations.length) !== BigInt(imported.revision)) throw new HttpError(400, 'Export revision and receipt count disagree');
      const normalized = { schema: data.schema, state: imported, operations };
      if (await checksum(JSON.stringify(normalized)) !== envelope.checksum) throw new HttpError(400, 'Export checksum mismatch');
      const snapshot = this.capture(imported);
      frame(snapshot, 1);
      this.ctx.storage.transactionSync(() => {
        if (this.load().revision !== '0' || sql.exec<{ n: number }>('SELECT COUNT(*) AS n FROM operations').one().n)
          throw new HttpError(409, 'Import requires an empty disposable counter');
        for (const r of operations) sql.exec('INSERT INTO operations VALUES (?,?,?,?,?,?)', r.id, r.owner, r.digest, r.result, r.revision, r.accepted);
        sql.exec('UPDATE counter_state SET revision=?,snapshot=?,frozen=1 WHERE id=1', imported.revision, JSON.stringify(imported));
      });
      await this.durable(snapshot);
      return new Response(null, { status: 204, headers });
    }
    throw new HttpError(404, 'Unknown operator route');
  }
}

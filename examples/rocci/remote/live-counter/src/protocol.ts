export const ABI = 1;
export const SCHEMA = 1;
export const MAX = 9007199254740991n;
export const BODY_CAP = 8192;
export const STATE_CAP = 256 * 1024;
export const OUTPUT_CAP = 512 * 1024;
export const RECEIPT_CAP = 10000;
export const encoder = new TextEncoder();
export const decoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
export type Click = { seq: string; ts: string; tz: string; chosen: boolean };
export type State = { count: string; revision: string; clicks: Click[] };
export type Command = { v: 1; id: string; command: 'increment'; origin: string; tz: string }
  | { v: 1; id: string; command: 'reset' };
export type Snapshot = { state: State; html: string; order?: number };
export class HttpError extends Error {
  constructor(public status: number, message: string) { super(message); }
}
export function exactKeys(value: unknown, keys: string[]): asserts value is Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)
    || Object.keys(value).sort().join(',') !== [...keys].sort().join(','))
    throw new HttpError(400, 'Unexpected or missing fields');
}
export function text(value: unknown): string {
  if (typeof value !== 'string' || encoder.encode(value).length > 128
    || !value.isWellFormed()) throw new HttpError(400, 'Invalid text field');
  return value;
}
export function integer(value: unknown): string {
  if (typeof value !== 'string' || !/^(0|[1-9][0-9]{0,15})$/.test(value)
    || BigInt(value) > MAX) throw new HttpError(400, 'Invalid exact integer');
  return value;
}
export function operationId(value: unknown): string {
  if (typeof value !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value))
    throw new HttpError(400, 'Operation ID must be a UUID');
  return value.toLowerCase();
}
export function command(value: unknown): Command {
  const tag = (value as Record<string, unknown> | null)?.command;
  exactKeys(value, tag === 'increment' ? ['v', 'id', 'command', 'origin', 'tz'] : ['v', 'id', 'command']);
  if (value.v !== ABI) throw new HttpError(400, 'Unsupported command version');
  const id = operationId(value.id);
  switch (tag) {
    case 'increment': return { v: ABI, id, command: tag, origin: text(value.origin), tz: text(value.tz) };
    case 'reset': return { v: ABI, id, command: tag };
    default: throw new HttpError(400, 'Unknown command');
  }
}
export function state(value: unknown): State {
  exactKeys(value, ['count', 'revision', 'clicks']);
  const count = integer(value.count), revision = integer(value.revision);
  if (!Array.isArray(value.clicks) || value.clicks.length > 50) throw new HttpError(400, 'Invalid feed');
  let before = BigInt(revision) + 1n;
  const clicks = value.clicks.map((row: unknown) => {
    exactKeys(row, ['seq', 'ts', 'tz', 'chosen']);
    const seq = integer(row.seq), ts = integer(row.ts), tz = text(row.tz);
    if (BigInt(seq) < 1n || BigInt(seq) >= before || typeof row.chosen !== 'boolean')
      throw new HttpError(400, 'Invalid feed order');
    before = BigInt(seq);
    return { seq, ts, tz, chosen: row.chosen };
  });
  const result = { count, revision, clicks };
  if (BigInt(count) > BigInt(revision) || encoder.encode(JSON.stringify(result)).length > STATE_CAP)
    throw new HttpError(400, 'Invalid snapshot');
  return result;
}
export function canonical(cmd: Command): string {
  return JSON.stringify(cmd.command === 'reset' ? [ABI, 'reset'] : [ABI, 'increment', cmd.origin, cmd.tz]);
}
export async function checksum(value: string): Promise<string> {
  const hash = await crypto.subtle.digest('SHA-256', encoder.encode(value));
  return Array.from(new Uint8Array(hash), b => b.toString(16).padStart(2, '0')).join('');
}
export async function body(request: Request, cap = BODY_CAP): Promise<unknown> {
  const length = request.headers.get('content-length');
  if (length && (!/^\d+$/.test(length) || Number(length) > cap)) throw new HttpError(413, 'Body too large');
  if (request.headers.get('content-type')?.split(';')[0].trim() !== 'application/json')
    throw new HttpError(415, 'Expected application/json');
  const reader = request.body?.getReader();
  const chunks: Uint8Array[] = [];
  let size = 0;
  if (reader) {
    try {
      for (;;) {
        const { value, done } = await reader.read();
        if (done) break;
        size += value.byteLength;
        if (size > cap) { await reader.cancel(); throw new HttpError(413, 'Body too large'); }
        chunks.push(value);
      }
    } finally { reader.releaseLock(); }
  }
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
  try { return JSON.parse(decoder.decode(bytes)); }
  catch { throw new HttpError(400, 'Invalid UTF-8 JSON'); }
}

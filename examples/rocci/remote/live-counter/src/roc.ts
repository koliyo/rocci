import module from '../dist/actor.wasm';
import { Command, State, Snapshot, HttpError, encoder, decoder, integer, state, OUTPUT_CAP } from './protocol';
type Exports = {
  memory: WebAssembly.Memory; input_ptr(): number; output_ptr(): number; output_len(): number;
  invoke(length: number): void; arena_used(): number;
};
const hex = (text: string) => text === '' ? '-' : Array.from(encoder.encode(text), b => b.toString(16).padStart(2, '0')).join('');
function unhex(text: string): string {
  if (text === '-') return '';
  if (!/^(?:[0-9a-f]{2})*$/.test(text)) throw new Error('Invalid Roc hex');
  return decoder.decode(Uint8Array.from(text.match(/../g) ?? [], b => parseInt(b, 16)));
}
export function encode(input: State, cmd: Command | undefined, now: string, document: boolean): string {
  return ['1', document ? 'document' : 'live', input.count, input.revision, integer(now),
    cmd?.command ?? 'read', hex(cmd?.command === 'increment' ? cmd.origin : ''),
    hex(cmd?.command === 'increment' ? cmd.tz : ''),
    ...input.clicks.map(c => [c.seq, c.ts, c.chosen ? '1' : '0', hex(c.tz)].join('|'))].join('\n');
}
export function decode(output: string): Snapshot {
  const split = output.indexOf('\n\n');
  if (split < 0) throw new Error('Invalid Roc output');
  const [version, status, count, revision, ...rows] = output.slice(0, split).split('\n');
  if (version !== '1') throw new Error('Invalid Roc ABI version');
  if (status === 'rejected') throw new HttpError(422, count);
  if (status !== 'ok') throw new Error('Roc rejected the ABI input');
  const clicks = rows.map(line => {
    const fields = line.split('|');
    if (fields.length !== 4 || !['0', '1'].includes(fields[2])) throw new Error('Invalid Roc feed');
    return { seq: fields[0], ts: fields[1], chosen: fields[2] === '1', tz: unhex(fields[3]) };
  });
  const html = output.slice(split + 2);
  if (encoder.encode(html).length > OUTPUT_CAP || !html.includes('id="counter"') || !html.includes('id="counter-feed"'))
    throw new Error('Invalid Roc HTML output');
  return { state: state({ count, revision, clicks }), html };
}
export class Roc {
  private instance = new WebAssembly.Instance(module, {}).exports as unknown as Exports;
  peakArena = 0;
  call(input: State, cmd?: Command, now = Math.floor(Date.now() / 1000).toString(), document = false): Snapshot {
    const bytes = encoder.encode(encode(input, cmd, now, document));
    if (bytes.length > 256 * 1024) throw new HttpError(413, 'ABI input too large');
    const e = this.instance;
    try {
      new Uint8Array(e.memory.buffer).set(bytes, e.input_ptr());
      e.invoke(bytes.length);
      const ptr = e.output_ptr(), len = e.output_len();
      if (len > OUTPUT_CAP || ptr < 0 || ptr + len > e.memory.buffer.byteLength) throw new Error('Roc memory bounds');
      const output = decoder.decode(new Uint8Array(e.memory.buffer, ptr, len).slice());
      this.peakArena = Math.max(this.peakArena, e.arena_used());
      return decode(output);
    } catch (error) {
      // A trap can leave the Wasm stack pointer unusable. Reconstruct the heap.
      if (error instanceof WebAssembly.RuntimeError) this.instance = new WebAssembly.Instance(module, {}).exports as unknown as Exports;
      throw error;
    }
  }
  get pages(): number { return this.instance.memory.buffer.byteLength / 65536; }
}

import { Snapshot, encoder, OUTPUT_CAP } from './protocol';
type Viewer = {
  writer: WritableStreamDefaultWriter<Uint8Array>; pending?: Snapshot;
  revision: bigint; serial: number; closed: boolean; heartbeat: boolean; wake?: () => void;
};
export function frame(snapshot: Snapshot, serial: number): Uint8Array {
  const lines = snapshot.html.split(/\r\n|\r|\n/).map(line => `data: elements ${line}\n`).join('');
  const result = encoder.encode(`id: ${snapshot.state.revision}:${serial}\nevent: datastar-patch-elements\n${lines}\n`);
  if (result.byteLength > OUTPUT_CAP * 2) throw new Error('SSE frame too large');
  return result;
}
export class Streams {
  readonly metrics = { frames: 0, bytes: 0, coalesced: 0, heartbeats: 0 };
  private viewers = new Set<Viewer>();
  private heartbeat?: ReturnType<typeof setInterval>;
  private clock?: ReturnType<typeof setInterval>;
  latest?: Snapshot;
  constructor(private refresh: () => Snapshot) {}
  get size(): number { return this.viewers.size; }
  subscribe(): { response: Response; viewer: Viewer } {
    if (this.size >= 20) throw new Error('Viewer capacity reached');
    const { readable, writable } = new TransformStream<Uint8Array, Uint8Array>();
    const writer = writable.getWriter();
    const viewer: Viewer = { writer, revision: -1n, serial: 0, closed: false, heartbeat: false };
    this.viewers.add(viewer);
    // Start one writer pump in this subscriber's request context. Mutation
    // requests only replace pending values and wake this pump; they never
    // start network writes on behalf of a different request.
    void this.pump(viewer).catch(() => this.remove(viewer));
    // Native stream cancellation rejects writer.closed even while no snapshot is pending.
    void writer.closed.then(() => this.remove(viewer), () => this.remove(viewer));
    if (!this.heartbeat) {
      this.heartbeat = setInterval(() => {
        for (const v of this.viewers) { v.heartbeat = true; v.wake?.(); }
      }, 15000);
      this.clock = setInterval(() => {
        try { if (this.latest) this.publish(this.refresh()); }
        catch { for (const viewer of this.viewers) this.remove(viewer); }
      }, 60000);
    }
    return { viewer, response: new Response(readable, { headers: {
      'Content-Type': 'text/event-stream', 'Cache-Control': 'no-store, no-transform',
      'X-Accel-Buffering': 'no', 'X-Content-Type-Options': 'nosniff',
    } }) };
  }
  deliver(viewer: Viewer, snapshot: Snapshot): void {
    if (viewer.closed || BigInt(snapshot.state.revision) < viewer.revision) return;
    if (viewer.pending) this.metrics.coalesced++;
    viewer.pending = snapshot;
    viewer.wake?.();
  }
  private async pump(viewer: Viewer): Promise<void> {
    while (!viewer.closed) {
      if (!viewer.pending && !viewer.heartbeat) {
        await new Promise<void>(resolve => { viewer.wake = resolve; });
        continue;
      }
      const snapshot = viewer.pending;
      viewer.pending = undefined;
      viewer.heartbeat = false;
      if (snapshot) {
        viewer.revision = BigInt(snapshot.state.revision);
        const bytes = frame(snapshot, ++viewer.serial);
        await viewer.writer.write(bytes);
        this.metrics.frames++;
        this.metrics.bytes += bytes.byteLength;
      } else {
        await viewer.writer.write(encoder.encode(': heartbeat\n\n'));
        this.metrics.heartbeats++;
      }
    }
  }
  publish(snapshot: Snapshot): void {
    if (this.latest && BigInt(snapshot.state.revision) < BigInt(this.latest.state.revision)) return;
    if (this.latest && snapshot.state.revision === this.latest.state.revision
      && (snapshot.order ?? 0) < (this.latest.order ?? 0)) return;
    this.latest = snapshot;
    for (const viewer of this.viewers) this.deliver(viewer, snapshot);
  }
  remove(viewer: Viewer): void {
    if (viewer.closed) return;
    viewer.closed = true;
    viewer.pending = undefined;
    viewer.wake?.();
    this.viewers.delete(viewer);
    void viewer.writer.abort().catch(() => {});
    if (!this.size) {
      if (this.heartbeat) clearInterval(this.heartbeat);
      if (this.clock) clearInterval(this.clock);
      this.heartbeat = undefined; this.clock = undefined;
    }
  }
}

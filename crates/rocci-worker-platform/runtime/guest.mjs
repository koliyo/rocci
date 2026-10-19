import { Admission } from "./admission.mjs";
import { ObjectSql } from "./sql.mjs";
import { Transactions } from "./transactions.mjs";

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });
export class Guest {
  #resources = new Map();
  #nextResource = 1;
  #exports;
  #gate;
  #signal;
  #source;
  #streamSources = new Map();
  constructor(
    module,
    {
      env = {},
      sql,
      storage,
      aliases = [],
      maxQueued = 32,
      outboundFetch = (request) => globalThis.fetch(request),
      sources = {},
    } = {},
  ) {
    this.env = env;
    this.sources = sources;
    this.routeSources = new Map(
      (sources.routes ?? []).map((route) => [
        `${route.method} ${route.path}`,
        route.location,
      ]),
    );
    this.outboundFetch = outboundFetch;
    this.sql = new ObjectSql(sql, aliases);
    this.transactions = new Transactions(storage);
    this.#gate = new Admission({ maxQueued });
    const exchange = (op, input, length, output, capacity) => {
      if (this.#gate.closed) throw new Error("Stale Worker continuation");
      const value = JSON.parse(
        decoder.decode(
          new Uint8Array(this.#exports.memory.buffer, input, length),
        ),
      );
      const write = (reply) => {
        const bytes = encoder.encode(JSON.stringify(reply));
        if (bytes.length > capacity)
          throw new Error("Host reply exceeds ABI capacity");
        new Uint8Array(this.#exports.memory.buffer).set(bytes, output);
        return bytes.length;
      };
      try {
        const reply = this.#effect(op, value);
        return reply instanceof Promise
          ? reply.then(write, (error) =>
              write({ error: String(error), code: error.code ?? 1 }),
            )
          : write(reply);
      } catch (error) {
        return write({ error: String(error), code: error.code ?? 1 });
      }
    };
    this.#exports = new WebAssembly.Instance(module, {
      rocci: {
        env: exchange,
        log: exchange,
        sql: exchange,
        clock: exchange,
        sleep: new WebAssembly.Suspending(exchange),
        transaction: new WebAssembly.Suspending(exchange),
        body: new WebAssembly.Suspending(exchange),
        http: new WebAssembly.Suspending(exchange),
        resource_drop: new WebAssembly.Suspending((id) => {
          const resource = this.#resources.get(Number(id));
          this.#resources.delete(Number(id));
          if (resource?.kind === "transaction")
            return this.transactions.drop(resource);
        }),
      },
    }).exports;
    if (this.#exports.worker_abi_version() !== 1)
      throw new Error("Worker host ABI mismatch");
  }
  get closed() {
    return this.#gate.closed;
  }
  get settled() {
    // The Wasm stack can settle before the provider has finished rolling back.
    // Keep reconstruction fenced until both owners have released the scope.
    return this.#gate.settled && !this.transactions.active;
  }
  discard(reason) {
    this.#gate.close(reason);
  }
  finishBody(id) {
    const body = this.#resources.get(id);
    if (body?.kind === "body") {
      if (!body.admitted) {
        this.#resources.delete(id);
        return;
      }
      body.finished = true;
      body.request = undefined;
    }
  }
  #capacity() {
    if (this.#resources.size >= 1024)
      throw Object.assign(new Error("Worker resource capacity exhausted"), {
        code: -4,
      });
    if (this.#nextResource >= 0x1_0000_0000)
      throw Object.assign(new Error("Worker resource ID exhausted"), {
        code: -4,
      });
  }
  add(resource) {
    this.#capacity();
    const id = this.#nextResource++;
    this.#resources.set(id, resource);
    return id;
  }
  #get(id, kind) {
    const resource = this.#resources.get(id);
    if (!resource || resource.kind !== kind)
      throw new Error(`Invalid or stale ${kind} resource`);
    return resource;
  }
  #effect(op, value) {
    switch (op) {
      case 1:
        return typeof this.env[value.name] === "string"
          ? { value: this.env[value.name] }
          : {};
      case 2:
        console.log(
          (this.#source ? this.#source + ": " : "") +
            String(value.message).slice(0, 8192),
        );
        return {};
      case 3: {
        if (this.transactions.active)
          throw new Error("Timers are unsupported inside an open transaction");
        if (
          !Number.isSafeInteger(value.milliseconds) ||
          value.milliseconds < 0 ||
          value.milliseconds > 30_000
        )
          throw new Error("Timer exceeds Worker limit");
        return new Promise((resolve, reject) => {
          const signal = this.#signal;
          const finish = () => {
            signal?.removeEventListener("abort", cancel);
            resolve({});
          };
          const timer = setTimeout(finish, value.milliseconds);
          const cancel = () => {
            clearTimeout(timer);
            reject(signal.reason ?? new Error("Cancelled"));
          };
          if (signal?.aborted) cancel();
          else signal?.addEventListener("abort", cancel, { once: true });
        });
      }
      case 4:
        return { milliseconds: Date.now() };
      case 10:
        return { id: this.add(this.sql.open(value.path, value.options)) };
      case 11:
        this.#get(value.db, "db");
        return { id: this.add(this.sql.prepare(value.query)) };
      case 12:
        return { columns: this.#get(value.statement, "statement").columns };
      case 13: {
        this.#capacity();
        const statement = this.#get(value.statement, "statement");
        if (statement.transaction?.finished)
          throw new Error("Transaction already finished");
        if (
          this.transactions.active &&
          statement.transaction !== this.transactions.active
        )
          throw new Error(
            "Use transaction-scoped statements while a transaction is open",
          );
        if (value.timeout !== 5000)
          throw new Error(
            "Worker SQL maps the default query timeout to provider limits; custom native query timeouts are unsupported",
          );
        return { id: this.add(this.sql.execute(statement, value.params)) };
      }
      case 14: {
        const execution = this.#get(value.execution, "execution");
        if (execution.index >= execution.rows.length) return { state: "done" };
        if (!value.allowRow) return { state: "rows" };
        const row = execution.rows[execution.index];
        if (row.bytes > value.maxBytes) return { state: "bytes" };
        execution.index++;
        return { state: "row", ...row };
      }
      case 20: {
        this.#capacity();
        this.#get(value.db, "db");
        return this.transactions.begin(value.mode).then(async (transaction) => {
          try {
            return { id: this.add(transaction) };
          } catch (error) {
            await this.transactions.drop(transaction);
            throw error;
          }
        });
      }
      case 21: {
        const transaction = this.#get(value.transaction, "transaction");
        if (transaction.finished)
          throw new Error("Transaction already finished");
        return {
          id: this.add({ ...this.sql.prepare(value.query), transaction }),
        };
      }
      case 22:
        return this.transactions
          .finish(this.#get(value.transaction, "transaction"), value.commit)
          .then(() => ({ ok: true }));
      case 30:
        return this.#readBody(value);
      case 31:
        return this.#fetch(value);
      default:
        throw new Error(`Unsupported Worker effect ${op}`);
    }
  }
  async #readBody(value) {
    if (this.transactions.active)
      return { error: "Body reads are unsupported inside an open transaction" };
    const body = this.#get(value.body, "body");
    if (body.finished) return { kind: "finished" };
    if (
      !Number.isSafeInteger(value.limit) ||
      value.limit < 0 ||
      value.limit > 1024 * 1024
    )
      return { error: "Body limit exceeds Worker profile" };
    body.finished = true;
    if (!body.request.body) return { bytes: [] };
    const reader = body.request.body.getReader();
    const signal = this.#signal;
    const cancel = () => {
      reader.cancel().catch(() => {});
    };
    signal?.addEventListener("abort", cancel, { once: true });
    const chunks = [];
    let received = 0;
    try {
      while (true) {
        const { value: chunk, done } = await reader.read();
        if (signal?.aborted) return { kind: "cancelled" };
        if (done) break;
        received += chunk.length;
        if (received > value.limit) {
          await reader.cancel();
          return { kind: "too-large", received };
        }
        chunks.push(chunk);
      }
      const bytes = new Uint8Array(received);
      let offset = 0;
      for (const chunk of chunks) {
        bytes.set(chunk, offset);
        offset += chunk.length;
      }
      return { bytes: [...bytes] };
    } catch (error) {
      return {
        kind: body.request.signal.aborted ? "cancelled" : "invalid",
        error: String(error),
      };
    } finally {
      signal?.removeEventListener("abort", cancel);
      reader.releaseLock();
    }
  }
  async #fetch(value) {
    if (this.transactions.active)
      return {
        error: "Outbound fetch is unsupported inside an open transaction",
      };
    if (
      !Number.isSafeInteger(value.limit) ||
      value.limit < 0 ||
      value.limit > 256 * 1024 ||
      !Number.isSafeInteger(value.timeout) ||
      value.timeout < 1 ||
      value.timeout > 30_000
    )
      return {
        error: "Outbound fetch limit or timeout exceeds Worker profile",
      };
    const timeout = new AbortController();
    const timer = setTimeout(
      () => timeout.abort(new Error("timeout")),
      value.timeout,
    );
    const signal = AbortSignal.any(
      [this.#signal, timeout.signal].filter(Boolean),
    );
    const method = [
      "CONNECT",
      "DELETE",
      value.methodExt,
      "GET",
      "HEAD",
      "OPTIONS",
      "PATCH",
      "POST",
      "PUT",
      "TRACE",
      "QUERY",
    ][value.method];
    try {
      const response = await this.outboundFetch(
        new Request(value.uri, {
          method,
          headers: value.headers,
          body: value.body.length ? new Uint8Array(value.body) : undefined,
          redirect: "manual",
          signal,
        }),
      );
      const chunks = [];
      let received = 0;
      if (response.body) {
        const reader = response.body.getReader();
        try {
          while (true) {
            const { value: chunk, done } = await reader.read();
            if (done) break;
            received += chunk.length;
            if (received > value.limit) {
              await reader.cancel();
              return { kind: "too-large", limit: value.limit, received };
            }
            chunks.push(chunk);
          }
        } finally {
          reader.releaseLock();
        }
      }
      const bytes = new Uint8Array(received);
      let offset = 0;
      for (const chunk of chunks) {
        bytes.set(chunk, offset);
        offset += chunk.length;
      }
      return {
        status: response.status,
        headers: [...response.headers],
        bytes: [...bytes],
      };
    } catch (error) {
      return {
        kind: timeout.signal.aborted
          ? "timeout"
          : signal.aborted
            ? "cancelled"
            : "transport",
        error: String(error),
      };
    } finally {
      clearTimeout(timer);
    }
  }
  async invoke(operation, value = {}, options = {}) {
    const method = [
      "CONNECT",
      "DELETE",
      value.methodExt,
      "GET",
      "HEAD",
      "OPTIONS",
      "PATCH",
      "POST",
      "PUT",
      "TRACE",
      "QUERY",
    ][value.method];
    const source =
      operation === 0
        ? (this.sources.init ?? this.sources.fallback)
        : operation === 1
          ? (this.routeSources.get(`${method} ${value.path}`) ??
            (method === "HEAD"
              ? this.routeSources.get(`GET ${value.path}`)
              : undefined) ??
            this.sources.fallback)
          : (this.#streamSources.get(value.id) ?? this.sources.fallback);
    let admitted = false;
    const bytes = encoder.encode(JSON.stringify(value));
    if (bytes.length > this.#exports.worker_input_capacity())
      throw new Error("Request exceeds Worker ABI capacity");
    try {
      return await this.#gate.run(async ({ signal }) => {
        admitted = true;
        this.#source = source;
        this.#signal = signal;
        if (operation === 1) {
          const body = this.#resources.get(value.body);
          if (body?.kind === "body") body.admitted = true;
        }
        new Uint8Array(this.#exports.memory.buffer).set(
          bytes,
          this.#exports.worker_input_ptr(),
        );
        await WebAssembly.promising(this.#exports.worker_invoke)(
          operation,
          bytes.length,
        );
        const output = JSON.parse(
          decoder.decode(
            new Uint8Array(
              this.#exports.memory.buffer,
              this.#exports.worker_output_ptr(),
              this.#exports.worker_output_len(),
            ),
          ),
        );
        if (output.error) throw new Error(output.error);
        if (operation === 0) this.limits = output.limits;
        if (operation === 1 && output.kind === "stream")
          this.#streamSources.set(output.id, source);
        if (operation === 3 || (operation === 2 && output.kind === "end"))
          this.#streamSources.delete(value.id);
        if (operation === 4) this.#streamSources.clear();
        return output;
      }, options);
    } catch (error) {
      if (this.#gate.closed) {
        if (this.transactions.active)
          await this.transactions
            .drop(this.transactions.active)
            .catch(() => {});
        this.#resources.clear();
      }
      if (admitted && source)
        throw new Error(`${source}: ${String(error)}`, { cause: error });
      throw error;
    }
  }
  metrics() {
    return this.#gate.run(() => ({
      liveBytes: this.#exports.worker_live_bytes(),
      liveAllocations: this.#exports.worker_live_allocations(),
      memoryBytes: this.#exports.memory.buffer.byteLength,
      resources: this.#resources.size,
    }));
  }
}

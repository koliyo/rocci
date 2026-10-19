/** One suspended invocation owns the instance until it settles or is discarded. */
export class Admission {
  #queue = [];
  #active = false;
  #generation = 0;
  #closed = false;

  constructor({ maxQueued = 32 } = {}) {
    if (!Number.isSafeInteger(maxQueued) || maxQueued < 0 || maxQueued > 1024) {
      throw new RangeError("maxQueued must be an integer between 0 and 1024");
    }
    this.maxQueued = maxQueued;
  }

  get pending() {
    return this.#queue.length;
  }
  get generation() {
    return this.#generation;
  }
  get closed() {
    return this.#closed;
  }
  get settled() {
    return !this.#active;
  }

  close(reason = new Error("Worker instance discarded")) {
    if (this.#closed) return;
    this.#closed = true;
    this.#generation++;
    for (const entry of this.#queue.splice(0)) entry.reject(reason);
  }

  run(operation, { signal, timeoutMs = 30_000 } = {}) {
    if (
      !Number.isSafeInteger(timeoutMs) ||
      timeoutMs <= 0 ||
      timeoutMs > 300_000
    ) {
      return Promise.reject(
        new RangeError("timeoutMs must be between 1 and 300000"),
      );
    }
    if (this.#closed)
      return Promise.reject(new Error("Worker instance discarded"));
    if (signal?.aborted)
      return Promise.reject(signal.reason ?? new Error("Cancelled"));
    if (this.#active && this.#queue.length >= this.maxQueued) {
      return Promise.reject(new Error("Worker invocation queue saturated"));
    }
    return new Promise((resolve, reject) => {
      const entry = { operation, signal, timeoutMs, resolve, reject };
      if (this.#active) this.#queue.push(entry);
      else this.#start(entry);
    });
  }

  async #start(entry) {
    this.#active = true;
    const { operation, signal, timeoutMs, resolve, reject } = entry;
    const generation = this.#generation;
    const controller = new AbortController();
    const cancel = (reason) => {
      controller.abort(reason);
      this.close(reason);
      reject(reason);
    };
    const onAbort = () => cancel(signal.reason ?? new Error("Cancelled"));
    const timer = setTimeout(
      () => cancel(new Error("Worker invocation timed out")),
      timeoutMs,
    );
    signal?.addEventListener("abort", onAbort, { once: true });
    try {
      if (signal?.aborted) throw signal.reason ?? new Error("Cancelled");
      const result = await operation({ signal: controller.signal, generation });
      if (generation !== this.#generation)
        throw new Error("Stale Worker continuation");
      resolve(result);
    } catch (error) {
      // A trap can leave reference counts, resources and the spill stack suspect.
      this.close(error);
      reject(error);
    } finally {
      clearTimeout(timer);
      signal?.removeEventListener("abort", onAbort);
      this.#active = false;
      if (!this.#closed) {
        let next;
        while ((next = this.#queue.shift())) {
          if (next.signal?.aborted)
            next.reject(next.signal.reason ?? new Error("Cancelled"));
          else {
            this.#start(next);
            break;
          }
        }
      }
    }
  }
}

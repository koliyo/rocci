import { Admission } from "./admission.mjs";

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true });
const prefix = "[retained Roc context → 🌍 with a heap string] ";

function check(condition, message) {
  if (!condition) throw new Error(message);
}

/** Runs actual Roc code; portable between Node, workerd and hosted Workers. */
export async function qualify(
  module,
  { iterations = 10_000, timer = true } = {},
) {
  check(
    typeof WebAssembly.Suspending === "function" &&
      typeof WebAssembly.promising === "function",
    "JSPI is unavailable",
  );
  const drops = [];
  let rejectEffect = false,
    suspensions = 0;
  const gate = new Admission();
  const instance = new WebAssembly.Instance(module, {
    rocci: {
      delay: new WebAssembly.Suspending(async (value) => {
        suspensions++;
        if (timer) await new Promise((resolve) => setTimeout(resolve, 0));
        else await Promise.resolve();
        return rejectEffect ? -1n : value + 1n;
      }),
      resource_drop: (id) => drops.push(Number(id)),
    },
  });
  const e = instance.exports;
  check(e.worker_abi_version() === 1, "Worker ABI version mismatch");
  await gate.run(() => WebAssembly.promising(e.worker_init)());
  const invoke = WebAssembly.promising(e.worker_invoke);
  const memoryBefore = e.memory.buffer.byteLength;
  const warm = new Map();
  const inputs = ["", "a", "Unicode → 🌍\0end", "x".repeat(1024)];
  async function call(input) {
    const bytes = encoder.encode(input);
    check(bytes.length <= e.input_capacity(), "Input exceeds capacity");
    return gate.run(async () => {
      // Recreate views for every access; an import can grow memory while suspended.
      new Uint8Array(e.memory.buffer).set(bytes, e.input_ptr());
      await invoke(bytes.length);
      return decoder.decode(
        new Uint8Array(e.memory.buffer, e.output_ptr(), e.output_len()),
      );
    });
  }
  for (let n = 0; n < iterations; n++) {
    const input = inputs[n % inputs.length];
    rejectEffect = n % 7 === 0;
    const output = await call(input);
    check(
      output ===
        (rejectEffect
          ? `rejected:${prefix}${input}`
          : `${prefix}${input}:42:19:73`),
      `Roc result mismatch at invocation ${n}: ${output}`,
    );
    const key = `${n % inputs.length}:${rejectEffect}`;
    const allocation = `${e.live_bytes()}:${e.live_allocations()}`;
    if (warm.has(key))
      check(
        allocation === warm.get(key),
        `Guest allocation growth at ${n}: ${allocation}, expected ${warm.get(key)}`,
      );
    else warm.set(key, allocation);
  }
  check(
    drops.length === iterations && drops.every((id) => id === 19),
    "Transient resource finalizer mismatch",
  );
  check(
    e.memory.buffer.byteLength === memoryBefore,
    "Linear memory grew during repeated invocations",
  );
  const highWater = e.heap_high_water();
  await gate.run(() => WebAssembly.promising(e.worker_close)());
  check(
    e.live_bytes() === 0 && e.live_allocations() === 0,
    "Guest allocations remain after closing context",
  );
  check(
    drops.length === iterations + 1 && drops.at(-1) === 73,
    "Retained resource was not finalized once",
  );
  return {
    iterations,
    suspensions,
    finalizers: drops.length,
    memoryBefore,
    memoryAfter: e.memory.buffer.byteLength,
    highWater,
    liveBytesAfterClose: e.live_bytes(),
  };
}

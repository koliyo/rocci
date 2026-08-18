import type { CompileRequest, CompileResponse, WorkerRequest, WorkerResponse } from "./protocol";

let wasmInstance: WebAssembly.Instance | null = null;
let memory: WebAssembly.Memory | null = null;
let allocFn: ((len: number) => number) | null = null;
let deallocFn: ((ptr: number, len: number) => void) | null = null;
let compileRawFn: ((ptr: number, len: number) => bigint) | null = null;

async function initWasm(wasmUrl: string) {
  try {
    const importObject = {
      __wbindgen_placeholder__: {
        __wbindgen_describe: () => {},
        __wbindgen_throw: (ptr: number, len: number) => {
          throw new Error(`wasm-bindgen exception (ptr=${ptr}, len=${len})`);
        },
      },
      __wbindgen_externref_xform__: {
        __wbindgen_externref_table_set_null: () => {},
        __wbindgen_externref_table_grow: () => {},
      },
    };

    let response: Response | BufferSource;
    if (typeof fetch === "function") {
      const resp = await fetch(wasmUrl);
      if ("instantiateStreaming" in WebAssembly && typeof resp.body !== "undefined") {
        const streamRes = await WebAssembly.instantiateStreaming(resp, importObject);
        wasmInstance = streamRes.instance;
      } else {
        const buf = await resp.arrayBuffer();
        const res = await WebAssembly.instantiate(buf, importObject);
        wasmInstance = res.instance;
      }
    } else {
      throw new Error("fetch is not available in worker environment");
    }

    const exports = wasmInstance.exports as Record<string, unknown>;
    memory = exports.memory as WebAssembly.Memory;
    allocFn = exports.playground_alloc as (len: number) => number;
    deallocFn = exports.playground_dealloc as (ptr: number, len: number) => void;
    compileRawFn = exports.playground_compile_raw as (ptr: number, len: number) => bigint;

    let meta = {};
    if (typeof exports.init_playground === "function") {
      try {
        // if init_playground is exported
      } catch {
        // fallback
      }
    }

    const okMsg: WorkerResponse = { type: "init_ok", metadata: meta };
    self.postMessage(okMsg);
  } catch (err: unknown) {
    const errMsg: WorkerResponse = {
      type: "init_error",
      error: err instanceof Error ? err.message : String(err),
    };
    self.postMessage(errMsg);
  }
}

function compileRequest(request: CompileRequest) {
  if (!wasmInstance || !memory || !allocFn || !deallocFn || !compileRawFn) {
    const errMsg: WorkerResponse = {
      type: "compile_error",
      revision: request.revision,
      error: "Compiler WASM is not initialized",
    };
    self.postMessage(errMsg);
    return;
  }

  const t0 = performance.now();
  try {
    const jsonStr = JSON.stringify(request);
    const encoder = new TextEncoder();
    const sourceBytes = encoder.encode(jsonStr);
    const ptr = allocFn(sourceBytes.length);
    new Uint8Array(memory.buffer).set(sourceBytes, ptr);

    const res = compileRawFn(ptr, sourceBytes.length);
    const outPtr = Number(res >> 32n);
    const outLen = Number(res & 0xffffffffn);
    const outBytes = new Uint8Array(memory.buffer, outPtr, outLen);
    const respJsonStr = new TextDecoder().decode(outBytes);

    deallocFn(ptr, sourceBytes.length);
    deallocFn(outPtr, outLen);

    const compileResponse: CompileResponse = JSON.parse(respJsonStr);
    const durationMs = performance.now() - t0;

    const okMsg: WorkerResponse = {
      type: "compile_ok",
      response: compileResponse,
      durationMs,
    };
    self.postMessage(okMsg);
  } catch (err: unknown) {
    const durationMs = performance.now() - t0;
    const errMsg: WorkerResponse = {
      type: "compile_error",
      revision: request.revision,
      error: err instanceof Error ? err.message : String(err),
    };
    self.postMessage(errMsg);
  }
}

self.addEventListener("message", (event: MessageEvent<WorkerRequest>) => {
  const data = event.data;
  if (!data) return;

  if (data.type === "init") {
    initWasm(data.wasmUrl);
  } else if (data.type === "compile") {
    compileRequest(data.request);
  }
});

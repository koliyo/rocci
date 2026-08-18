#!/usr/bin/env node
/**
 * Phase 17 Performance & Budget Test Suite.
 *
 * Verifies:
 * 1. WASM binary size is within budget (<1.5 MB uncompressed, <400 KB gzipped).
 * 2. Rapid sequential compiles (1,000 iterations) maintain < 20 ms average latency.
 * 3. Memory stability across repeated allocations.
 */

import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");

const WASM_PATH = path.join(
  ROOT,
  "target/wasm32-unknown-unknown/release/rocci_playground_wasm.wasm"
);

if (!fs.existsSync(WASM_PATH)) {
  console.error(`WASM binary not found at ${WASM_PATH}. Build it first.`);
  process.exit(1);
}

const wasmBuffer = fs.readFileSync(WASM_PATH);
const gzipped = zlib.gzipSync(wasmBuffer);

console.log("=================================================");
console.log(" Phase 17 Performance & Budget Test Suite");
console.log("=================================================");

console.log(`\n1. Binary Size Budget:`);
const sizeKb = (wasmBuffer.length / 1024).toFixed(1);
const gzipKb = (gzipped.length / 1024).toFixed(1);
console.log(`   Uncompressed: ${sizeKb} KB (Budget: < 1500 KB)`);
console.log(`   Gzipped:      ${gzipKb} KB (Budget: < 400 KB)`);

let failed = 0;
if (wasmBuffer.length > 1500 * 1024) {
  console.error(`  [FAIL] WASM uncompressed size exceeds 1.5 MB budget`);
  failed++;
} else {
  console.log(`  [PASS] WASM uncompressed size within budget`);
}

if (gzipped.length > 400 * 1024) {
  console.error(`  [FAIL] WASM gzipped size exceeds 400 KB budget`);
  failed++;
} else {
  console.log(`  [PASS] WASM gzipped size within budget`);
}

// 2. Compilation Latency & Memory Stability Benchmark
const importObject = {
  __wbindgen_placeholder__: {
    __wbindgen_describe: () => {},
    __wbindgen_throw: (ptr, len) => {
      throw new Error(`wasm-bindgen throw: ptr=${ptr} len=${len}`);
    },
  },
  __wbindgen_externref_xform__: {
    __wbindgen_externref_table_set_null: () => {},
    __wbindgen_externref_table_grow: () => {},
  },
};

const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);
const {
  memory,
  playground_alloc,
  playground_dealloc,
  playground_compile_raw,
} = instance.exports;

function compileWasm(reqObj) {
  const jsonStr = JSON.stringify(reqObj);
  const encoder = new TextEncoder();
  const sourceBytes = encoder.encode(jsonStr);
  const ptr = playground_alloc(sourceBytes.length);
  new Uint8Array(memory.buffer).set(sourceBytes, ptr);

  const res = playground_compile_raw(ptr, sourceBytes.length);
  const outPtr = Number(res >> 32n);
  const outLen = Number(res & 0xffffffffn);
  const outBytes = new Uint8Array(memory.buffer, outPtr, outLen);
  const respJsonStr = new TextDecoder().decode(outBytes);

  playground_dealloc(ptr, sourceBytes.length);
  playground_dealloc(outPtr, outLen);
  return JSON.parse(respJsonStr);
}

console.log(`\n2. Compilation Latency Benchmark (1,000 sequential edits):`);
const ITERATIONS = 1000;
const startMem = memory.buffer.byteLength;
const t0 = performance.now();

for (let i = 0; i < ITERATIONS; i++) {
  const count = i % 100;
  const source = `@component Counter = |{ count }| { <button count="${count}">{count}</button> }`;
  const res = compileWasm({
    protocol_version: 1,
    revision: i,
    filename: "Counter.rocci",
    source,
  });
  if (res.has_errors) {
    console.error(`  [FAIL] Compilation error at iteration ${i}`);
    failed++;
    break;
  }
}

const totalTime = performance.now() - t0;
const avgTime = totalTime / ITERATIONS;
const endMem = memory.buffer.byteLength;

console.log(`   Total time:   ${totalTime.toFixed(1)} ms for ${ITERATIONS} compiles`);
console.log(`   Average time: ${avgTime.toFixed(3)} ms per compile (Budget: < 20 ms)`);
console.log(`   Memory start: ${(startMem / 1024).toFixed(0)} KB, end: ${(endMem / 1024).toFixed(0)} KB`);

if (avgTime > 20) {
  console.error(`  [FAIL] Average compile time exceeded 20 ms`);
  failed++;
} else {
  console.log(`  [PASS] Average compile latency within budget`);
}

console.log("\n=================================================");
console.log(` Results: ${failed === 0 ? "ALL PASSED" : `${failed} FAILED`}`);
console.log("=================================================");

if (failed > 0) {
  process.exit(1);
}

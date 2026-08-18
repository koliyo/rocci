#!/usr/bin/env node
/**
 * Phase 2 WASM verification harness for Rocci & Rocdown Playground.
 *
 * Loads the compiled rocci_playground_wasm.wasm binary, verifies JSON protocol
 * compilation, initialization metadata, capabilities, UTF-16 diagnostic offsets,
 * and error resilience.
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
  console.error(`WASM binary not found at ${WASM_PATH}. Run 'cargo build -p rocci-playground-wasm --target wasm32-unknown-unknown --release' first.`);
  process.exit(1);
}

const wasmBuffer = fs.readFileSync(WASM_PATH);
const rawSize = wasmBuffer.length;
const gzipSize = zlib.gzipSync(wasmBuffer).length;

console.log("=================================================");
console.log(" Phase 2 Playground WASM Verification Harness");
console.log("=================================================");
console.log(`Binary size (raw release):   ${(rawSize / 1024).toFixed(1)} KB (${rawSize} bytes)`);
console.log(`Binary size (gzipped):       ${(gzipSize / 1024).toFixed(1)} KB (${gzipSize} bytes)`);
console.log("-------------------------------------------------");

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

function runCompileJson(reqObj) {
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

let passed = 0;
let failed = 0;

function check(label, condition, details = "") {
  if (condition) {
    console.log(`  [PASS] ${label}`);
    passed++;
  } else {
    console.error(`  [FAIL] ${label} - ${details}`);
    failed++;
  }
}

// 1. Test Counter.rocci
console.log("\n1. Testing Counter.rocci via playground request:");
const counterSource = fs.readFileSync(path.join(ROOT, "examples/counter/Counter.rocci"), "utf-8");
const t0 = performance.now();
const counterResult = runCompileJson({
  protocol_version: 1,
  revision: 1,
  filename: "Counter.rocci",
  source: counterSource,
});
const counterDuration = performance.now() - t0;
console.log(`   Compiled in ${counterDuration.toFixed(2)} ms`);

check("protocol_version is 1", counterResult.protocol_version === 1);
check("language is rocci", counterResult.language === "rocci");
check("no compilation errors", !counterResult.has_errors);
check("generates counterCard Roc function", counterResult.roc.includes("counterCard ="));
check("generates counterPage Roc function", counterResult.roc.includes("counterPage ="));
check("formats LISP AST", counterResult.ast.includes("(component CounterCard"));
check("roc capability is available", counterResult.capabilities?.roc?.available === true);
check("ast capability is available", counterResult.capabilities?.ast?.available === true);
check("html capability is unavailable with reason", counterResult.capabilities?.html?.available === false && counterResult.capabilities?.html?.reason?.includes("HTML preview is not available yet"));

// 2. Test Guide.rocdown
console.log("\n2. Testing Guide.rocdown:");
const guideSource = fs.readFileSync(path.join(ROOT, "examples/rocdown/Guide.rocdown"), "utf-8");
const t1 = performance.now();
const guideResult = runCompileJson({
  protocol_version: 1,
  revision: 2,
  filename: "Guide.rocdown",
  source: guideSource,
});
const guideDuration = performance.now() - t1;
console.log(`   Compiled in ${guideDuration.toFixed(2)} ms`);

check("language is rocdown", guideResult.language === "rocdown");
check("no compilation errors", !guideResult.has_errors);
check("generates Rocdom HTML elements", guideResult.roc.includes("Html."));
check("formats Rocdown AST", guideResult.ast.includes("(rocdown"));

// 3. Test Non-BMP Unicode & Diagnostic Range
console.log("\n3. Testing Unicode emoji and UTF-16 diagnostic offsets:");
const unicodeBrokenSource = "Party 🎉 rock\n@component Incomplete = { <div ";
const brokenResult = runCompileJson({
  protocol_version: 1,
  revision: 3,
  filename: "Test.rocci",
  source: unicodeBrokenSource,
});

check("reports errors flag", brokenResult.has_errors === true);
check("emits structured diagnostics", brokenResult.diagnostics.length > 0);
const firstDiag = brokenResult.diagnostics[0];
check("diagnostic has UTF-16 from/to coordinates", typeof firstDiag?.from === "number" && typeof firstDiag?.to === "number");
check("UTF-16 coordinates are ordered and in range", firstDiag.from <= firstDiag.to && firstDiag.to <= 100);

// 4. Test Invalid JSON resilience
console.log("\n4. Testing invalid JSON error resilience:");
const rawInvalidPtr = playground_alloc(5);
new Uint8Array(memory.buffer).set(new TextEncoder().encode("hello"), rawInvalidPtr);
const rawRes = playground_compile_raw(rawInvalidPtr, 5);
const outPtr = Number(rawRes >> 32n);
const outLen = Number(rawRes & 0xffffffffn);
const outBytes = new Uint8Array(memory.buffer, outPtr, outLen);
const errResp = JSON.parse(new TextDecoder().decode(outBytes));
playground_dealloc(rawInvalidPtr, 5);
playground_dealloc(outPtr, outLen);

check("returns structured error response without crashing", errResp.has_errors === true && Boolean(errResp.error));

console.log("\n=================================================");
console.log(` Results: ${passed} passed, ${failed} failed`);
console.log("=================================================");

if (failed > 0) {
  process.exit(1);
}

#!/usr/bin/env node
/**
 * Phase 0 WASM verification harness for Rocci & Rocdown.
 *
 * Loads the compiled Rust WASM binary and executes single-file compilation
 * of representative .rocci and .rocdown fixtures, validating generated Roc,
 * formatted AST, diagnostics, and size metrics.
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
  "target/wasm32-unknown-unknown/release/rocci_playground_spike.wasm"
);

if (!fs.existsSync(WASM_PATH)) {
  console.error(`WASM binary not found at ${WASM_PATH}. Run 'cargo build -p rocci-playground-spike --target wasm32-unknown-unknown --release' first.`);
  process.exit(1);
}

const wasmBuffer = fs.readFileSync(WASM_PATH);
const rawSize = wasmBuffer.length;
const gzipSize = zlib.gzipSync(wasmBuffer).length;

console.log("=================================================");
console.log(" Phase 0 WASM Verification Harness");
console.log("=================================================");
console.log(`Binary size (raw release):   ${(rawSize / 1024).toFixed(1)} KB (${rawSize} bytes)`);
console.log(`Binary size (gzipped):       ${(gzipSize / 1024).toFixed(1)} KB (${gzipSize} bytes)`);
console.log("-------------------------------------------------");

const importObject = {
  __wbindgen_placeholder__: {
    __wbindgen_describe: () => {},
  },
  __wbindgen_externref_xform__: {
    __wbindgen_externref_table_set_null: () => {},
    __wbindgen_externref_table_grow: () => {},
  },
};

const { instance } = await WebAssembly.instantiate(wasmBuffer, importObject);
const {
  memory,
  spike_alloc,
  spike_dealloc,
  compile_rocci_raw,
  compile_rocdown_raw,
} = instance.exports;

function runCompile(fn, source) {
  const encoder = new TextEncoder();
  const sourceBytes = encoder.encode(source);
  const ptr = spike_alloc(sourceBytes.length);
  new Uint8Array(memory.buffer).set(sourceBytes, ptr);

  const res = fn(ptr, sourceBytes.length);
  const outPtr = Number(res >> 32n);
  const outLen = Number(res & 0xffffffffn);
  const outBytes = new Uint8Array(memory.buffer, outPtr, outLen);
  const jsonStr = new TextDecoder().decode(outBytes);

  spike_dealloc(ptr, sourceBytes.length);
  spike_dealloc(outPtr, outLen);
  return JSON.parse(jsonStr);
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
console.log("\n1. Testing Counter.rocci:");
const counterSource = fs.readFileSync(path.join(ROOT, "examples/counter/Counter.rocci"), "utf-8");
const t0 = performance.now();
const counterResult = runCompile(compile_rocci_raw, counterSource);
const counterDuration = performance.now() - t0;
console.log(`   Compiled in ${counterDuration.toFixed(2)} ms`);

check("language is rocci", counterResult.language === "rocci");
check("no compilation errors", !counterResult.has_errors);
check("generates counterCard Roc function", counterResult.roc.includes("counterCard ="));
check("generates counterPage Roc function", counterResult.roc.includes("counterPage ="));
check("formats LISP AST", counterResult.ast.includes("(component CounterCard"));

// 2. Test AllSyntax.rocci
console.log("\n2. Testing test/AllSyntax.rocci:");
const allSyntaxRocciSource = fs.readFileSync(path.join(ROOT, "test/AllSyntax.rocci"), "utf-8");
const t1 = performance.now();
const allSyntaxRocciResult = runCompile(compile_rocci_raw, allSyntaxRocciSource);
const allSyntaxRocciDuration = performance.now() - t1;
console.log(`   Compiled in ${allSyntaxRocciDuration.toFixed(2)} ms`);

check("language is rocci", allSyntaxRocciResult.language === "rocci");
check("no compilation errors", !allSyntaxRocciResult.has_errors);
check("contains badge component in Roc", allSyntaxRocciResult.roc.includes("badge ="));
check("formats all-syntax AST", allSyntaxRocciResult.ast.includes("(component"));

// 3. Test Guide.rocdown
console.log("\n3. Testing Guide.rocdown:");
const guideSource = fs.readFileSync(path.join(ROOT, "examples/rocdown/Guide.rocdown"), "utf-8");
const t2 = performance.now();
const guideResult = runCompile(compile_rocdown_raw, guideSource);
const guideDuration = performance.now() - t2;
console.log(`   Compiled in ${guideDuration.toFixed(2)} ms`);

check("language is rocdown", guideResult.language === "rocdown");
check("no compilation errors", !guideResult.has_errors);
check("generates Rocdom HTML elements", guideResult.roc.includes("Html."));
check("formats Rocdown AST", guideResult.ast.includes("(rocdown"));

// 4. Test AllSyntax.rocdown
console.log("\n4. Testing test/AllSyntax.rocdown:");
const allSyntaxRocdownSource = fs.readFileSync(path.join(ROOT, "test/AllSyntax.rocdown"), "utf-8");
const t3 = performance.now();
const allSyntaxRocdownResult = runCompile(compile_rocdown_raw, allSyntaxRocdownSource);
const allSyntaxRocdownDuration = performance.now() - t3;
console.log(`   Compiled in ${allSyntaxRocdownDuration.toFixed(2)} ms`);

check("language is rocdown", allSyntaxRocdownResult.language === "rocdown");
check("no compilation errors", !allSyntaxRocdownResult.has_errors);
check("contains hello component in Roc", allSyntaxRocdownResult.roc.includes("hello ="));
check("generates Rocdom HTML in Roc", allSyntaxRocdownResult.roc.includes("Html."));
check("formats AST headings", allSyntaxRocdownResult.ast.includes("(h 1"));

// 5. Test Invalid Syntax / Diagnostics
console.log("\n5. Testing invalid syntax diagnostic reporting:");
const brokenSource = "@component Broken = |{ { <div> }";
const brokenResult = runCompile(compile_rocci_raw, brokenSource);

check("reports errors flag", brokenResult.has_errors === true);
check("emits structured diagnostics", brokenResult.diagnostics.length > 0);
check("diagnostic contains severity and range", Boolean(brokenResult.diagnostics[0]?.severity && typeof brokenResult.diagnostics[0]?.start === "number"));

console.log("\n=================================================");
console.log(` Results: ${passed} passed, ${failed} failed`);
console.log("=================================================");

if (failed > 0) {
  process.exit(1);
}

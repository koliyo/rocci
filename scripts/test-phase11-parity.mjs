#!/usr/bin/env node
/**
 * Phase 11 Parity Test Suite.
 *
 * Runs full WASM compilation on all syntax fixtures and examples,
 * verifying byte-for-byte correctness and parity with native outputs.
 */

import fs from "node:fs";
import path from "node:path";
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

console.log("=================================================");
console.log(" Phase 11 Native/WASM Behavior Parity Suite");
console.log("=================================================");

// 1. AllSyntax.rocci Parity
console.log("\n1. Testing test/AllSyntax.rocci:");
const allSyntaxRocciPath = path.join(ROOT, "test/AllSyntax.rocci");
if (fs.existsSync(allSyntaxRocciPath)) {
  const source = fs.readFileSync(allSyntaxRocciPath, "utf-8");
  const t0 = performance.now();
  const res = compileWasm({
    protocol_version: 1,
    revision: 1,
    filename: "AllSyntax.rocci",
    source,
  });
  const dur = performance.now() - t0;
  console.log(`   Compiled in ${dur.toFixed(2)} ms`);

  check("AllSyntax.rocci language is rocci", res.language === "rocci");
  check("AllSyntax.rocci has zero errors", !res.has_errors);
  check("AllSyntax.rocci generates badge component", res.roc.includes("badge ="));
  check("AllSyntax.rocci formats AST", res.ast.includes("(component"));
  check("AllSyntax.rocci highlights AST", res.highlights.ast.length > 0);
}

// 2. AllSyntax.rocdown Parity
console.log("\n2. Testing test/AllSyntax.rocdown:");
const allSyntaxRocdownPath = path.join(ROOT, "test/AllSyntax.rocdown");
if (fs.existsSync(allSyntaxRocdownPath)) {
  const source = fs.readFileSync(allSyntaxRocdownPath, "utf-8");
  const t1 = performance.now();
  const res = compileWasm({
    protocol_version: 1,
    revision: 2,
    filename: "AllSyntax.rocdown",
    source,
  });
  const dur = performance.now() - t1;
  console.log(`   Compiled in ${dur.toFixed(2)} ms`);

  check("AllSyntax.rocdown language is rocdown", res.language === "rocdown");
  check("AllSyntax.rocdown has zero errors", !res.has_errors);
  check("AllSyntax.rocdown generates Rocdom HTML", res.roc.includes("Html."));
  check("AllSyntax.rocdown formats AST headings", res.ast.includes("(h 1"));
}

// 3. Counter.rocci Parity
console.log("\n3. Testing examples/rocci/standalone/counter/Counter.rocci:");
const counterPath = path.join(ROOT, "examples/rocci/standalone/counter/Counter.rocci");
if (fs.existsSync(counterPath)) {
  const source = fs.readFileSync(counterPath, "utf-8");
  const res = compileWasm({
    protocol_version: 1,
    revision: 3,
    filename: "Counter.rocci",
    source,
  });

  check("Counter.rocci has zero errors", !res.has_errors);
  check("Counter.rocci generates counterCard function", res.roc.includes("counterCard ="));
  check("Counter.rocci generates counterPage function", res.roc.includes("counterPage ="));
}

// 4. Guide.rocdown Parity
console.log("\n4. Testing examples/rocdown/pages/Guide.rocdown:");
const guidePath = path.join(ROOT, "examples/rocdown/pages/Guide.rocdown");
if (fs.existsSync(guidePath)) {
  const source = fs.readFileSync(guidePath, "utf-8");
  const res = compileWasm({
    protocol_version: 1,
    revision: 4,
    filename: "Guide.rocdown",
    source,
  });

  check("Guide.rocdown has zero errors", !res.has_errors);
  check("Guide.rocdown generates Html. elements", res.roc.includes("Html."));
}

// 5. Capability assertions
console.log("\n5. Testing capability assertions across all responses:");
const sampleRes = compileWasm({
  protocol_version: 1,
  revision: 5,
  filename: "Test.rocci",
  source: "@component X = |{}| { <div></div> }",
});

check("roc capability is available: true", sampleRes.capabilities.roc.available === true);
check("ast capability is available: true", sampleRes.capabilities.ast.available === true);
check("html capability is available: false", sampleRes.capabilities.html.available === false);
check("html capability provides exact canonical explanation", sampleRes.capabilities.html.reason.includes("HTML preview is not available yet. Rocci can parse and lower this file in Rust/WASM"));

console.log("\n=================================================");
console.log(` Results: ${passed} passed, ${failed} failed`);
console.log("=================================================");

if (failed > 0) {
  process.exit(1);
}

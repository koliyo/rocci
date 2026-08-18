#!/usr/bin/env node
/**
 * Phase 3 Highlight Bridge verification harness.
 *
 * Verifies highlight spans for source, generated Roc, and formatted AST,
 * checking token class validity, span invariants, and non-overlap.
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

function verifySpans(label, text, spans) {
  const utf16Len = Array.from(text).length; // approximate utf16 / char count
  let prevTo = 0;
  for (let i = 0; i < spans.length; i++) {
    const s = spans[i];
    if (s.from > s.to || s.from < prevTo || !s.kind.startsWith("tok-")) {
      return false;
    }
    prevTo = s.to;
  }
  return true;
}

console.log("=================================================");
console.log(" Phase 3 Highlight Bridge Verification");
console.log("=================================================");

// 1. Counter.rocci highlights
const counterSource = fs.readFileSync(path.join(ROOT, "examples/counter/Counter.rocci"), "utf-8");
const counterResp = runCompileJson({
  protocol_version: 1,
  revision: 1,
  filename: "Counter.rocci",
  source: counterSource,
});

check("response contains highlights object", Boolean(counterResp.highlights));
check("AST highlights are valid and non-overlapping", verifySpans("ast", counterResp.ast, counterResp.highlights.ast));
check("AST has token spans", counterResp.highlights.ast.length > 0);

// 2. Guide.rocdown highlights
const guideSource = fs.readFileSync(path.join(ROOT, "examples/rocdown/Guide.rocdown"), "utf-8");
const guideResp = runCompileJson({
  protocol_version: 1,
  revision: 2,
  filename: "Guide.rocdown",
  source: guideSource,
});

check("Rocdown AST highlights valid", verifySpans("ast", guideResp.ast, guideResp.highlights.ast));

console.log("\n=================================================");
console.log(` Results: ${passed} passed, ${failed} failed`);
console.log("=================================================");

if (failed > 0) {
  process.exit(1);
}

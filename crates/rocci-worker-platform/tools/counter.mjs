import { mkdir, writeFile, copyFile } from "node:fs/promises";
import { serve } from "./serve.mjs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const pkg = fileURLToPath(new URL("../", import.meta.url));
const out = path.join(root, "target/worker-counter");
await mkdir(path.join(out, "runtime"), { recursive: true });
await mkdir(path.join(out, "public/assets"), { recursive: true });
await copyFile(
  path.join(out, "assets/rocci.css"),
  path.join(out, "public/assets/rocci.css"),
);
await copyFile(
  process.env.DATASTAR_JS ??
    path.join(process.env.HOME, ".rocci/cache/datastar/v1.0.2/datastar.js"),
  path.join(out, "public/assets/datastar.js"),
);
for (const file of ["admission", "sql", "guest", "cloudflare", "transactions"])
  await copyFile(
    path.join(pkg, `runtime/${file}.mjs`),
    path.join(out, `runtime/${file}.mjs`),
  );
await writeFile(
  path.join(out, "index.mjs"),
  `
import module from './app.wasm';
import { WorkerObject, objectWorker } from './runtime/cloudflare.mjs';
export class RocciApp extends WorkerObject {
  constructor(ctx, env) { super(module, ctx, env, {sqliteAliases:['./counter.db']}); }
}
export default objectWorker('APP', 'counter');
`,
);
await writeFile(
  path.join(out, "wrangler.json"),
  JSON.stringify(
    {
      name: "rocci-counter-conformance",
      main: "index.mjs",
      compatibility_date: "2026-10-19",
      workers_dev: false,
      preview_urls: false,
      assets: {
        directory: "public",
        binding: "ASSETS",
        run_worker_first: true,
      },
      durable_objects: { bindings: [{ name: "APP", class_name: "RocciApp" }] },
      migrations: [{ tag: "v1", new_sqlite_classes: ["RocciApp"] }],
    },
    null,
    2,
  ),
);
const withServer = (test) => serve(out, test, { port: 8794, ready: "/health" });
const origin = "http://127.0.0.1:8794";
const request = (url, init = {}) =>
  fetch(origin + url, { ...init, signal: AbortSignal.timeout(10000) });
await withServer(async () => {
  let response = await request("/actions/counter/reset", { method: "POST" });
  assert.equal(response.status, 200);
  assert.match(await response.text(), /<output[^>]*>0<\/output>/);
  response = await request("/");
  assert.equal(
    response.headers.get("content-type"),
    "text/html; charset=utf-8",
  );
  const before = await response.text();
  assert.match(before, /Welcome to Rocci/);
  response = await request("/actions/counter/increment", {
    method: "POST",
    headers: { "datastar-request": "true" },
  });
  assert.equal(response.headers.get("content-type"), "text/event-stream");
  const patch = await response.text();
  assert.match(patch, /event: datastar-patch-elements/);
  assert.match(patch, /<output[^>]*>1<\/output>/);
  assert.equal(patch.match(/event:/g).length, 1);
  response = await request("/");
  assert.match(await response.text(), /<output[^>]*>1<\/output>/);
  assert.equal((await request("/missing")).status, 404);
  assert.equal((await request("/actions/counter/increment")).status, 404);
  await writeFile(path.join(out, "document.html"), before);
  await writeFile(path.join(out, "increment.sse"), patch);
});
await withServer(async () => {
  const response = await request("/");
  assert.equal(response.status, 200);
  assert.match(await response.text(), /<output[^>]*>1<\/output>/);
});
console.log(
  "PASS: unchanged counter document, finite fragment, routes, and persistent reconstruction in actual workerd",
);

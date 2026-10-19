import { mkdir, mkdtemp, copyFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
import { serve } from "./serve.mjs";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const pkg = fileURLToPath(new URL("../", import.meta.url));
const built = path.join(root, "target/worker-transactions");
const out = await mkdtemp(path.join(built, "run-"));
await mkdir(path.join(out, "runtime"));
await copyFile(path.join(built, "app.wasm"), path.join(out, "app.wasm"));
for (const file of ["admission", "sql", "guest", "cloudflare", "transactions"])
  await copyFile(
    path.join(pkg, `runtime/${file}.mjs`),
    path.join(out, `runtime/${file}.mjs`),
  );
await writeFile(
  path.join(out, "index.mjs"),
  `
import module from './app.wasm';
import {WorkerObject,objectWorker} from './runtime/cloudflare.mjs';
import {Guest} from './runtime/guest.mjs';
export class RocciApp extends WorkerObject{
  constructor(ctx,env){super(module,ctx,env,{sqliteAliases:['./transactions.db']});}
  async fetch(request){
    const path=new URL(request.url).pathname;
    if(path==='/__init_failure'){
      const bad=new Guest(module,{sql:this.context.storage.sql,storage:this.context.storage,aliases:[]});
      try{await bad.invoke(0);throw new Error('Invalid alias unexpectedly initialized');}catch(error){if(!String(error).includes('Roc init failed'))throw error;}
      return Response.json({closed:bad.closed,settled:bad.settled});
    }
    if(path==='/__metrics')return Response.json(await this.guest.metrics());
    if(path==='/__close'){await this.guest.invoke(4);return Response.json(await this.guest.metrics());}
    return super.fetch(request);
  }
}
export default objectWorker('APP','transactions');
`,
);
await writeFile(
  path.join(out, "wrangler.json"),
  JSON.stringify({
    name: "rocci-transaction-conformance",
    main: "index.mjs",
    compatibility_date: "2026-10-19",
    workers_dev: false,
    preview_urls: false,
    durable_objects: { bindings: [{ name: "APP", class_name: "RocciApp" }] },
    migrations: [{ tag: "v1", new_sqlite_classes: ["RocciApp"] }],
  }),
);
await serve(out, async (origin) => {
  const request = (path) =>
    fetch(origin + path, { signal: AbortSignal.timeout(10000) });
  const count = async () => Number(await (await request("/snapshot")).text());
  assert.equal(await count(), 0);
  assert.deepEqual(await(await request("/__init_failure")).json(),{closed:true,settled:true});
  const values = await (await request("/values")).text();
  for (const text of [
    "9223372036854775807",
    "-9223372036854775808",
    "🌍",
    "255",
    "null",
  ])
    assert.ok(values.includes(text), values);
  for (const [path, status, n] of [
    ["/commit", 204, 2],
    ["/rollback", 204, 2],
    ["/drop", 204, 2],
    ["/typed-error", 500, 2],
    ["/trap", 500, 2],
    ["/unsupported-effect", 500, 2],
    ["/post-commit-error", 500, 4],
  ]) {
    assert.equal((await request(path)).status, status, path);
    assert.equal(await count(), n, `visibility after ${path}`);
    console.log(`PASS ${path}: ${n} durable rows visible from a new request`);
  }
  const metrics = await (await request("/__close")).json();
  assert.equal(metrics.liveBytes, 0, JSON.stringify(metrics));
  assert.equal(metrics.resources, 0, JSON.stringify(metrics));
  await writeFile(
    path.join(built, "results.json"),
    JSON.stringify(
      {
        scope: "Actual local workerd and real Roc continuation",
        scenarios: [
          "commit",
          "two-write rollback",
          "final-reference drop rollback",
          "typed error recovery",
          "trap reconstruction",
          "unsupported effect rollback",
          "post-commit error preserves commit",
        ],
        metrics,
      },
      null,
      2,
    ) + "\n",
  );
});

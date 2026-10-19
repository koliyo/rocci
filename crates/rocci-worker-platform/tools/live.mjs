import { mkdir, mkdtemp, copyFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
import { serve } from "./serve.mjs";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const pkg = fileURLToPath(new URL("../", import.meta.url));
const fixture = process.argv[2] ?? "c2";
assert.ok(["c1", "c2"].includes(fixture));
const built = path.join(root, `target/worker-${fixture}`);
const out = await mkdtemp(path.join(built, "run-"));
await mkdir(path.join(out, "runtime"));
await mkdir(path.join(out, "public/assets"), { recursive: true });
await copyFile(path.join(built, "app.wasm"), path.join(out, "app.wasm"));
await copyFile(
  path.join(built, "assets/rocci.css"),
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
import {WorkerObject,objectWorker} from './runtime/cloudflare.mjs';
export class RocciApp extends WorkerObject{
  constructor(ctx,env){super(module,ctx,env,{sqliteAliases:['./counter.db']});}
  async fetch(request){
    const path=new URL(request.url).pathname;
    if(path==='/__metrics')return Response.json(await this.guest.metrics());
    return super.fetch(request);
  }
}
export default objectWorker('APP','live');
`,
);
await writeFile(
  path.join(out, "wrangler.json"),
  JSON.stringify({
    name: `rocci-${fixture}-conformance`,
    main: "index.mjs",
    compatibility_date: "2026-10-19",
    workers_dev: false,
    preview_urls: false,
    assets: { directory: "public", binding: "ASSETS", run_worker_first: true },
    durable_objects: { bindings: [{ name: "APP", class_name: "RocciApp" }] },
    migrations: [{ tag: "v1", new_sqlite_classes: ["RocciApp"] }],
  }),
);
await serve(
  out,
  async (origin) => {
    const command = async (path, datastar = false) => {
      const response = await fetch(origin + `/actions/counter/${path}`, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          ...(datastar ? { "datastar-request": "true" } : {}),
        },
        body: '{"origin":"Oslo","tz":"Europe/Oslo"}',
        signal: AbortSignal.timeout(20000),
      });
      assert.equal(response.status, datastar ? 200 : 204);
      if (datastar) {
        assert.match(
          response.headers.get("content-type"),
          /text\/event-stream/,
        );
        assert.equal(await response.text(), "");
      }
    };
    await command("reset");
    const document = await (await fetch(origin + "/")).text();
    assert.match(document, /<html/);
    assert.match(document, /\/sse/);
    assert.match(
      await (await fetch(origin + "/assets/datastar.js")).text(),
      /datastar/i,
    );
    async function reader() {
      const controller = new AbortController();
      const response = await fetch(origin + "/sse", {
        signal: controller.signal,
      });
      assert.match(response.headers.get("content-type"), /text\/event-stream/);
      const reader = response.body.getReader();
      let buffer = "";
      return {
        async count(expected) {
          const deadline = Date.now() + 15000;
          while (Date.now() < deadline) {
            let timer;
            const chunk = await Promise.race([
              reader.read(),
              new Promise((_, reject) => {
                timer = setTimeout(
                  () => reject(new Error("live reader timed out")),
                  15000,
                );
              }),
            ]).finally(() => clearTimeout(timer));
            if (chunk.done) throw new Error("live stream ended");
            buffer += new TextDecoder().decode(chunk.value);
            if (
              new RegExp(`<output[^>]*>${expected}<\\/output>`).test(buffer)
            ) {
              buffer = "";
              return;
            }
          }
          throw new Error(`count ${expected} never reached reader`);
        },
        async close() {
          controller.abort();
          await reader.cancel().catch(() => {});
        },
      };
    }
    const first = await reader(),
      second = await reader();
    await Promise.all([first.count(0), second.count(0)]);
    await new Promise((resolve) => setTimeout(resolve, 1500));
    await command("increment", true);
    await Promise.all([first.count(1), second.count(1)]);
    await Promise.all(Array.from({ length: 20 }, () => command("increment")));
    await Promise.all([first.count(21), second.count(21)]);
    await first.close();
    await second.close();
    const reopened = await reader();
    await reopened.count(21);
    await reopened.close();
    await new Promise((resolve) => setTimeout(resolve, 100));
    const metrics = await (await fetch(origin + "/__metrics")).json();
    await writeFile(
      path.join(built, "live-results.json"),
      JSON.stringify(
        {
          fixture,
          scope:
            "Actual local workerd, unchanged effectful handlers; two HTTP SSE readers",
          commands: 21,
          reconnectedCount: 21,
          metrics,
        },
        null,
        2,
      ) + "\n",
    );
    console.log(
      `PASS ${fixture}: two live readers, 20 concurrent commands, empty-SSE/204 roles, reconnect, and cancellation`,
    );
    if (process.argv.includes("--keep")) {
      console.log(
        `Browser origin: ${origin}; press Ctrl-C after browser validation`,
      );
      await new Promise((resolve) => process.once("SIGINT", resolve));
    }
  },
  { port: 8796, ready: "/health" },
);

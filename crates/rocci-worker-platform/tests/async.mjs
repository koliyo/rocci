import { createServer } from "node:http";
import { mkdir, mkdtemp, copyFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
import { serve } from "../tools/serve.mjs";
const root = fileURLToPath(new URL("../../../", import.meta.url)),
  pkg = fileURLToPath(new URL("../", import.meta.url));
const built = path.join(root, "target/worker-async"),
  out = await mkdtemp(path.join(built, "run-"));
await mkdir(path.join(out, "runtime"));
await copyFile(path.join(built, "app.wasm"), path.join(out, "app.wasm"));
for (const name of ["admission", "guest", "sql", "transactions", "cloudflare"])
  await copyFile(
    path.join(pkg, `runtime/${name}.mjs`),
    path.join(out, `runtime/${name}.mjs`),
  );
await writeFile(
  path.join(out, "index.mjs"),
  `
import module from './app.wasm';
import {statelessWorker,respond} from './runtime/cloudflare.mjs';
import {Guest} from './runtime/guest.mjs';
const app=statelessWorker(module,{sources:{fallback:"Async.roc:1"}});
export default {async fetch(request,env){
 if(new URL(request.url).pathname!='/__cancel')return app.fetch(request,env);
 const overloaded=new Guest(module);await overloaded.invoke(0);
 const admissions=await Promise.allSettled(Array.from({length:50},()=>respond(overloaded,new Request('http://fixture/sleep'))));
 if(admissions.filter(result=>result.status==='rejected').length!==17)throw new Error('Admission bound changed');
 if((await overloaded.metrics()).resources!==0)throw new Error('Rejected requests leaked body resources');
 await overloaded.invoke(4);
 const limited=new Guest(module);await limited.invoke(0);
 for(let n=0;n<1024;n++)limited.add({kind:'qualification'});
 let saturated=false;try{limited.add({kind:'qualification'});}catch(error){saturated=error.code===-4;}
 if(!saturated||(await limited.metrics()).resources!==1024)throw new Error('Resource bound was not enforced');
 await limited.invoke(4);
 let releaseRollback,rolledBack=false;
 const rollbackFence=new Promise(resolve=>releaseRollback=resolve);
 const fenced=new Guest(module,{storage:{async transaction(callback){
  await callback({rollback(){rolledBack=true;}});
  await rollbackFence;
 }}});
 await fenced.invoke(0);await fenced.transactions.begin(0);
 const failed=respond(fenced,new Request('http://fixture/sleep')).catch(error=>error);
 for(let n=0;n<100&&!rolledBack;n++)await new Promise(resolve=>setTimeout(resolve,1));
 if(!rolledBack||!fenced.closed||fenced.settled)throw new Error('Provider rollback did not fence reconstruction');
 releaseRollback();await failed;
 if(!fenced.settled)throw new Error('Completed rollback retained reconstruction fence');
 for(let i=0;i<100;i++){
  const guest=new Guest(module);await guest.invoke(0);
  const body=guest.add({kind:'body',request:new Request('http://fixture/sleep'),finished:false});
  const controller=new AbortController();
  const active=guest.invoke(1,{method:3,path:'/sleep',headers:[],body},{signal:controller.signal});
  setTimeout(()=>controller.abort(new Error('qualification cancellation')),1);
  try{await active;throw new Error('cancelled operation succeeded');}catch(error){if(!String(error).includes('qualification cancellation'))throw error;}
  for(let n=0;n<100&&!guest.settled;n++)await new Promise(resolve=>setTimeout(resolve,1));
  if(!guest.closed||!guest.settled)throw new Error('abandoned continuation did not settle');
 }
 return Response.json({cancelled:100});
}};
`,
);
await writeFile(
  path.join(out, "wrangler.json"),
  JSON.stringify({
    name: "rocci-async-conformance",
    main: "index.mjs",
    compatibility_date: "2026-10-19",
    vars: { UPSTREAM_URL: "http://127.0.0.1:8799" },
    workers_dev: false,
    preview_urls: false,
  }),
);
const upstream = createServer((request, response) => {
  if (request.url === "/timeout") {
    setTimeout(() => response.end("late"), 300);
    return;
  }
  if (request.url === "/redirect") {
    response.writeHead(302, { location: "/small" });
    response.end();
    return;
  }
  if (request.url === "/large") {
    response.end(Buffer.alloc(300 * 1024, 255));
    return;
  }
  if (request.url === "/chunk") {
    response.end(Buffer.alloc(200 * 1024, 123));
    return;
  }
  response.end("Roc fetch 🌍");
});
await new Promise((resolve) => upstream.listen(8799, "127.0.0.1", resolve));
try {
  await serve(
    out,
    async (origin) => {
      const request = (url, init = {}) =>
        fetch(origin + url, {
          ...init,
          redirect: "manual",
          signal: AbortSignal.timeout(10000),
        });
      assert.equal(await (await request("/small")).text(), "Roc fetch 🌍");
      assert.match(
        await (await request("/method", { method: "PROPFIND" })).text(),
        /PROPFIND/,
      );
      assert.equal(
        (await (await request("/chunk")).arrayBuffer()).byteLength,
        200 * 1024,
      );
      assert.equal((await request("/redirect")).status, 302);
      assert.equal((await request("/large")).status, 500);
      assert.equal((await request("/timeout")).status, 500);
      const bytes = Buffer.alloc(1024 * 1024, 255);
      bytes[0] = 0;
      assert.deepEqual(
        Buffer.from(
          await (
            await request("/body", { method: "POST", body: bytes })
          ).arrayBuffer(),
        ),
        bytes,
      );
      assert.equal(
        (
          await request("/body", {
            method: "POST",
            body: Buffer.alloc(1024 * 1024 + 1),
          })
        ).status,
        500,
      );
      assert.equal(await (await request("/small")).text(), "Roc fetch 🌍");
      assert.deepEqual(await (await request("/__cancel")).json(), {
        cancelled: 100,
      });
      await writeFile(
        path.join(built, "results.json"),
        JSON.stringify(
          {
            scope:
              "Actual Roc and workerd: bounded async HTTP/body, 100 cancelled actual continuations",
            passed: true,
          },
          null,
          2,
        ) + "\n",
      );
      console.log(
        "PASS: async fetch/redirect/timeout/size, binary body bounds, cancellation and recovery",
      );
    },
    { port: 8798, ready: "/small" },
  );
} finally {
  await new Promise((resolve) => upstream.close(resolve));
}

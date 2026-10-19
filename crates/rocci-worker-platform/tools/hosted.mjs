/** Explicit owner-authorized, disposable managed SQL/live acceptance lane.
 * Creates a guarded test Worker and three SQLite namespaces. Deletes that
 * exact test Worker afterwards. Never binds an existing namespace or app DB.
 */
import {
  mkdir,
  mkdtemp,
  copyFile,
  readFile,
  writeFile,
} from "node:fs/promises";
import { randomBytes } from "node:crypto";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
const root = fileURLToPath(new URL("../../../", import.meta.url)),
  pkg = fileURLToPath(new URL("../", import.meta.url));
const out = await mkdtemp(path.join(root, "target/worker-hosted-"));
const name = `rocci-portability-test-${Date.now().toString(36)}`;
const token = randomBytes(32).toString("hex");
const env = {
  ...process.env,
  WRANGLER_SEND_METRICS: "false",
  CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV: "false",
  XDG_CONFIG_HOME: path.join(out, "config"),
  WRANGLER_LOG_PATH: path.join(out, "wrangler.log"),
};
if (!process.env.ROCCI_WORKER_CREDENTIALS)
  throw new Error(
    "Explicit ROCCI_WORKER_CREDENTIALS is required for the hosted acceptance lane",
  );
for (const line of (
  await readFile(process.env.ROCCI_WORKER_CREDENTIALS, "utf8")
).split("\n")) {
  const match = line.match(
    /^(CLOUDFLARE_API_TOKEN|CLOUDFLARE_ACCOUNT_ID)\s*=\s*["']?([^"'\r\n]+)["']?\s*$/,
  );
  if (match) env[match[1]] = match[2].trim();
}
const wrangler =
  process.env.WRANGLER_JS ??
  path.join(pkg, "tooling/node_modules/wrangler/bin/wrangler.js");
const run = (args, options = {}) => {
  const result = spawnSync(process.execPath, [wrangler, ...args], {
    cwd: out,
    env,
    encoding: "utf8",
    timeout: 120000,
    ...options,
  });
  if (result.status !== 0)
    throw new Error(
      `Wrangler ${args[0]} failed:\n${result.stdout}\n${result.stderr}`,
    );
  return `${result.stdout}\n${result.stderr}`;
};
await mkdir(path.join(out, "runtime"));
for (const file of ["guest", "cloudflare", "transactions", "admission", "sql"])
  await copyFile(
    path.join(pkg, `runtime/${file}.mjs`),
    path.join(out, `runtime/${file}.mjs`),
  );
for (const fixture of ["c0", "c2", "transactions"])
  await copyFile(
    path.join(root, `target/worker-${fixture}/app.wasm`),
    path.join(out, `${fixture}.wasm`),
  );
await writeFile(
  path.join(out, "index.mjs"),
  `
import c0 from './c0.wasm';import c2 from './c2.wasm';import tx from './transactions.wasm';
import {WorkerObject} from './runtime/cloudflare.mjs';
export class Starter extends WorkerObject{constructor(ctx,env){super(c0,ctx,env,{sqliteAliases:['./counter.db']});}}
export class Live extends WorkerObject{constructor(ctx,env){super(c2,ctx,env,{sqliteAliases:['./counter.db']});}}
export class Transactions extends WorkerObject{constructor(ctx,env){super(tx,ctx,env,{sqliteAliases:['./transactions.db']});}}
export default {async fetch(request,env){
 if(!env.TEST_TOKEN||request.headers.get('Authorization')!=='Bearer '+env.TEST_TOKEN)return new Response('Unauthorized',{status:401});
 const url=new URL(request.url);const parts=url.pathname.split('/');const binding={c0:'STARTER',c2:'LIVE',tx:'TRANSACTIONS'}[parts[1]];
 if(!binding)return new Response('Missing fixture',{status:404});url.pathname='/'+parts.slice(2).join('/');
 return env[binding].getByName('qualification').fetch(new Request(url,request));
}};
`,
);
await writeFile(
  path.join(out, "wrangler.json"),
  JSON.stringify(
    {
      name,
      main: "index.mjs",
      compatibility_date: "2026-10-19",
      workers_dev: true,
      preview_urls: false,
      durable_objects: {
        bindings: [
          { name: "STARTER", class_name: "Starter" },
          { name: "LIVE", class_name: "Live" },
          { name: "TRANSACTIONS", class_name: "Transactions" },
        ],
      },
      migrations: [
        { tag: "v1", new_sqlite_classes: ["Starter", "Live", "Transactions"] },
      ],
    },
    null,
    2,
  ),
);
let deployed = false;
const results = {
  name,
  scope:
    "Disposable authenticated managed Worker with newly created SQLite namespaces",
  scenarios: [],
};
try {
  const deployment = run(["deploy", "--config", "wrangler.json"]);
  deployed = true;
  const origin = deployment.match(/https:\/\/[a-z0-9.-]+\.workers\.dev/)?.[0];
  assert.ok(origin, "No workers.dev URL in deploy output");
  results.origin = origin;
  let status;
  for (let attempt = 0; attempt < 120; attempt++) {
    status = (await fetch(origin, { signal: AbortSignal.timeout(10000) }))
      .status;
    if (status === 401) break;
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  assert.equal(status, 401); // Denies every route before a secret is installed.
  run(["secret", "put", "TEST_TOKEN", "--config", "wrangler.json"], {
    input: token + "\n",
  });
  let ready = false,
    lastStatus;
  for (let attempt = 0; attempt < 120; attempt++) {
    const response = await fetch(
      origin + "/c0/health?qualification=" + Date.now(),
      {
        headers: {
          Authorization: `Bearer ${token}`,
          "Cache-Control": "no-cache",
        },
        signal: AbortSignal.timeout(10000),
      },
    );
    lastStatus = response.status;
    if (response.ok) {
      ready = true;
      break;
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  assert.ok(
    ready,
    "Hosted authenticated route did not propagate; last status " + lastStatus,
  );
  const request = async (url, init = {}) => {
    // Only the guard's 401 proves the operation never entered the object. Wait
    // for secret propagation; never replay a timeout, 5xx or lost response.
    for (let attempt = 0; attempt < 30; attempt++) {
      const response = await fetch(origin + url, {
        ...init,
        headers: { Authorization: `Bearer ${token}`, ...init.headers },
        signal: init.signal ?? AbortSignal.timeout(20000),
      });
      if (response.status !== 401) return response;
      await response.arrayBuffer();
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
    throw new Error("Hosted secret propagation did not settle");
  };
  assert.match(
    await (
      await request("/c0/actions/counter/reset", { method: "POST" })
    ).text(),
    /<output[^>]*>0<\/output>/,
  );
  assert.match(
    await (
      await request("/c0/actions/counter/increment", {
        method: "POST",
        headers: { "datastar-request": "true" },
      })
    ).text(),
    /<output[^>]*>1<\/output>/,
  );
  assert.match(
    await (await request("/c0/")).text(),
    /<output[^>]*>1<\/output>/,
  );
  results.scenarios.push("unchanged C0 init/document/finite fragment");
  const values = await (await request("/tx/values")).text();
  for (const value of [
    "9223372036854775807",
    "-9223372036854775808",
    "🌍",
    "255",
    "null",
  ])
    assert.ok(values.includes(value), values);
  results.scenarios.push(
    "typed full-range I64, REAL, text, blob and NULL storage class",
  );
  for (const [endpoint, count] of [
    ["commit", 2],
    ["rollback", 2],
    ["drop", 2],
    ["typed-error", 2],
    ["trap", 2],
    ["unsupported-effect", 2],
    ["post-commit-error", 4],
  ]) {
    await request("/tx/" + endpoint);
    assert.equal(Number(await (await request("/tx/snapshot")).text()), count);
  }
  results.scenarios.push(
    "real transaction commit/rollback/drop/typed error/trap/post-commit visibility",
  );
  await request("/c2/actions/counter/reset", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: "{}",
  });
  results.scenarios.push("C2 reset status: " + (await request("/c2/")).status);
  async function subscriber() {
    const controller = new AbortController();
    const response = await request("/c2/sse", { signal: controller.signal });
    assert.equal(response.status, 200);
    const reader = response.body.getReader();
    return {
      async count(expected) {
        let text = "";
        const timeout = setTimeout(() => controller.abort(), 20000);
        try {
          while (true) {
            const chunk = await reader.read();
            if (chunk.done) throw new Error("SSE ended");
            text += new TextDecoder().decode(chunk.value);
            if (
              new RegExp("<output[^>]*>" + expected + "<\\/output>").test(text)
            )
              return;
          }
        } finally {
          clearTimeout(timeout);
        }
      },
      async close() {
        controller.abort();
        await reader.cancel().catch(() => {});
      },
    };
  }
  const first = await subscriber();
  results.scenarios.push("first SSE headers");
  await first.count(0);
  results.scenarios.push("first SSE snapshot");
  const second = await subscriber();
  results.scenarios.push("second SSE headers");
  try {
    await second.count(0);
    results.scenarios.push("second SSE snapshot");
    await request("/c2/actions/counter/increment", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: "{}",
    });
    await Promise.all([first.count(1), second.count(1)]);
  } finally {
    await first.close();
    await second.close();
  }
  results.scenarios.push(
    "unchanged C2 two subscribers and commit-before-publish",
  );
  run(["deploy", "--config", "wrangler.json"]);
  assert.match(
    await (await request("/c0/")).text(),
    /<output[^>]*>1<\/output>/,
  );
  results.scenarios.push("redeploy reconstructs durable C0 state");
  results.passed = true;
} finally {
  if (deployed) {
    run(["delete", "--name", name, "--force"]);
    results.cleanedUp = true;
  }
  await writeFile(
    path.join(out, "results.json"),
    JSON.stringify(results, null, 2) + "\n",
  );
}
console.log(`PASS: hosted application acceptance and cleanup: ${out}`);

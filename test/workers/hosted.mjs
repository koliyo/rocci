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
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
import { applications } from "./cases.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url));
const pkg = path.join(root, "crates/rocci-worker-platform");
const subjects = await Promise.all(
  applications
    .filter((application) => application.hosted)
    .map(async (application) => ({
      id: application.id,
      ...(await import(
        pathToFileURL(path.join(root, application.hosted)).href
      )),
    })),
);
assert.ok(subjects.length);
for (const { profile } of subjects) {
  assert.match(profile.className, /^[A-Za-z_][A-Za-z0-9_]*$/);
  assert.match(profile.binding, /^[A-Za-z_][A-Za-z0-9_]*$/);
  assert.match(profile.prefix, /^[a-z][a-z0-9-]*$/);
}

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
for (const application of subjects)
  await copyFile(
    path.join(root, `target/worker-${application.id}/app.wasm`),
    path.join(out, `${application.id}.wasm`),
  );
const definitions = subjects.map(
  (application, index) => `
import module${index} from './${application.id}.wasm';
export class ${application.profile.className} extends WorkerObject {
 constructor(ctx, env) { super(module${index}, ctx, env, {sqliteAliases:${JSON.stringify(application.profile.aliases)}}); }
}`,
);
const bindings = Object.fromEntries(
  subjects.map((application) => [
    application.profile.prefix,
    application.profile.binding,
  ]),
);
await writeFile(
  path.join(out, "index.mjs"),
  `
import {WorkerObject} from './runtime/cloudflare.mjs';
${definitions.join("\n")}
const bindings = ${JSON.stringify(bindings)};
export default {async fetch(request,env){
 if(!env.TEST_TOKEN||request.headers.get('Authorization')!=='Bearer '+env.TEST_TOKEN)return new Response('Unauthorized',{status:401});
 const url=new URL(request.url);const parts=url.pathname.split('/');const binding=bindings[parts[1]];
 if(!binding)return new Response('Missing test application',{status:404});url.pathname='/'+parts.slice(2).join('/');
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
        bindings: subjects.map((application) => ({
          name: application.profile.binding,
          class_name: application.profile.className,
        })),
      },
      migrations: [
        {
          tag: "v1",
          new_sqlite_classes: subjects.map(
            (application) => application.profile.className,
          ),
        },
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
      origin +
        "/" +
        subjects[0].profile.prefix +
        "/health?qualification=" +
        Date.now(),
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
  const forApplication = (application) => (url, init) =>
    request("/" + application.profile.prefix + url, init);
  for (const application of subjects)
    await application.exercise(forApplication(application), results.scenarios);
  run(["deploy", "--config", "wrangler.json"]);
  for (const application of subjects)
    if (application.afterRedeploy)
      await application.afterRedeploy(
        forApplication(application),
        results.scenarios,
      );
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

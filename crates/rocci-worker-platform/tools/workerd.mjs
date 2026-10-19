import { mkdir, readFile, writeFile, copyFile } from "node:fs/promises";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const pkg = fileURLToPath(new URL("../", import.meta.url));
const out = path.join(root, "target/worker-conformance");
const wrangler =
  process.env.WRANGLER_JS ??
  path.join(
    root,
    "crates/rocci-worker-platform/tooling/node_modules/wrangler/bin/wrangler.js",
  );
const remote = process.argv.includes("--remote");
const result = spawnSync(process.execPath, [wrangler, "--version"], {
  encoding: "utf8",
});
assert.equal(result.status, 0, result.stderr);
assert.match(result.stdout, /4\.151\.1/);
await mkdir(out, { recursive: true });
for (const file of ["admission.mjs", "qualification.mjs"])
  await copyFile(path.join(pkg, "runtime", file), path.join(out, file));
await writeFile(
  path.join(out, "index.mjs"),
  `
import dev from './dev-yes.wasm';
import speed from './speed-yes.wasm';
import size from './size-yes.wasm';
import { qualify } from './qualification.mjs';
export default { async fetch(request) {
  const opt = new URL(request.url).searchParams.get('opt');
  if (!opt) return new Response('ready');
  const module = {dev, speed, size}[opt];
  if (!module) return new Response('unknown profile', {status: 400});
  try { return Response.json({ opt, result: await qualify(module) }); }
  catch (error) { return Response.json({ opt, error: String(error) }, {status: 500}); }
} };
`,
);
await writeFile(
  path.join(out, "wrangler.json"),
  JSON.stringify(
    {
      name: "rocci-source-portability-probe",
      main: "index.mjs",
      compatibility_date: "2026-10-19",
      workers_dev: false,
      preview_urls: false,
    },
    null,
    2,
  ) + "\n",
);
const env = {
  ...process.env,
  WRANGLER_SEND_METRICS: "false",
  CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV: "false",
  XDG_CONFIG_HOME: path.join(out, "config"),
  WRANGLER_LOG_PATH: path.join(out, "wrangler.log"),
};
// Optional explicit credential file: read only the two provider credentials;
// never copy credentials to an artifact, module graph, manifest or diagnostic.
if (remote && process.env.ROCCI_WORKER_CREDENTIALS) {
  const text = await readFile(process.env.ROCCI_WORKER_CREDENTIALS, "utf8");
  for (const line of text.split("\n")) {
    const match = line.match(
      /^(CLOUDFLARE_API_TOKEN|CLOUDFLARE_ACCOUNT_ID)\s*=\s*["']?([^"'\r\n]+)["']?\s*$/,
    );
    if (match) env[match[1]] = match[2].trim();
  }
}
const child = spawn(
  process.execPath,
  [
    wrangler,
    "dev",
    "--config",
    path.join(out, "wrangler.json"),
    "--ip",
    "127.0.0.1",
    "--port",
    "8793",
    "--inspector-port",
    "9293",
    ...(remote ? ["--remote"] : []),
  ],
  { cwd: out, env, detached: true, stdio: ["ignore", "pipe", "pipe"] },
);
let output = "";
child.stdout.on("data", (chunk) => {
  output += chunk;
});
child.stderr.on("data", (chunk) => {
  output += chunk;
});
const exited = new Promise((resolve) => child.once("exit", resolve));
let stopped = false;
child.once("exit", () => {
  stopped = true;
});
try {
  let ready = false;
  for (let i = 0; i < 300; i++) {
    if (stopped) throw new Error("Wrangler exited before becoming ready");
    try {
      ready = (
        await fetch("http://127.0.0.1:8793/", {
          signal: AbortSignal.timeout(1000),
        })
      ).ok;
    } catch {}
    if (ready) break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.ok(ready, "Wrangler did not become ready within 30 seconds");
  const results = [];
  for (const opt of ["dev", "speed", "size"]) {
    console.log(
      `Qualify ${opt} in ${remote ? "managed preview" : "local workerd"}`,
    );
    const response = await fetch(`http://127.0.0.1:8793/?opt=${opt}`, {
      signal: AbortSignal.timeout(60_000),
    });
    const value = await response.json();
    results.push(value);
    await writeFile(
      path.join(out, remote ? "hosted-results.json" : "workerd-results.json"),
      JSON.stringify(
        {
          scope: remote
            ? "Managed remote preview; no public routes or durable namespace"
            : "Local workerd via Wrangler 4.151.1",
          compatibilityDate: "2026-10-19",
          results,
        },
        null,
        2,
      ) + "\n",
    );
    assert.equal(response.status, 200, JSON.stringify(value));
    console.log(
      `${opt}: ${value.result.suspensions} suspensions, ${value.result.liveBytesAfterClose} guest bytes after close`,
    );
  }
} finally {
  // Terminate only the process group created by this harness, including workerd.
  if (!stopped) {
    try {
      process.kill(-child.pid, "SIGTERM");
    } catch {}
  }
  await Promise.race([
    exited,
    new Promise((resolve) => setTimeout(resolve, 2000)),
  ]);
  if (!stopped) {
    try {
      process.kill(-child.pid, "SIGKILL");
    } catch {}
  }
  await writeFile(
    path.join(out, remote ? "hosted-dev.log" : "local-dev.log"),
    output,
  );
}

/** Fresh app, normal npm/Wrangler hooks, watch rebuild and atomic failure proof. */
import { spawn, spawnSync } from "node:child_process";
import { mkdtemp, readFile, writeFile, mkdir, readdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
const root = fileURLToPath(new URL("../../", import.meta.url));
const cli = path.join(root, "target/debug/rocci");
const out = await mkdtemp(path.join(root, "target/worker fresh app "));
const env = {
  ...process.env,
  PATH: [
    path.dirname(cli),
    path.dirname(process.execPath),
    process.env.PATH,
  ].join(path.delimiter),
  ROCCI_WORKER_ROC: process.env.ROCCI_WORKER_ROC ?? "roc",
  ROCCI_CACHE: path.join(root, "target/worker-package-cache"),
  WRANGLER_SEND_METRICS: "false",
  CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV: "false",
  XDG_CONFIG_HOME: path.join(out, ".config"),
};
function run(cmd, args, options = {}) {
  const result = spawnSync(cmd, args, {
    cwd: out,
    env,
    encoding: "utf8",
    timeout: 120000,
    ...options,
  });
  if (result.status !== 0)
    throw new Error(`${cmd}: ${result.stdout}\n${result.stderr}`);
  return result.stdout;
}
run(cli, ["worker", "init", out, "--stateless"]);
run(cli, ["worker", "build", "--offline"]);
assert.match(
  run(cli, ["worker", "build", "--offline", "--locked"]),
  /build cached/,
);
await mkdir(path.join(out, "assets"), { recursive: true });
await writeFile(
  path.join(out, "assets/custom.bin"),
  new Uint8Array([0, 255, 1]),
);
assert.doesNotMatch(run(cli, ["worker", "build", "--offline"]), /build cached/);
assert.deepEqual(
  await readFile(path.join(out, ".rocci/worker/public/assets/custom.bin")),
  Buffer.from([0, 255, 1]),
);
const wasm = await readFile(path.join(out, ".rocci/worker/app.wasm"));
await writeFile(path.join(out, ".rocci/worker/app.wasm"), "tampered");
assert.doesNotMatch(run(cli, ["worker", "build", "--offline"]), /build cached/);
assert.deepEqual(
  await readFile(path.join(out, ".rocci/worker/app.wasm")),
  wasm,
);
const entry = path.join(out, "Hello.rocci"),
  original = await readFile(entry, "utf8");
await writeFile(entry, original + "\nthis cannot compile\n");
const failed = spawnSync(cli, ["worker", "build", "--offline"], {
  cwd: out,
  env,
  encoding: "utf8",
});
assert.notEqual(failed.status, 0);
assert.deepEqual(
  await readFile(path.join(out, ".rocci/worker/app.wasm")),
  wasm,
);
await writeFile(entry, original);
run(cli, ["worker", "build", "--offline"]);
run("npm", ["ci", "--no-audit", "--no-fund"]);
const child = spawn(
  "npm",
  [
    "run",
    "dev",
    "--",
    "--ip",
    "127.0.0.1",
    "--port",
    "8797",
    "--inspector-port",
    "9297",
  ],
  { cwd: out, env, detached: true, stdio: ["ignore", "pipe", "pipe"] },
);
let log = "",
  stopped = false;
child.stdout.on("data", (chunk) => {
  log += chunk;
  process.stdout.write(chunk);
});
child.stderr.on("data", (chunk) => {
  log += chunk;
  process.stderr.write(chunk);
});
const exited = new Promise((resolve) => child.once("exit", resolve));
child.once("exit", () => {
  stopped = true;
});
const origin = "http://127.0.0.1:8797";
async function contains(expected) {
  for (let deadline = Date.now() + 30000; Date.now() < deadline; ) {
    if (stopped) throw new Error(log);
    try {
      const response = await fetch(origin, {
        signal: AbortSignal.timeout(1000),
      });
      if (response.ok && (await response.text()).includes(expected)) return;
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`Never served ${expected}\n${log}`);
}
try {
  await contains("Hello from Rocci");
  await writeFile(
    entry,
    original.replace("Hello from Rocci", "Watch rebuild verified"),
  );
  await contains("Watch rebuild verified");
  const count = () => log.match(/\[custom build\] Running:/g)?.length ?? 0;
  const before = count();
  await new Promise((resolve) => setTimeout(resolve, 3000));
  assert.equal(count(), before, "Watch loop did not quiesce");
  await writeFile(
    path.join(out, "packaging-results.json"),
    JSON.stringify(
      {
        scope:
          "Fresh path with spaces, npm ci/run dev, source watch, cache/tamper/failure checks",
        watchBuilds: count(),
      },
      null,
      2,
    ) + "\n",
  );
} finally {
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
  await writeFile(path.join(out, "packaging.log"), log);
}
run("npm", [
  "run",
  "deploy",
  "--",
  "--dry-run",
  "--outdir",
  path.join(root, "target/worker-dry-deploy", path.basename(out)),
]);
const deployment = path.join(
  root,
  "target/worker-dry-deploy",
  path.basename(out),
);
const moduleFile = (await readdir(deployment)).find((name) =>
  name.endsWith(".wasm"),
);
assert.ok(moduleFile, "Dry deploy omitted Wasm");
assert.deepEqual(
  await readFile(path.join(deployment, moduleFile)),
  await readFile(path.join(out, ".rocci/worker/app.wasm")),
);
console.log(`PASS: packaging and normal Wrangler workflow: ${out}`);

/** Release-only packaging. Applications do not need Cargo, clang or this repo. */
import {
  mkdtemp,
  mkdir,
  copyFile,
  readFile,
  writeFile,
  readdir,
} from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";
import assert from "node:assert/strict";
import path from "node:path";
const root = fileURLToPath(new URL("../../../", import.meta.url)),
  pkg = fileURLToPath(new URL("../", import.meta.url));
const base = path.join(root, "target/worker-release");
await mkdir(base, { recursive: true });
const out = await mkdtemp(path.join(base, "run-"));
function run(cmd, args, cwd = root) {
  const result = spawnSync(cmd, args, {
    cwd,
    env: process.env,
    encoding: "utf8",
    timeout: 300000,
  });
  if (result.status !== 0)
    throw new Error(`${cmd}: ${result.stdout}\n${result.stderr}`);
  return result.stdout;
}
run("cargo", [
  "run",
  "-q",
  "-p",
  "rocci-cli",
  "--example",
  "worker_bundle",
  "--",
  out,
  path.join(
    root,
    "target/wasm32-unknown-unknown/release/librocci_worker_platform.a",
  ),
  process.env.ROCCI_WORKER_ROC ?? "roc",
]);
const descriptor = JSON.parse(
  await readFile(path.join(out, "rocci-worker-platform.json"), "utf8"),
);
const npm = path.join(out, "npm/package");
await mkdir(path.join(npm, "runtime"), { recursive: true });
for (const name of [
  "cloudflare.mjs",
  "cloudflare.d.mts",
  "admission.mjs",
  "guest.mjs",
  "sql.mjs",
  "transactions.mjs",
])
  await copyFile(
    path.join(pkg, "runtime", name),
    path.join(npm, "runtime", name),
  );
await writeFile(
  path.join(npm, "package.json"),
  JSON.stringify(
    {
      name: "@rocci/worker-runtime",
      version: descriptor.version,
      type: "module",
      license: "Apache-2.0",
      exports: {
        ".": {
          types: "./runtime/cloudflare.d.mts",
          default: "./runtime/cloudflare.mjs",
        },
      },
    },
    null,
    2,
  ),
);
await copyFile(path.join(root, "LICENSE"), path.join(npm, "LICENSE"));
await writeFile(
  path.join(npm, "README.md"),
  "# Roc Worker runtime\n\nMaintained core-Wasm adapter. See https://github.com/koliyo/rocci/tree/main/crates/rocci-worker-platform for the compiler pin, ABI, limits, plain Roc examples and conformance.\n",
);
run("tar", [
  "-czf",
  path.join(out, descriptor.runtime),
  "-C",
  path.join(out, "npm"),
  "package",
]);
await copyFile(
  path.join(
    root,
    "target/wasm32-unknown-unknown/release/librocci_worker_platform.a",
  ),
  path.join(out, "rocci-worker-host.a"),
);
for (const name of await readdir(out)) {
  if (
    !name.endsWith(".a") &&
    !name.endsWith(".tar.zst") &&
    !name.endsWith(".tar.gz") &&
    !name.endsWith(".json")
  )
    continue;
  const hash = createHash("sha256")
    .update(await readFile(path.join(out, name)))
    .digest("hex");
  await writeFile(path.join(out, name + ".sha256"), `${hash}  ${name}\n`);
}
console.log(`Worker platform/runtime release artifacts: ${out}`);

// Prove the exported archives are sufficient after packaging, with no frontend.
const independent = path.join(out, "independent");
await mkdir(path.join(independent, "platform"), { recursive: true });
run("tar", [
  "-xf",
  path.join(out, descriptor.bundle),
  "-C",
  path.join(independent, "platform"),
]);
await copyFile(
  path.join(pkg, "examples/hello/main.roc"),
  path.join(independent, "main.roc"),
);
const compiler = process.env.ROCCI_WORKER_ROC ?? "roc";
const built = spawnSync(
  compiler,
  [
    "build",
    "--no-cache",
    "--target=wasm32",
    "--specialize=yes",
    "--opt=speed",
    "--wasm-stack-size=1048576",
    "--output=app.wasm",
    "main.roc",
  ],
  {
    cwd: independent,
    encoding: "utf8",
    timeout: 60000,
    env: {
      ...process.env,
      XDG_CACHE_HOME: path.join(
        root,
        "target/worker-package-cache/worker/compiler-cache",
      ),
    },
  },
);
if (
  built.status !== 0 &&
  !(
    built.status === 2 &&
    `${built.stdout}${built.stderr}`
      .split("\n")
      .some((line) => line.trimStart().startsWith("0 errors and "))
  )
)
  throw new Error(`Independent Roc build: ${built.stdout}\n${built.stderr}`);
const { Guest } = await import(
  pathToFileURL(path.join(npm, "runtime/guest.mjs"))
);
const guest = new Guest(
  new WebAssembly.Module(await readFile(path.join(independent, "app.wasm"))),
);
await guest.invoke(0);
const response = await guest.invoke(1, {
  method: 3,
  path: "/",
  headers: [],
  body: guest.add({ kind: "body", request: new Request("http://fixture/") }),
});
assert.equal(
  new TextDecoder().decode(new Uint8Array(response.body)),
  "Hello from ordinary Roc",
);
await guest.invoke(4);
const metrics = await guest.metrics();
assert.equal(metrics.liveBytes, 0);
assert.equal(metrics.resources, 0);
await writeFile(
  path.join(out, "plain-roc-conformance.json"),
  JSON.stringify(
    {
      platform: descriptor.bundle,
      runtime: descriptor.runtime,
      metrics,
      passed: true,
    },
    null,
    2,
  ) + "\n",
);
console.log(
  "PASS: independent platform bundle, plain Roc build and packaged runtime",
);

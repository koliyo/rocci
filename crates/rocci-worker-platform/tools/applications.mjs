/** Maintainer lane: stage ordinary sources once, test those exact Wasm files. */
import { spawnSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import path from "node:path";
const root = fileURLToPath(new URL("../../../", import.meta.url));
const roc = process.env.ROCCI_WORKER_ROC ?? "roc";
const archive = path.join(
  root,
  "target/wasm32-unknown-unknown/release/librocci_worker_platform.a",
);
function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    timeout: 180000,
    ...options,
  });
  if (result.status !== 0)
    throw new Error(
      `${command} ${args.join(" ")}\n${result.stdout}\n${result.stderr}`,
    );
  return result;
}
const fixtures = [
  ["counter", "examples/rocci/standalone/counter/Counter.rocci"],
  ["live-counter", "examples/rocci/standalone/live-counter/LiveCounter.rocci"],
  ["transactions", "crates/rocci-worker-platform/examples/sql/main.roc"],
  ["async", "crates/rocci-worker-platform/tests/Async.roc"],
];
const artifacts = [];
for (const [name, input] of fixtures) {
  const out = path.join(root, `target/worker-${name}`);
  await mkdir(out, { recursive: true });
  run("cargo", [
    "run",
    "-q",
    "-p",
    "rocci-cli",
    "--example",
    "worker_inputs",
    "--",
    input,
    out,
    archive,
  ]);
  const built = spawnSync(
    roc,
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
      cwd: out,
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
    throw new Error(`${name}: ${built.stdout}\n${built.stderr}`);
  const bytes = await readFile(path.join(out, "app.wasm"));
  artifacts.push({
    application: name,
    source: input,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    bytes: bytes.length,
    imports: WebAssembly.Module.imports(new WebAssembly.Module(bytes)),
  });
}
run(
  process.execPath,
  [
    "--test",
    "crates/rocci-worker-platform/tests/admission.test.mjs",
    "crates/rocci-worker-platform/tests/sql.test.mjs",
  ],
  { stdio: "inherit" },
);
for (const args of [
  ["counter.mjs"],
  ["transactions.mjs"],
  ["live.mjs"],
  ["async.mjs"],
])
  run(
    process.execPath,
    [`crates/rocci-worker-platform/tools/${args[0]}`, ...args.slice(1)],
    { stdio: "inherit" },
  );
await mkdir(path.join(root, "target/worker-conformance"), { recursive: true });
await writeFile(
  path.join(root, "target/worker-conformance/applications.json"),
  JSON.stringify(
    {
      compiler: run(roc, ["version"]).stdout.trim(),
      compatibilityDate: "2026-10-19",
      artifacts,
    },
    null,
    2,
  ) + "\n",
);

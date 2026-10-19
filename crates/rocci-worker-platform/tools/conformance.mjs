import { mkdir, readFile, writeFile, copyFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { qualify } from "../runtime/qualification.mjs";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const pkg = fileURLToPath(new URL("../", import.meta.url));
const out = path.join(root, "target/worker-conformance");
const pins = {
  roc: "nightly-2026-10-09-258ab27",
  clang: "23.1.3",
  node: "v26.12.0",
  wrangler: "4.151.1",
  workerd: "1.20261012.1",
  compatibilityDate: "2026-10-19",
  abi: 1,
};
const roc = process.env.ROCCI_WORKER_ROC ?? "roc";
const clang =
  process.env.CLANG ??
  (process.platform === "darwin"
    ? "/opt/homebrew/opt/llvm/bin/clang"
    : "clang");
const env = {
  ...process.env,
  XDG_CACHE_HOME: path.join(root, "target/worker-cache"),
};
function run(cmd, args) {
  const result = spawnSync(cmd, args, {
    cwd: root,
    env,
    encoding: "utf8",
    timeout: 60_000,
  });
  if (result.status !== 0)
    throw new Error(
      `${cmd}: ${result.error ?? ""}\n${result.stdout}\n${result.stderr}`,
    );
  return (result.stdout + result.stderr).trim();
}
if (!run(roc, ["version"]).includes(pins.roc))
  throw new Error(
    `Use ${pins.roc} via ROCCI_WORKER_ROC; native .roc-version stays unchanged`,
  );
if (!run(clang, ["--version"]).includes(pins.clang))
  throw new Error(`Use Clang ${pins.clang} via CLANG`);
if (process.version !== pins.node) throw new Error(`Use Node ${pins.node}`);
await mkdir(path.join(out, "platform/targets/wasm32"), { recursive: true });
for (const file of ["main.roc", "Effect.roc"])
  await copyFile(
    path.join(pkg, "platform", file),
    path.join(out, "platform", file),
  );
await writeFile(
  path.join(out, "Effects.roc"),
  (await readFile(path.join(pkg, "tests/Effects.roc"), "utf8")).replace(
    "../platform/main.roc",
    "platform/main.roc",
  ),
);
run(clang, [
  "--target=wasm32",
  "-ffreestanding",
  "-fno-builtin",
  "-O2",
  "-Wall",
  "-Wextra",
  "-Werror",
  "-c",
  path.join(pkg, "platform/host.c"),
  "-o",
  path.join(out, "platform/targets/wasm32/host.o"),
]);
console.log(
  run(process.execPath, ["--test", path.join(pkg, "tests/admission.test.mjs")]),
);
const results = [];
for (const opt of ["dev", "speed", "size"]) {
  for (const specialize of ["yes", "no"]) {
    const name = `${opt}-${specialize}`;
    const filename = path.join(out, `${name}.wasm`);
    const record = { opt, specialize };
    try {
      console.log(`Compile and qualify ${name}`);
      run(roc, [
        "build",
        "--no-cache",
        "--target=wasm32",
        `--opt=${opt}`,
        `--specialize=${specialize}`,
        "--wasm-stack-size=1048576",
        `--output=${filename}`,
        path.join(out, "Effects.roc"),
      ]);
      const bytes = await readFile(filename);
      const module = new WebAssembly.Module(bytes);
      record.sha256 = createHash("sha256").update(bytes).digest("hex");
      record.bytes = bytes.length;
      record.imports = WebAssembly.Module.imports(module);
      record.measurementBefore = process.memoryUsage();
      record.result = await qualify(module);
      if (global.gc) global.gc();
      record.measurementAfter = process.memoryUsage();
      record.status = "passed";
    } catch (error) {
      record.status = "failed";
      record.error = String(error);
    }
    console.log(`${name}: ${record.status}`);
    results.push(record);
    await writeFile(
      path.join(out, "results.json"),
      JSON.stringify(
        {
          pins,
          scope:
            "Local Node; separate actual workerd and hosted qualification required",
          results,
        },
        null,
        2,
      ) + "\n",
    );
  }
}
// Experimental boxy lowering is measured, but is not an admitted build profile.
if (
  results.some(
    (result) => result.specialize === "yes" && result.status !== "passed",
  )
)
  process.exitCode = 1;

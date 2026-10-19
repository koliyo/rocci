/** Reuse the speed lane sources and real runtime acceptance for other profiles. */
import { readFile, writeFile, copyFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import path from "node:path";
import { fileURLToPath } from "node:url";
const root = fileURLToPath(new URL("../../../", import.meta.url));
const roc = process.env.ROCCI_WORKER_ROC ?? "roc";
const results = [];
for (const [fixture, runner] of [
  ["c0", "counter"],
  ["async", "async"],
]) {
  const directory = path.join(root, "target/worker-" + fixture),
    wasm = path.join(directory, "app.wasm");
  const original = await readFile(wasm);
  try {
    for (const opt of ["dev", "size"]) {
      const output = path.join(directory, "app-" + opt + ".wasm");
      const result = spawnSync(
        roc,
        [
          "build",
          "--no-cache",
          "--target=wasm32",
          "--specialize=yes",
          "--opt=" + opt,
          "--wasm-stack-size=1048576",
          "--output=" + output,
          "main.roc",
        ],
        {
          cwd: directory,
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
        result.status !== 0 &&
        !(
          result.status === 2 &&
          (result.stdout + result.stderr)
            .split("\n")
            .some((line) => line.trimStart().startsWith("0 errors and "))
        )
      )
        throw new Error(result.stdout + result.stderr);
      await copyFile(output, wasm);
      const test = spawnSync(
        process.execPath,
        ["crates/rocci-worker-platform/tools/" + runner + ".mjs"],
        { cwd: root, encoding: "utf8", timeout: 180000, env: process.env },
      );
      if (test.status !== 0) throw new Error(test.stdout + test.stderr);
      const bytes = await readFile(output);
      results.push({
        fixture,
        opt,
        sha256: createHash("sha256").update(bytes).digest("hex"),
        bytes: bytes.length,
        passed: true,
      });
      console.log("PASS maintained host " + fixture + " " + opt);
    }
  } finally {
    await writeFile(wasm, original);
  }
}
await writeFile(
  path.join(root, "target/worker-conformance/application-profiles.json"),
  JSON.stringify(
    {
      compiler: "nightly-2026-10-09-258ab27",
      scope:
        "Actual maintained Rust host on workerd; existing counter and async/cancellation harnesses; speed artifacts restored",
      results,
    },
    null,
    2,
  ) + "\n",
);

/** Optional two-toolchain lane; compare native and Worker counter bytes at one origin per host. */
import { mkdtemp, copyFile, writeFile } from "node:fs/promises";
import { execFileSync, spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import assert from "node:assert/strict";
import { serve } from "../../../../../crates/rocci-worker-platform/tools/serve.mjs";
const root = fileURLToPath(new URL("../../../../../", import.meta.url));
if (!process.env.ROCCI_NATIVE_ROC)
  throw new Error("Set ROCCI_NATIVE_ROC to the repository native compiler pin");
const out = await mkdtemp(path.join(root, "target/worker-native-parity-"));
await copyFile(
  path.join(root, "examples/rocci/standalone/counter/Counter.rocci"),
  path.join(out, "Counter.rocci"),
);
const env = {
  ...process.env,
  DB_PATH: path.join(out, "native-counter.db"),
  PATH: [
    path.dirname(process.env.ROCCI_NATIVE_ROC),
    path.dirname(process.execPath),
    process.env.PATH,
  ].join(path.delimiter),
  ROCCI_CACHE: path.join(root, "target/worker-package-cache"),
  XDG_CACHE_HOME: path.join(root, "target/worker-native-cache"),
};
const child = spawn(
  path.join(root, "target/debug/rocci"),
  ["run", out, "--no-window", "--port", "auto", "--no-live-reload"],
  { cwd: out, env, detached: true, stdio: ["ignore", "pipe", "pipe"] },
);
function descendants(parent) {
  const rows = execFileSync("ps", ["-axo", "pid=,ppid="], {
    encoding: "utf8",
    timeout: 1000,
  })
    .trim()
    .split("\n")
    .map((line) => line.trim().split(/\s+/).map(Number));
  const owned = new Set([parent]);
  const pending = [parent];
  while (pending.length) {
    const next = pending.pop();
    for (const [pid, ppid] of rows) {
      if (ppid === next && !owned.has(pid)) {
        owned.add(pid);
        pending.push(pid);
      }
    }
  }
  return [...owned].reverse();
}
let log = "",
  stopped = false;
child.stdout.on("data", (chunk) => {
  log += chunk;
});
child.stderr.on("data", (chunk) => {
  log += chunk;
});
const exited = new Promise((resolve) => child.once("exit", resolve));
child.once("exit", () => {
  stopped = true;
});
async function sequence(origin) {
  const data = [];
  for (const [url, method, headers] of [
    ["/actions/counter/reset", "POST", {}],
    ["/", "GET", {}],
    ["/actions/counter/increment", "POST", { "datastar-request": "true" }],
    ["/", "GET", {}],
  ]) {
    const response = await fetch(origin + url, {
      method,
      headers,
      signal: AbortSignal.timeout(10000),
    });
    data.push({
      status: response.status,
      contentType: response.headers.get("content-type"),
      body: await response.text(),
    });
  }
  return data;
}
try {
  let ready = false,
    nativeOrigin;
  for (let n = 0; n < 300; n++) {
    if (stopped) throw new Error(log);
    nativeOrigin = log
      .match(/preview_ready (http:\/\/[^\s]+)/)?.[1]
      ?.replace(/\/$/, "");
    try {
      ready =
        !!nativeOrigin &&
        (
          await fetch(nativeOrigin + "/health", {
            signal: AbortSignal.timeout(500),
          })
        ).ok;
    } catch {}
    if (ready) break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.ok(ready, log);
  const native = await sequence(nativeOrigin);
  await serve(
    path.join(root, "target/worker-counter"),
    async (origin) => {
      const worker = await sequence(origin);
      const css = [];
      const canonical = (responses) =>
        responses.map((response) => ({
          ...response,
          body: response.body
            .replace(/<style>([\s\S]*?)<\/style>/g, (_, style) => {
              css.push(style);
              return "";
            })
            .replace('<link rel="stylesheet" href="/assets/rocci.css">', ""),
        }));
      assert.deepEqual(canonical(worker), canonical(native));
      const workerCss = await (
        await fetch(origin + "/assets/rocci.css")
      ).text();
      for (const style of css) {
        for (const block of style
          .split(/(?=@scope\s*\()/)
          .filter((block) => block.trim().startsWith("@scope"))) {
          assert.ok(
            workerCss.includes(block.trim()),
            "Original scoped CSS block was not preserved in the asset",
          );
        }
      }
      await writeFile(
        path.join(out, "results.json"),
        JSON.stringify(
          {
            passed: true,
            scope:
              "Unchanged counter HTML/finite SSE bytes, status/content-type; original scoped CSS preserved in an asset with compatibility selectors instead of native inline style",
            responses: worker.map(({ body, ...rest }) => ({
              ...rest,
              bytes: Buffer.byteLength(body),
            })),
          },
          null,
          2,
        ) + "\n",
      );
    },
    { port: 8803, ready: "/health" },
  );
  console.log(`PASS: exact native/Worker counter parity: ${out}`);
} finally {
  // Roc starts its compiler/server in a separate process group. Snapshot the
  // owned descendants before terminating the CLI and losing their parent IDs.
  const owned = descendants(child.pid);
  if (!stopped) {
    try {
      process.kill(-child.pid, "SIGTERM");
    } catch {}
  }
  await Promise.race([
    exited,
    new Promise((resolve) => setTimeout(resolve, 2000)),
  ]);
  // The CLI can exit before its native server closes the inherited pipes.
  try {
    process.kill(-child.pid, "SIGKILL");
  } catch {}
  for (const pid of owned) {
    try {
      process.kill(pid, "SIGKILL");
    } catch {}
  }
  await writeFile(path.join(out, "native.log"), log);
}

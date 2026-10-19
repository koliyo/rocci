import { spawn } from "node:child_process";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

export async function serve(
  directory,
  test,
  { port = 8795, ready = "/snapshot" } = {},
) {
  const root = fileURLToPath(new URL("../../../", import.meta.url));
  const wrangler =
    process.env.WRANGLER_JS ??
    path.join(
      root,
      "crates/rocci-worker-platform/tooling/node_modules/wrangler/bin/wrangler.js",
    );
  const origin = `http://127.0.0.1:${port}`;
  const child = spawn(
    process.execPath,
    [
      wrangler,
      "dev",
      "--config",
      path.join(directory, "wrangler.json"),
      "--ip",
      "127.0.0.1",
      "--port",
      String(port),
      "--inspector-port",
      String(port + 500),
      "--persist-to",
      path.join(directory, "storage"),
    ],
    {
      cwd: directory,
      detached: true,
      stdio: ["ignore", "pipe", "pipe"],
      env: {
        ...process.env,
        WRANGLER_SEND_METRICS: "false",
        CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV: "false",
        XDG_CONFIG_HOME: path.join(directory, "config"),
        WRANGLER_LOG_PATH: path.join(directory, "wrangler.log"),
      },
    },
  );
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
  try {
    let response;
    for (let i = 0; i < 300; i++) {
      if (stopped) throw new Error("workerd exited before becoming ready");
      try {
        response = await fetch(origin + ready, {
          signal: AbortSignal.timeout(1000),
        });
        if (response.ok) break;
      } catch {}
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    if (!response?.ok)
      throw new Error("workerd did not become ready within 30 seconds");
    await test(origin);
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
    // A provider wrapper can exit before its remaining child processes.
    try {
      process.kill(-child.pid, "SIGKILL");
    } catch {}
    await writeFile(path.join(directory, "runtime.log"), log);
  }
}

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";
import * as esbuild from "esbuild";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const DIST = path.join(__dirname, "dist");

if (!fs.existsSync(DIST)) {
  fs.mkdirSync(DIST, { recursive: true });
}

console.log("Building Playground Web Bundle...");

// 1. Bundle App JS
await esbuild.build({
  entryPoints: [path.join(__dirname, "src/app.ts")],
  bundle: true,
  outfile: path.join(DIST, "app.js"),
  format: "esm",
  target: ["es2022"],
  minify: true,
  sourcemap: true,
});

// 2. Bundle Worker JS
await esbuild.build({
  entryPoints: [path.join(__dirname, "src/compiler-worker.ts")],
  bundle: true,
  outfile: path.join(DIST, "compiler-worker.js"),
  format: "esm",
  target: ["es2022"],
  minify: true,
  sourcemap: true,
});

// 3. Copy/Bundle CSS
fs.copyFileSync(path.join(__dirname, "src/styles.css"), path.join(DIST, "styles.css"));

// 4. Copy WASM binary if exists in target/
const wasmSrc = path.join(ROOT, "target/wasm32-unknown-unknown/release/rocci_playground_wasm.wasm");
const wasmDst = path.join(DIST, "compiler.wasm");
if (fs.existsSync(wasmSrc)) {
  fs.copyFileSync(wasmSrc, wasmDst);
}

// 5. Generate Standalone index.html
const indexHtmlContent = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Rocci & Rocdown Playground</title>
  <link rel="stylesheet" href="./styles.css">
</head>
<body>
  <div id="playground-root"></div>
  <script type="module">
    import { PlaygroundApp } from "./app.js";

    const bootstrap = {
      protocol_version: 1,
      documents: [
        {
          id: "counter",
          filename: "Counter.rocci",
          language: "rocci",
          source: "@component Counter = |{ count }| {\\n  <button class=\\"btn\\">{count}</button>\\n}"
        },
        {
          id: "guide",
          filename: "Guide.rocdown",
          language: "rocdown",
          source: "# Welcome to Rocdown\\n\\nThis is a **live** document compiled in WASM."
        }
      ],
      selected_document: "counter",
      compiler_wasm_url: "./compiler.wasm",
      worker_url: "./compiler-worker.js",
      mode: "wasm",
      compile_url: "",
      native_languages: [],
      html_runtime: {
        available: false,
        reason: "HTML preview is not available in WASM mode. The browser cannot dynamically compile generated Roc to WebAssembly."
      }
    };

    const root = document.getElementById("playground-root");
    new PlaygroundApp({ container: root, bootstrap });
  </script>
</body>
</html>`;

fs.writeFileSync(path.join(DIST, "index.html"), indexHtmlContent, "utf-8");

// 6. Generate Manifest
function sha256File(filepath) {
  if (!fs.existsSync(filepath)) return null;
  const buf = fs.readFileSync(filepath);
  return crypto.createHash("sha256").update(buf).digest("hex");
}

const manifest = {
  version: 1,
  generated_at: new Date().toISOString(),
  files: {},
};

for (const name of ["app.js", "compiler-worker.js", "styles.css", "compiler.wasm", "index.html"]) {
  const filePath = path.join(DIST, name);
  if (fs.existsSync(filePath)) {
    const stat = fs.statSync(filePath);
    manifest.files[name] = {
      path: name,
      size: stat.size,
      sha256: sha256File(filePath),
    };
  }
}

fs.writeFileSync(path.join(DIST, "manifest.json"), JSON.stringify(manifest, null, 2), "utf-8");

console.log("Playground build complete. Manifest:");
console.log(JSON.stringify(manifest, null, 2));

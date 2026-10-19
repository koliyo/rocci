import { build } from 'esbuild';
import { readFile, mkdir } from 'node:fs/promises';
import { createRequire } from 'node:module';
await mkdir('test-dist', { recursive: true });
await build({ stdin: { contents: `export * from './src/protocol'; export * from './src/roc'; export * from './src/streams'; export * from './src/auth';`, resolveDir: process.cwd() },
  bundle: true, platform: 'node', format: 'esm', outfile: `test-dist/host-${process.pid}.mjs`, plugins: [{ name: 'core-wasm', setup(b) {
    b.onLoad({ filter: /\.wasm$/ }, async args => ({ contents: `export default new WebAssembly.Module(Uint8Array.from(atob('${(await readFile(args.path)).toString('base64')}'), c=>c.charCodeAt(0)));`, loader: 'js' }));
  } }] });
export const host = await import(`../test-dist/host-${process.pid}.mjs`);
export const require = createRequire(import.meta.url);

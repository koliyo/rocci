import { mkdir, readFile, writeFile, copyFile, readdir, stat, rename, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const dir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const root = path.resolve(dir, '../../../..');
// Keep compiler caches inside the writable derived build tree.
process.env.XDG_CACHE_HOME = path.join(root, 'target/actor-cache');
export const pins = { roc: 'nightly-2026-10-09-258ab27', clang: '23.1.3', node: 'v26.12.0', abi: 1, schema: 1, datastar: '1.0.2', compatibilityDate: '2026-10-19' };
const roc = process.env.ROC ?? 'roc';
const clang = process.env.CLANG ?? '/opt/homebrew/opt/llvm/bin/clang';
function run(cmd, args, cwd = root) {
  const r = spawnSync(cmd, args, { cwd, encoding: 'utf8', env: process.env });
  if (r.status !== 0) throw new Error(`${cmd} ${args.join(' ')}\n${r.stderr}\n${r.stdout}`);
  return (r.stdout + r.stderr).trim();
}
if (!run(roc, ['version']).includes(pins.roc)) throw new Error(`Use Roc ${pins.roc}; root nightly has a Wasm List.concat trap`);
if (!run(clang, ['--version']).includes(pins.clang)) throw new Error(`Use Clang ${pins.clang}`);
if (process.version !== pins.node) throw new Error(`Use Node ${pins.node}`);
const stage = path.join(dir, 'dist.next');
await rm(stage, { recursive: true, force: true });
await mkdir(stage, { recursive: true });
try {
  console.log('Lowering shared Rocci components and staging pure modules');
  run('cargo', ['run', '-q', '-p', 'rocci-cli', '--example', 'actor_inputs', '--', stage]);
  await copyFile(path.join(dir, 'Actor.roc'), path.join(stage, 'Actor.roc'));
  await mkdir(path.join(stage, 'platform/targets/wasm32'), { recursive: true });
  await copyFile(path.join(dir, 'platform/main.roc'), path.join(stage, 'platform/main.roc'));
  await writeFile(path.join(stage, 'main.roc'), 'app [call] { pf: platform "platform/main.roc" }\nimport Actor\ncall = Actor.call\n');
  const host = path.join(dir, 'platform/host.c');
  run(clang, ['--target=wasm32', '-ffreestanding', '-fno-builtin', '-O2', '-c', host, '-o', path.join(stage, 'platform/targets/wasm32/host.o')]);
  console.log('Compiling independent core Wasm');
  run(roc, ['build', '--no-cache', '--target=wasm32', '--wasm-stack-size=1048576', `--output=${stage}/actor.wasm`, `${stage}/main.roc`]);
  const nativeTarget = process.platform === 'darwin' && process.arch === 'arm64' ? 'arm64mac' : process.platform === 'linux' && process.arch === 'x64' ? 'x64glibc' : undefined;
  if (!nativeTarget) throw new Error('Native parity currently supports Apple Silicon macOS or x64 Linux');
  await mkdir(path.join(stage, `platform/targets/${nativeTarget}`), { recursive: true });
  run(clang, ['-ffreestanding', '-fno-builtin', '-O2', '-c', host, '-o', path.join(stage, `platform/targets/${nativeTarget}/host.o`)]);
  run(roc, ['build', '--no-cache', `--target=${nativeTarget}`, `--output=${stage}/native`, `${stage}/main.roc`]);
  const asset = process.env.DATASTAR_JS ?? path.join(process.env.HOME, '.rocci/cache/datastar/v1.0.2/datastar.js');
  const datastar = await readFile(asset);
  if (!datastar.toString().slice(0, 512).includes('1.0.2')) throw new Error('Supply pinned Datastar 1.0.2 using DATASTAR_JS');
  await writeFile(path.join(stage, 'assets/datastar.js'), datastar);
  for (const file of await readdir(path.join(dir, 'assets'))) await copyFile(path.join(dir, 'assets', file), path.join(stage, 'assets', file));
  const module = new WebAssembly.Module(await readFile(path.join(stage, 'actor.wasm')));
  if (WebAssembly.Module.imports(module).length) throw new Error('Core Wasm unexpectedly has imports');
  const hashes = {};
  async function hash(name, filename) { hashes[name] = createHash('sha256').update(await readFile(filename)).digest('hex'); }
  for (const file of await readdir(stage)) if ((await stat(path.join(stage, file))).isFile()) await hash(file, path.join(stage, file));
  for (const file of await readdir(path.join(stage, 'assets'))) await hash(`assets/${file}`, path.join(stage, 'assets', file));
  for (const file of ['Actor.roc', 'platform/main.roc', 'platform/host.c', 'package-lock.json',
    ...['build', 'configure', 'package', 'fingerprint', 'test', 'deploy'].map(f => `tools/${f}.mjs`),
    ...((await readdir(path.join(dir, 'src'))).map(f => `src/${f}`)),
    ...((await readdir(path.join(dir, 'assets'))).map(f => `assets/${f}`))]) await hash(`source/${file}`, path.join(dir, file));
  await hash('generator/crates/rocci-cli/examples/actor_inputs.rs', path.join(root, 'crates/rocci-cli/examples/actor_inputs.rs'));
  await hash('generator/Cargo.lock', path.join(root, 'Cargo.lock'));
  for (const file of ['CounterCore.roc', 'Origin.roc', 'LiveCounterUi.rocci', 'CounterDocument.rocci']) await hash(`shared/${file}`, path.join(root, 'examples/rocci/standalone/live-counter', file));
  const manifest = { pins, sourceRevision: run('git', ['rev-parse', 'HEAD']), sourceDirty: !!run('git', ['status', '--porcelain']), hashes,
    imports: WebAssembly.Module.imports(module), exports: WebAssembly.Module.exports(module), deployment: null };
  await writeFile(path.join(stage, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  // Preserve the previous successful output on any build failure.
  await rm(path.join(dir, 'dist.previous'), { recursive: true, force: true });
  try { await rename(path.join(dir, 'dist'), path.join(dir, 'dist.previous')); } catch (e) { if (e.code !== 'ENOENT') throw e; }
  await rename(stage, path.join(dir, 'dist'));
  console.log('Built dist/actor.wasm, dist/native, assets and input/artifact manifest');
} catch (error) { await rm(stage, { recursive: true, force: true }); throw error; }

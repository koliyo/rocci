import { readFile, writeFile, readdir, rm, rename } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { configure, dir } from './configure.mjs';
await configure(false);
const next = path.join(dir, 'dist/bundle.next');
await rm(next, { recursive: true, force: true });
const r = spawnSync(process.execPath, ['node_modules/wrangler/bin/wrangler.js', 'deploy', '--dry-run', '--config', 'dist/wrangler.json', '--outdir', next], {
  cwd: dir, stdio: 'inherit', env: { ...process.env, WRANGLER_SEND_METRICS: 'false',
    WRANGLER_LOG_PATH: path.join(dir, '.wrangler/logs/package.log'), XDG_CONFIG_HOME: path.join(dir, '.wrangler/config') },
});
if (r.status !== 0) process.exit(r.status ?? 1);
await readFile(path.join(next, 'index.js')); // Fail closed if Wrangler did not emit the entrypoint.
await rm(path.join(dir, 'dist/bundle.previous'), { recursive: true, force: true });
try { await rename(path.join(dir, 'dist/bundle'), path.join(dir, 'dist/bundle.previous')); } catch (e) { if (e.code !== 'ENOENT') throw e; }
await rename(next, path.join(dir, 'dist/bundle'));
const hashes = {};
for (const file of await readdir(path.join(dir, 'dist/bundle'))) {
  const bytes = await readFile(path.join(dir, 'dist/bundle', file));
  hashes[file] = createHash('sha256').update(bytes).digest('hex');
}
hashes['../wrangler.json'] = createHash('sha256').update(await readFile(path.join(dir, 'dist/wrangler.json'))).digest('hex');
await writeFile(path.join(dir, 'dist/package.json'), JSON.stringify({ hashes }, null, 2) + '\n');
console.log('Packaged immutable tested-artifact candidate; deploy does not rebuild it');

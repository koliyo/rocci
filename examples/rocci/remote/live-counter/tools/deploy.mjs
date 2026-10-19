import { readFile, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { dir, settings } from './configure.mjs';
import path from 'node:path';
import { fingerprint, hash } from './fingerprint.mjs';
const values = await settings(true);
const config = JSON.parse(await readFile(path.join(dir, 'dist/wrangler.json'), 'utf8'));
if (config.name !== values.WORKER_NAME || config.vars.COUNTER_HOSTNAME !== values.COUNTER_HOSTNAME
  || config.vars.ACCESS_AUD !== values.ACCESS_AUD || config.vars.ACCESS_TEAM_DOMAIN !== values.ACCESS_TEAM_DOMAIN
  || config.vars.OWNER_EMAIL !== values.OWNER_EMAIL || config.account_id !== values.CLOUDFLARE_ACCOUNT_ID
  || config.vars.COUNTER_KEYS !== values.COUNTER_KEYS || config.vars.STORAGE_SCHEMA !== values.STORAGE_SCHEMA)
  throw new Error('Configuration changed; configure, package and test the final artifact before deployment');
const pkg = JSON.parse(await readFile(path.join(dir, 'dist/package.json'), 'utf8'));
for (const [file, expected] of Object.entries(pkg.hashes)) {
  const actual = createHash('sha256').update(await readFile(path.join(dir, 'dist/bundle', file))).digest('hex');
  if (actual !== expected) throw new Error(`Packaged artifact changed: ${file}`);
}
const tested = JSON.parse(await readFile(path.join(dir, 'dist/tested.json'), 'utf8'));
if (hash(JSON.stringify(tested.files)) !== hash(JSON.stringify(await fingerprint())))
  throw new Error('Package, source, configuration or assets changed after conformance; package and test again');
// No rebuild here. Only the packaged JavaScript and core Wasm are uploaded.
const result = spawnSync(process.execPath, ['node_modules/wrangler/bin/wrangler.js', 'deploy', path.join(dir, 'dist/bundle/index.js'), '--no-bundle', '--config', 'dist/wrangler.json', '--durable-objects-code-update-mode', 'immediate'], {
  cwd: dir, env: { ...process.env, CLOUDFLARE_ACCOUNT_ID: values.CLOUDFLARE_ACCOUNT_ID,
    ...(values.CLOUDFLARE_API_TOKEN ? { CLOUDFLARE_API_TOKEN: values.CLOUDFLARE_API_TOKEN } : {}) }, encoding: 'utf8',
});
process.stdout.write(result.stdout ?? ''); process.stderr.write(result.stderr ?? '');
if (result.status !== 0) process.exit(result.status ?? 1);
await writeFile(path.join(dir, 'dist/deployment.txt'), result.stdout + result.stderr);

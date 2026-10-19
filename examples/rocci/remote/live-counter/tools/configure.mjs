import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { parseEnv } from 'node:util';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
export const dir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export async function settings(required = false) {
  let local = {};
  try { local = parseEnv(await readFile(path.join(dir, '.env'), 'utf8')); }
  catch (error) { if (required || error.code !== 'ENOENT') throw error; }
  const defaults = parseEnv(await readFile(path.join(dir, '.env.example'), 'utf8'));
  const values = { ...defaults, ...local };
  for (const key of Object.keys(defaults)) if (process.env[key]) values[key] = process.env[key];
  if (!/^(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+rocci\.dev$/.test(values.COUNTER_HOSTNAME)) throw new Error('Use a dedicated staging subdomain of rocci.dev');
  if (!/^[a-z][a-z0-9-]{2,50}$/.test(values.WORKER_NAME)) throw new Error('Invalid stable Worker name');
  if (required && (!/^[a-f0-9]{32}$/.test(values.CLOUDFLARE_ACCOUNT_ID)
    || !/^[a-z0-9-]+\.cloudflareaccess\.com$/.test(values.ACCESS_TEAM_DOMAIN)
    || !/^[a-f0-9]{64}$/.test(values.ACCESS_AUD) || !values.OWNER_EMAIL.includes('@')))
    throw new Error('Complete Account ID, team domain, application AUD and owner email in .env; see README dashboard walkthrough');
  return values;
}
export async function configure(required = false) {
  const values = await settings(required);
  const config = {
    name: values.WORKER_NAME, main: '../src/index.ts', compatibility_date: '2026-10-19',
    compatibility_flags: ['enable_request_signal', 'request_signal_passthrough'],
    workers_dev: false, preview_urls: false,
    find_additional_modules: true,
    rules: [{ type: 'CompiledWasm', globs: ['**/*.wasm'], fallthrough: true }],
    routes: [{ pattern: values.COUNTER_HOSTNAME, custom_domain: true }],
    assets: { directory: './assets', binding: 'ASSETS', run_worker_first: true, html_handling: 'none' },
    durable_objects: { bindings: [{ name: 'COUNTERS', class_name: 'Counter' }] },
    exports: { Counter: { type: 'durable-object', storage: 'sqlite' } },
    version_metadata: { binding: 'CF_VERSION_METADATA' },
    vars: {
      COUNTER_HOSTNAME: values.COUNTER_HOSTNAME, ACCESS_TEAM_DOMAIN: values.ACCESS_TEAM_DOMAIN || 'unconfigured.cloudflareaccess.com',
      ACCESS_AUD: values.ACCESS_AUD || 'unconfigured', OWNER_EMAIL: values.OWNER_EMAIL,
      COUNTER_KEYS: values.COUNTER_KEYS, STORAGE_SCHEMA: values.STORAGE_SCHEMA,
    },
  };
  if (values.CLOUDFLARE_ACCOUNT_ID) config.account_id = values.CLOUDFLARE_ACCOUNT_ID;
  await mkdir(path.join(dir, 'dist'), { recursive: true });
  await writeFile(path.join(dir, 'dist/wrangler.json'), JSON.stringify(config, null, 2) + '\n');
  await writeFile(path.join(dir, 'dist/wrangler.local.json'), JSON.stringify({ ...config,
    name: 'rocci-counter-local-fixture', main: '../test/fixture.ts', routes: [],
    vars: { ...config.vars, OWNER_EMAIL: 'fixture@example.invalid' },
  }, null, 2) + '\n');
  console.log(`Configured ${config.name} for ${values.COUNTER_HOSTNAME}; credentials excluded from generated files`);
  return config;
}
if (process.argv[1] === fileURLToPath(import.meta.url)) await configure(true);

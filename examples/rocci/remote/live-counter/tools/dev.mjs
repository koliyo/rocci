import { spawn } from 'node:child_process';
import path from 'node:path';
import { configure, dir } from './configure.mjs';
await configure(false);
const child = spawn(process.execPath, ['node_modules/wrangler/bin/wrangler.js', 'dev', '--config', 'dist/wrangler.local.json', '--ip', '127.0.0.1', ...process.argv.slice(2)], {
  cwd: dir, stdio: 'inherit', env: { ...process.env, CLOUDFLARE_LOAD_DEV_VARS_FROM_DOT_ENV: 'false',
    WRANGLER_SEND_METRICS: 'false', WRANGLER_LOG_PATH: path.join(dir, '.wrangler/logs/dev.log'), XDG_CONFIG_HOME: path.join(dir, '.wrangler/config') },
});
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal));
child.on('exit', code => process.exit(code ?? 0));

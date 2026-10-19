import { readFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { dir } from './configure.mjs';
export const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export async function fingerprint() {
  const manifest = JSON.parse(await readFile(path.join(dir, 'dist/manifest.json'), 'utf8'));
  for (const [key, expected] of Object.entries(manifest.hashes)) {
    const source = key.startsWith('source/') ? path.join(dir, key.slice(7))
      : key.startsWith('shared/') ? path.resolve(dir, '../../standalone/live-counter', key.slice(7))
      : key.startsWith('generator/') ? path.resolve(dir, '../../../..', key.slice(10)) : undefined;
    if (source && hash(await readFile(source)) !== expected) throw new Error(`Build input changed: ${key}; rebuild, package and test`);
  }
  const files = ['dist/actor.wasm', 'dist/manifest.json', 'dist/wrangler.json', 'dist/package.json', 'package-lock.json'];
  for (const folder of ['src', 'dist/assets', 'dist/bundle'])
    for (const file of await readdir(path.join(dir, folder))) files.push(`${folder}/${file}`);
  const result = {};
  for (const file of files.sort()) result[file] = hash(await readFile(path.join(dir, file)));
  return result;
}

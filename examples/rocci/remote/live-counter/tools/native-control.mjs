// Opt-in smoke against the native HTTP origin and its disposable SQLite file.
import { DatabaseSync } from 'node:sqlite';
import assert from 'node:assert/strict';
const [origin, filename] = process.argv.slice(2);
if (!origin || !filename) throw new Error('Usage: node tools/native-control.mjs HTTP_ORIGIN DISPOSABLE_DB_PATH');
const db = new DatabaseSync(filename);
const row = () => db.prepare('SELECT value,revision FROM counter WHERE id=1').get();
const clicks = () => db.prepare('SELECT COUNT(*) AS n FROM clicks').get().n;
const post = (path, datastar = false) => fetch(new URL(`/actions/counter/${path}`, origin), { method: 'POST', headers: {
  'Content-Type': 'application/json', ...(datastar ? { 'Datastar-Request': 'true' } : {}),
}, body: JSON.stringify({ origin: 'Europe/Oslo', tz: '' }) });
try {
  const before = row();
  assert.equal((await post('increment')).status, 204);
  assert.equal(row().value, before.value + 1);
  const accepted = row(), count = clicks();
  db.exec("CREATE TRIGGER native_control_rollback BEFORE INSERT ON clicks BEGIN SELECT RAISE(ABORT,'native control rollback'); END");
  try { assert.equal((await post('increment')).status, 500); assert.deepEqual(row(), accepted); assert.equal(clicks(), count); }
  finally { db.exec('DROP TRIGGER native_control_rollback'); }
  const reset = await post('reset', true);
  assert.equal(reset.status, 200); assert.equal(await reset.text(), '');
  assert.equal(row().value, 0); assert.equal(clicks(), count);
  const page = await (await fetch(origin)).text();
  assert.match(page, /<output[^>]*>0<\/output>/); assert.ok(page.includes('Oslo, Norway'));
  console.log('Native control passed: shared core, finite command responses, reset/feed and atomic rollback');
} finally { db.close(); }

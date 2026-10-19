import test from 'node:test';
import assert from 'node:assert/strict';
import { generateKeyPair, exportJWK, SignJWT } from 'jose';
import { host } from './support.mjs';
test('strict JSON envelope, exact integers, field bounds and canonical retry digest', async () => {
  const id = crypto.randomUUID();
  const c = { v: 1, id, command: 'increment', origin: '', tz: 'Europe/Oslo' };
  assert.deepEqual(host.command(c), c);
  assert.equal(await host.checksum(host.canonical(c)), await host.checksum(host.canonical({ ...c, id: crypto.randomUUID() })));
  for (const invalid of [{ ...c, other: 1 }, { ...c, v: 2 }, { ...c, id: 'no' }, { ...c, tz: '🌍'.repeat(33) }, { ...c, origin: '\ud800' }, { v: 1, id, command: 'reset', tz: '' }]) assert.throws(() => host.command(invalid));
  for (const v of ['-1', '01', '9007199254740992', 42, '1e2']) assert.throws(() => host.integer(v));
  for (const [value, status] of [[new Uint8Array([255]), 400], [new TextEncoder().encode('{'), 400], [new Uint8Array(8193), 413]]) {
    await assert.rejects(host.body(new Request('http://localhost', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: value })), e => e.status === status);
  }
  assert.throws(() => host.csrf(new Request('https://counter.example/command', { method: 'POST', headers: { Origin: 'https://evil.example' } })), e => e.status === 403);
});
test('Access JWT requires signature, issuer, audience, expiry and exact owner', async t => {
  const { privateKey, publicKey } = await generateKeyPair('RS256');
  const jwk = { ...await exportJWK(publicKey), kid: 'fixture', alg: 'RS256', use: 'sig' };
  t.mock.method(globalThis, 'fetch', async url => {
    assert.equal(String(url), 'https://fixture.cloudflareaccess.com/cdn-cgi/access/certs');
    return Response.json({ keys: [jwk] });
  });
  const env = { COUNTER_HOSTNAME: 'counter-private.rocci.dev', ACCESS_TEAM_DOMAIN: 'fixture.cloudflareaccess.com', ACCESS_AUD: 'fixture-audience', OWNER_EMAIL: 'nils.hjelte@gmail.com' };
  const issue = overrides => new SignJWT({ email: env.OWNER_EMAIL, ...overrides }).setProtectedHeader({ alg: 'RS256', kid: 'fixture' }).setIssuer(`https://${env.ACCESS_TEAM_DOMAIN}`).setAudience(env.ACCESS_AUD).setSubject('owner-subject').setIssuedAt().setExpirationTime(overrides.exp ?? '1h').sign(privateKey);
  const request = token => new Request('https://counter-private.rocci.dev/assets/datastar.js', { headers: { 'Cf-Access-Jwt-Assertion': token, 'Cf-Access-Authenticated-User-Email': env.OWNER_EMAIL } });
  assert.equal(await host.authenticate(request(await issue({})), env), 'owner-subject');
  for (const token of ['', 'forged', await issue({ email: 'someone@example.com' }), await issue({ exp: 1 }), await new SignJWT({ email: env.OWNER_EMAIL }).setProtectedHeader({ alg: 'RS256', kid: 'fixture' }).setIssuer('https://wrong.cloudflareaccess.com').setAudience(env.ACCESS_AUD).setSubject('owner').setIssuedAt().setExpirationTime('1h').sign(privateKey)]) {
    await assert.rejects(host.authenticate(request(token), env));
  }
  await assert.rejects(host.authenticate(request(await issue({})), { ...env, ACCESS_AUD: 'wrong-app' }));
  await assert.rejects(host.authenticate(new Request('https://alternate.workers.dev/'), env), e => e.status === 403);
});
test('SSE coalesces a stalled reader, rejects stale durable completions and cleans timers', async () => {
  const snapshot = revision => ({ state: { count: String(revision), revision: String(revision), clicks: [] }, html: `<section id="counter">${revision}</section><div id="counter-feed">\r\nevent: evil</div>` });
  const streams = new host.Streams(() => snapshot(100));
  const a = streams.subscribe(), b = streams.subscribe();
  streams.publish(snapshot(1));
  const r = b.response.body.getReader();
  const first = new TextDecoder().decode((await r.read()).value);
  assert.ok(first.includes('id: 1:1'));
  for (let i = 2; i <= 100; i++) streams.publish(snapshot(i));
  streams.publish(snapshot(2));
  assert.equal(streams.latest.state.revision, '100');
  streams.publish({ ...snapshot(100), order: 20 });
  streams.publish({ ...snapshot(100), order: 19, html: 'obsolete presentation' });
  assert.equal(streams.latest.order, 20);
  const last = new TextDecoder().decode((await r.read()).value);
  assert.ok(last.includes('id: 100:2')); assert.ok(!last.includes('\nevent: evil'));
  await r.cancel(); await a.response.body.cancel();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(streams.size, 0);
});
test('replacement fences every old stream at the shared heartbeat and stops both timers', async t => {
  t.mock.timers.enable({ apis: ['setInterval'] });
  const snapshot = { state: { count: '1', revision: '1', clicks: [] }, html: '<div id="counter">1</div><div id="counter-feed"></div>' };
  let replaced = false, checks = 0;
  const streams = new host.Streams(() => snapshot, () => {
    checks++;
    if (replaced) throw new Error('Actor no longer owns storage');
  });
  const a = streams.subscribe().response.body.getReader(), b = streams.subscribe().response.body.getReader();
  streams.publish(snapshot);
  await Promise.all([a.read(), b.read()]);
  t.mock.timers.tick(15000);
  assert.equal(checks, 1);
  await Promise.all([a.read(), b.read()]);
  replaced = true;
  t.mock.timers.tick(15000);
  assert.equal(streams.size, 0);
  await Promise.all([assert.rejects(a.read()), assert.rejects(b.read())]);
  t.mock.timers.tick(60000);
  assert.equal(checks, 2);
});

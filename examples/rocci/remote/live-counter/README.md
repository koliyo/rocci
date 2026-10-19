# Private actor live-counter experiment

Experimental implementation of the [portable actor backend plan](../../../../knowledge/plans/rocci/portable-actor-backends.md).
The independent Roc/core-Wasm gate precedes storage, streams, and deployment.
Local results and hosted acceptance results must be recorded separately.

## Cloudflare preparation required from the owner

Prepare this before the first hosted probe. No account resources or billing
settings have been changed by adding these instructions.

Use the existing account signed in as `nils.hjelte@gmail.com` and the existing
`rocci.dev` zone. The selected demo address is `counter-private.worker.rocci.dev`.
This is an additional subdomain; the public rocci.dev site keeps its routing.

| Setting | What it does | Where you find it |
| --- | --- | --- |
| `CLOUDFLARE_ACCOUNT_ID` | Selects the account that owns the uploaded Worker and its data | Account dashboard / Workers & Pages account details; copy Account ID |
| `CLOUDFLARE_API_TOKEN` | Lets the local deployment tool upload code and configure the staging domain | Profile → API Tokens → Create Token |
| `COUNTER_HOSTNAME` | The browser address and permitted HTTP origin | Use `counter-private.worker.rocci.dev` |
| `ACCESS_TEAM_DOMAIN` | Identifies your Cloudflare login service and its JWT signing keys | Cloudflare One / Zero Trust team settings; looks like `your-team.cloudflareaccess.com` |
| `ACCESS_AUD` | Identifies this Access application, so a token for another app is rejected | Access → Applications → this app → application details / AUD tag |
| `OWNER_EMAIL` | Limits valid authenticated users to you | Use `nils.hjelte@gmail.com`, matching the identity used to sign in |
| `WORKER_NAME` | Stable name for the uploaded program and its object namespace | Keep the example value `rocci-counter-private` |
| `COUNTER_KEYS` | Two independent counters for the isolation test | Keep `demo,isolation` |

The account ID, hostname, team domain, and AUD are identifiers, not passwords.
The API token is a secret. The Gmail address is the allowed identity, not a
Google API credential. You do not need to set up a Google OAuth project:
Cloudflare's email one-time PIN can authenticate that address.

### Dashboard walkthrough

1. Sign in to [the Cloudflare dashboard](https://dash.cloudflare.com/) using
   `nils.hjelte@gmail.com`. Select the account containing **rocci.dev**.
   Open **Workers & Pages** and copy **Account ID** into `.env`.
2. Open **Zero Trust** (also called **Cloudflare One**). If this is the first
   visit, create the team, choose its team name, and select an available free
   plan within the existing account. If setup asks for a payment method or
   paid upgrade, pause there and review it; no upgrade is assumed. Record
   the resulting `TEAM.cloudflareaccess.com` domain in `.env`.
3. Under **Settings → Authentication → Login methods**, enable email
   **One-time PIN**, or use an identity provider you already configured.
4. Go to **Access → Applications → Add an application → Self-hosted**.
   Name it `Rocci private counter`. Add public hostname
   `counter-private.worker.rocci.dev`; leave the path blank to cover every path.
   Pick a session duration appropriate for the demo (for example one hour).
5. Add an **Allow** policy named `Nils only`. Set **Include → Emails** to
   `nils.hjelte@gmail.com`. Save the policy and application. Do not select
   **Emails ending in** or **Everyone**. Copy the app's **AUD Tag** into
   `.env` as `ACCESS_AUD` (copy the tag, not a JWT from a browser cookie).
6. At your avatar / **My Profile → API Tokens → Create Token**, choose
   **Edit Cloudflare Workers**. Restrict account resources to this account
   and zone resources to `rocci.dev`. Save the token once in the ignored
   `.env` as `CLOUDFLARE_API_TOKEN`. Do not grant Access administration merely
   for deployment; the Access application above is configured by you.
7. Keep the dedicated hostname free for the Worker's Custom Domain. The
   deploy tooling will attach it and request the certificate. There is no
   server IP address to enter and no manual D1 database to create.
8. After deployment, open `https://counter-private.worker.rocci.dev/` in both browser
   profiles. With one-time PIN, enter your Gmail address, read the email,
   and enter its code. Use a signed-out profile to verify denial too.

Dashboard labels can vary by account. The linked Cloudflare instructions
below are the provider reference. Account setup enables deployment; filling
`.env` alone does not create an Access policy or protect a hostname.

### Local configuration

Copy `.env.example` to `.env` **in this directory** and fill the blank values.
The tools read it directly; do not `source` it or paste its contents into chat.
Keep `WORKER_NAME` and both counter keys stable after the first deployment:
they identify retained data. There are no runtime secrets to upload.

A deployment API token can upload Workers without permission to read Access
applications. An empty Access application list from that token is **not proof
that no application exists**: direct reads may return `403 Forbidden`. Verify
in the dashboard instead, or optionally grant the narrow Access applications
read permission for automated inspection. Access edit permission is unnecessary.
On October 19 the dashboard confirmed the configured application's hostname,
matching AUD and exact-email Allow policy; the deployment token could not
read that application. No Access configuration change was needed.

No D1 database, R2 bucket, KV namespace, Tunnel, or origin VM is required.
The Durable Object's SQLite namespace is provisioned by the class declaration
on first deployment; it is separate from application SQL migrations. Keep
script/class/namespace identity stable on later updates.

## Required deployment controls

The deployment configuration must keep `workers_dev: false`,
`preview_urls: false`, and `assets.run_worker_first: true`. The Worker must
validate the Access JWT signature, issuer, audience, expiry, and owner email
before serving documents, commands, streams, export/import, or assets.
The hostname and same-origin mutation checks must fail closed. These controls
are implementation acceptance requirements, not evidence of a hosted test.

Before data tests, verify signed-out denial on `/`, `/sse`, both command paths,
operator paths, and `/assets/datastar.js`. Inspect **Workers → Settings →
Domains & Routes** and test every listed hostname, including old workers.dev
and version preview URLs. See [workers.dev controls](https://developers.cloudflare.com/workers/configuration/routing/workers-dev/),
[preview controls](https://developers.cloudflare.com/workers/versions-and-deployments/version-urls/),
and [Worker-first assets](https://developers.cloudflare.com/workers/static-assets/routing/worker-script/).

## Build, test and deploy

Use Node **26.12.0**, Roc **nightly-2026-10-09-258ab27**, Clang **23.1.3**,
Rust/Cargo and the committed npm lockfile. `ROC` selects the actor compiler;
`CLANG` overrides the Apple Silicon default `/opt/homebrew/opt/llvm/bin/clang`.
Native parity supports Apple Silicon macOS and x64 Linux. The repository's
September 18 Roc pin stays unchanged for the ordinary native server: that
compiler traps in this independent Wasm workload. This private experiment
has its own October 9 pin; it is not a supported general Rocci target.

From this directory:

```sh
cp .env.example .env  # first setup only; preserve an existing .env
npm ci --ignore-scripts
ROC=/path/to/nightly-2026-10-09-258ab27/roc npm run build
npm run check
npm run package
npm test
npm run deploy
```

The build uses `cargo run -p rocci-cli --example actor_inputs` to lower the
actual shared `.rocci` components, stages pure Html/Datastar/Origin modules,
and compiles native and import-free core Wasm. It reads pinned Datastar 1.0.2
from `~/.rocci/cache/datastar/v1.0.2/datastar.js`; set `DATASTAR_JS` to another
local copy of that version if needed. Build failures preserve the previous
successful `dist/`. Nothing compiles inside a request handler.

`package` validates settings and creates the exact Wrangler upload bundle.
`test` runs ABI/native parity, browser retry behavior, JWT/protocol/stream
checks and actual local workerd/SQLite fault and recovery scenarios. Tests
stamp the package/config/assets/source fingerprint in `dist/tested.json`.
`deploy` refuses changed or untested artifacts and uploads that bundle with
`--no-bundle`; it never rebuilds it. After changes, rebuild/package/test again.
Deployment explicitly requests immediate Durable Object code replacement,
so open streams reconnect instead of indefinitely retaining an older actor
version. A reconnect always receives a current durable snapshot.
Generated configuration excludes the API token. Source, generated Roc,
Wasm, assets, tool pins and ABI/schema versions are hashed in
`dist/manifest.json`; bundle hashes are in `dist/package.json`. Deployment
output/version is retained in `dist/deployment.txt`. These are derived,
ignored artifacts. Save them with acceptance evidence before rebuilding.

The first deploy provisions the SQLite object class through Wrangler's pinned
**declarative class exports** configuration. Later ordinary code deployments
keep the same namespace. Application schema upgrades are a separate concern.
Cloudflare creates a certificate for the Custom Domain, including this nested
subdomain; no Advanced Certificate Manager purchase is needed. Certificate
issuance can take time after upload: a TLS handshake failure before the
certificate is ready does not diagnose an Access policy problem. See
[Custom Domain certificates](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/#certificates).

For local interactive development:

```sh
npm run dev
```

Open the printed `http://127.0.0.1:PORT/c/demo/` URL. This uses the explicitly
local `test/fixture.ts` identity and loopback binding. It does not validate
Cloudflare login and is never deployed. The production bundle always checks
Access, including static assets. Stop local servers before running conformance.
The opt-in native HTTP/SQLite control uses the root native compiler pin:
`ROC=/path/to/root-pinned/roc node tools/native-control.mjs`.

## Owner browser acceptance

After certificate readiness, open `https://counter-private.worker.rocci.dev/`
and sign in through your existing Cloudflare identity provider. Use a second
browser/profile signed in as the same owner. On the counter, increment from
each browser, reset, close/reopen one, background/foreground it, and confirm
both count and feed converge. Reset clears the count and preserves the feed.
`/c/isolation/` must keep its independent count and revision.

Open `/assets/verify.html` on the same hostname. **Run 100-command check** resets
the demo count, runs 100 concurrent increments, retries one UUID, checks a
conflicting reuse, opens two SSE readers, and checks reconnect. It accepts
102 operations in total and ends at count 101. Keep the second browser on
the counter during this exercise. **Start 10-minute idle check** records
heartbeat continuity; minute refreshes update age labels without changing
the durable revision. **Read manifest** returns the actual deployed version,
input hashes, revision, schema, Wasm pages, peak invocation arena, viewers, SQL rows/statements and stream
frames/bytes/coalescing. SQL and stream counters are per activation and include
failed attempts where the provider reports them; they are not a billing ledger.
Save the displayed JSON with the dated acceptance record. Do not copy cookies
or JWTs into chat or artifacts.

A signed-out browser must reach Access login/denial for `/`, `/c/demo/`,
`/c/demo/sse`, `/c/demo/command`, `/c/demo/operator/export`,
`/c/demo/operator/import`, `/assets/datastar.js`, and `/assets/verify.html`.
Also test POST requests to command/import without credentials. No response
may expose a document, patch, state, export or asset. Enumerate actual domains
in Workers → Settings → Domains & Routes; test the disabled workers.dev
hostname and any version preview hostname. A spoofed email header is not a
credential. Local fixtures are not evidence of hosted authorization.

The helper functions are also available from `/assets/verify.js` for an
owner's browser console (`run`, `idle`, `api`, `exportCounter`, `restoreCounter`).
The verification page supplies buttons so console scripting is optional.

## State, transport and limits

`CounterCore.roc` owns decisions and view values. Both hosts lower and render
`LiveCounterUi.rocci` and `CounterDocument.rocci`; Origin chooses the same
labels and supplied-time age buckets. Native handlers retain native SQLite
and HTTP effects and transact count/revision/feed together. The small actor
host owns provider SQL, identity, operation receipts, routing and streams.
It does not proxy a native server or reimplement the domain decisions in JS.
The existing default native and documentation island call shapes remain valid.

Each object instantiates its own Wasm memory. The synchronous private ABI
copies input/output, uses a 4 MiB per-call arena reset on each call, rejects
oversized values and reconstructs the instance after a trap. The Wasm module
has no imports. ABI details live alongside `platform/host.c`, `Actor.roc`
and `src/roc.ts`; this is not a public runtime ABI.

Commands are JSON `{v:1,id:UUID,command:"increment",origin:STRING,tz:STRING}`
or `{v:1,id:UUID,command:"reset"}`. Increment fields are required (empty strings
are allowed). UUID identifies one browser intent. Canonical decoded payload
and verified owner must match a stored receipt on retries, or the host
returns `409`. Commit and receipt insertion are one SQL transaction; a
storage durability barrier precedes successful acknowledgement or publication.
Ordinary command responses are finite `204`; Datastar-request responses are
finite empty `200 text/event-stream`. Errors stay visible in the browser.
Retries reuse the same envelope; reload never silently creates a replacement
operation after an ambiguous outcome.

Count, revision and sequence cross JS as decimal strings, bounded by
`9007199254740991`. Reset advances revision and preserves feed; revision never
comes from count or timestamp. Retention is 50 descending feed rows. Commands
are limited to 8 KiB, origin/timezone to 128 UTF-8 bytes each, snapshots to
256 KiB, rendered HTML to 512 KiB. Each object accepts at most 10,000 operations;
receipts never silently expire. New operations then return `429`; matching
retries still work. Frozen exports reject new writes with `423`.

Each viewer gets an initial coherent count/feed snapshot, then committed
snapshots. Reconnect resynchronizes from current state rather than replaying a
log. Revision and a connection-local serial identify each SSE patch batch;
`Last-Event-ID` is not an exactly-once cursor. The host discards stale durable
completions and presentation ticks, coalesces a slow reader to one in-flight
frame plus one latest pending snapshot, and caps viewers at 20 per object.
Heartbeats run every 15 seconds; age refreshes run every minute only while
viewers exist. Cancellation clears viewers/timers, sometimes after the next
heartbeat detects the closed connection. Both incoming-request signal flags
are explicitly enabled. There is no SQL polling loop or hibernation claim for
an open SSE connection.

## Export, restore, compatible upgrade and rollback

1. On the source's `/assets/verify.html`, click **Download frozen export**.
   This fences new writes before downloading schema, state and all receipts
   with a SHA-256 checksum. Keep that JSON safely; reset does not delete receipts.
   Repeated accepted IDs can still be acknowledged while frozen.
2. For a restore drill, create a **separate** disposable Worker name, such as
   `rocci-counter-restore`, and a dedicated hostname, such as
   `counter-restore.worker.rocci.dev`. First add that exact hostname as another
   destination in the same owner-only Access application (same AUD), or create
   a separate application and use its own AUD. Use process environment overrides
   or a separate checkout with its ignored `.env`; do not accidentally overwrite
   the retained demo's settings. Package, test and deploy that configuration.
   A new Worker name/class combination creates a separate object namespace.
3. On the restore hostname's verification page, choose the export and click
   **Import into this empty demo counter**. Import validates checksum/schema,
   owner, unique UUIDs/revisions, values, sizes and receipt capacity; a nonempty
   destination returns `409`. Restore stays frozen. Compare count, feed,
   revision and receipts/export checksum. Unfreeze only after verification.
4. For a cutover, keep the source frozen, verify the target, change browser
   routing and reconnect. Do not run two writers. For rollback, fence the target
   before returning to the source. The initial exercise is a drill, not a
   production routing migration.
5. Save the schema-1 version ID from deployment output. Keeping the original
   Worker name, hostname, class and keys, set `STORAGE_SCHEMA=2` in `.env`, then
   configure/package/test/deploy. It adds `upgrade_note` and advances stored
   schema once. Reconnect, check revision/feed and retry a previously accepted
   envelope. Set `STORAGE_SCHEMA=1`, package/test/deploy again: compatible old
   code accepts stored schema 2 and preserves it. Confirm continuity again.
   This is a compatible code/config rollback, not undoing the SQL migration.
   Do not delete/recreate the class or namespace to perform it. An older
   provider version can also be selected through deployment rollback only if
   its class and schema compatibility have been checked.

Precise pre-commit SQL failure and post-commit reply loss are exercised locally
by the test fixture and SQL trigger; production has no fault-injection route.
Local actor eviction proves reconstruction from SQLite. A hosted code update
provides the managed-runtime reconstruction/reconnect drill; it is not a
claim that the operator can power-fail Cloudflare nodes on demand.

## Resource and cost review

`dist/abi-metrics.json` records repeated-call native/Node/Wasm measurements;
`dist/runtime-metrics.json` records workerd recovery scenarios. Run
`node tools/resources.mjs` for the opt-in local workerd inspector workload; it
writes `dist/resource-metrics.json` with both full feeds and 20 stalled clients
per key. Query both keys'
manifest endpoints while active. Wasm page totals and arena peaks do not measure
whole-isolate memory; Node RSS is also not workerd isolate memory. Capture a
local runtime heap profile and provider CPU/memory/SQL analytics for the
acceptance workload before declaring the plan's 64 MiB target achieved.

Keep the first hosted exercise bounded to two keys, two viewers and 10,000
accepted operations per key. SSE keeps an object resident even while viewers
are idle. Two viewers on one object share duration; two active objects accrue
separate duration. For a one-hour exercise with both objects continuously
active, the duration estimate is `2 × 3600 × 0.128 = 921.6 GB-s`; for 24 hours
it is `22,118.4 GB-s`. The latter exceeds today's Free daily duration allowance
before other account traffic. On Free, exhausted quotas reject work rather
than implying an automatic paid upgrade. Close tabs/streams after the exercise.
Compare actual account usage and include ingress Worker requests, SQL rows and
stored bytes. Recheck [current pricing](https://developers.cloudflare.com/durable-objects/platform/pricing/)
before extending the run; these estimates are not a measured bill.

## Cleanup and evidence

Close all viewer and verification tabs to release streams/timers. Retain the
private demo's Worker/class identity and data. Record its deployment ID,
namespace ID, manifest/test fingerprints, browser results and metrics in the
linked [status record](../../../../knowledge/status/portable-actor-backends.md).
Required CI and Knowledge run IDs remain distinct from local test results.

For a disposable restore/probe Worker, export first and confirm the exact
Worker/namespace/hostname before deleting it in Workers & Pages. Namespace or
class deletion can permanently erase stored counters and receipts. Remove
only that disposable hostname's Custom Domain/DNS and its Access destination;
leave the demo and unrelated policies intact. Cloudflare does not automatically
remove a Custom Domain's generated certificate when the domain is removed;
remove that unused certificate explicitly under SSL/TLS → Edge Certificates
when retiring the disposable domain. Revoke a temporary deployment token when
no longer needed. The optional celld deployment remains a later plan gate.

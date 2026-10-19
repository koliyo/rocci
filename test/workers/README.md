# Repository Worker integration tests

This directory coordinates repository integration coverage. `cases.mjs` selects
application entries and their app-owned acceptance drivers. It is not part of
the Worker platform or an application build dependency.

```sh
ROCCI_WORKER_ROC=/path/to/pinned/roc node test/workers/applications.mjs
ROCCI_WORKER_ROC=/path/to/pinned/roc node test/workers/profiles.mjs
node test/workers/packaging.mjs
```

`applications.mjs` also accepts application IDs to run selected cases. The
platform's focused async fixture and driver remain together in its `tests/`.
Example drivers live in the corresponding example's `tests/`. Reusable provider
server support and release packaging stay in the platform's `tools/`.

`hosted.mjs` is an explicit opt-in provider test coordinator. It imports each
app's hosted profile and assertions, generates generic guarded hosting and
deletes its exact temporary Worker and new storage in cleanup. It requires
`ROCCI_WORKER_CREDENTIALS` pointing to an explicitly authorized test credential
file. It is never run by the offline suite or normal application builds.

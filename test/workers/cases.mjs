/** Repository integration subjects; the Worker platform has no app registry. */
export const applications = [
  {
    id: "counter",
    entry: "examples/rocci/standalone/counter/Counter.rocci",
    test: "examples/rocci/standalone/counter/tests/worker.mjs",
    hosted: "examples/rocci/standalone/counter/tests/worker-hosted.mjs",
    profiles: ["speed", "dev", "size"],
  },
  {
    id: "live-counter",
    entry: "examples/rocci/standalone/live-counter/LiveCounter.rocci",
    test: "examples/rocci/standalone/live-counter/tests/worker.mjs",
    hosted: "examples/rocci/standalone/live-counter/tests/worker-hosted.mjs",
    profiles: ["speed"],
  },
  {
    id: "transactions",
    entry: "crates/rocci-worker-platform/examples/sql/main.roc",
    test: "crates/rocci-worker-platform/examples/sql/tests/worker.mjs",
    hosted: "crates/rocci-worker-platform/examples/sql/tests/worker-hosted.mjs",
    profiles: ["speed"],
  },
  {
    id: "async",
    entry: "crates/rocci-worker-platform/tests/Async.roc",
    test: "crates/rocci-worker-platform/tests/async.mjs",
    profiles: ["speed", "dev", "size"],
  },
];

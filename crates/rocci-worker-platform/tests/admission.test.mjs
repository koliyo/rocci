import assert from "node:assert/strict";
import test from "node:test";
import { Admission } from "../runtime/admission.mjs";

const deferred = () => {
  let resolve;
  const promise = new Promise((r) => {
    resolve = r;
  });
  return { promise, resolve };
};

test("suspension retains admission and bounds the queue", async () => {
  const gate = new Admission({ maxQueued: 1 });
  const delay = deferred();
  const order = [];
  const first = gate.run(async () => {
    order.push(1);
    await delay.promise;
    order.push(2);
  });
  const second = gate.run(() => order.push(3));
  await assert.rejects(
    gate.run(() => assert.fail("overload entered Wasm")),
    /saturated/,
  );
  assert.deepEqual(order, [1]);
  delay.resolve();
  await Promise.all([first, second]);
  assert.deepEqual(order, [1, 2, 3]);
});

test("cancelled continuations cannot reenter or release queued work", async () => {
  const gate = new Admission();
  const delay = deferred();
  const controller = new AbortController();
  const first = gate.run(() => delay.promise, { signal: controller.signal });
  const second = gate.run(() =>
    assert.fail("queued operation entered discarded instance"),
  );
  const firstRejected = assert.rejects(first, /cancel/);
  const secondRejected = assert.rejects(second, /cancel/);
  controller.abort(new Error("cancel"));
  await Promise.all([firstRejected, secondRejected]);
  assert.equal(gate.closed, true);
  await assert.rejects(
    gate.run(() => {}),
    /discarded/,
  );
  delay.resolve("late value");
  await new Promise((r) => setTimeout(r, 0));
  assert.equal(gate.generation, 1);
});

test("trap and timeout discard instance and reject queued invocations", async () => {
  for (const fail of ["trap", "timeout"]) {
    const gate = new Admission();
    const first = gate.run(
      async () => {
        if (fail === "trap") throw new WebAssembly.RuntimeError("trap");
        await new Promise(() => {});
      },
      { timeoutMs: 5 },
    );
    const second = gate.run(() => assert.fail("reused suspect stack"));
    await Promise.all([assert.rejects(first), assert.rejects(second)]);
    assert.equal(gate.closed, true);
  }
});

test("aborted queued work is skipped without losing subsequent admission", async () => {
  const gate = new Admission();
  const delay = deferred();
  const controller = new AbortController();
  const first = gate.run(() => delay.promise);
  const second = gate.run(() => assert.fail("cancelled queue item ran"), {
    signal: controller.signal,
  });
  const rejected = assert.rejects(second);
  const third = gate.run(() => 3);
  controller.abort();
  delay.resolve();
  await first;
  await rejected;
  assert.equal(await third, 3);
});

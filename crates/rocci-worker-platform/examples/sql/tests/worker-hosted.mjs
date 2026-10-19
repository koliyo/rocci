import assert from "node:assert/strict";

export const profile = {
  prefix: "tx",
  className: "Transactions",
  binding: "TRANSACTIONS",
  aliases: ["./transactions.db"],
};

export async function exercise(request, scenarios) {
  const values = await (await request("/values")).text();
  for (const value of [
    "9223372036854775807",
    "-9223372036854775808",
    "🌍",
    "255",
    "null",
  ])
    assert.ok(values.includes(value), values);
  scenarios.push(
    "typed full-range I64, REAL, text, blob and NULL storage class",
  );
  for (const [endpoint, count] of [
    ["commit", 2],
    ["rollback", 2],
    ["drop", 2],
    ["typed-error", 2],
    ["trap", 2],
    ["unsupported-effect", 2],
    ["post-commit-error", 4],
  ]) {
    await request("/" + endpoint);
    assert.equal(Number(await (await request("/snapshot")).text()), count);
  }
  scenarios.push(
    "real transaction commit/rollback/drop/typed error/trap/post-commit visibility",
  );
}

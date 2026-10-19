import assert from "node:assert/strict";

export const profile = {
  prefix: "counter",
  className: "Starter",
  binding: "STARTER",
  aliases: ["./counter.db"],
};

export async function exercise(request, scenarios) {
  assert.match(
    await (await request("/actions/counter/reset", { method: "POST" })).text(),
    /<output[^>]*>0<\/output>/,
  );
  assert.match(
    await (
      await request("/actions/counter/increment", {
        method: "POST",
        headers: { "datastar-request": "true" },
      })
    ).text(),
    /<output[^>]*>1<\/output>/,
  );
  assert.match(await (await request("/")).text(), /<output[^>]*>1<\/output>/);
  scenarios.push("unchanged counter init/document/finite fragment");
}

export async function afterRedeploy(request, scenarios) {
  assert.match(await (await request("/")).text(), /<output[^>]*>1<\/output>/);
  scenarios.push("redeploy reconstructs durable counter state");
}

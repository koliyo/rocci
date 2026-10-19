import assert from "node:assert/strict";

export const profile = {
  prefix: "live-counter",
  className: "Live",
  binding: "LIVE",
  aliases: ["./counter.db"],
};

export async function exercise(request, scenarios) {
  await request("/actions/counter/reset", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: "{}",
  });
  scenarios.push("live counter reset status: " + (await request("/")).status);
  async function subscriber() {
    const controller = new AbortController();
    const response = await request("/sse", {
      signal: controller.signal,
    });
    assert.equal(response.status, 200);
    const reader = response.body.getReader();
    return {
      async count(expected) {
        let text = "";
        const timeout = setTimeout(() => controller.abort(), 20000);
        try {
          while (true) {
            const chunk = await reader.read();
            if (chunk.done) throw new Error("SSE ended");
            text += new TextDecoder().decode(chunk.value);
            if (
              new RegExp("<output[^>]*>" + expected + "<\\/output>").test(text)
            )
              return;
          }
        } finally {
          clearTimeout(timeout);
        }
      },
      async close() {
        controller.abort();
        await reader.cancel().catch(() => {});
      },
    };
  }
  const first = await subscriber();
  scenarios.push("first SSE headers");
  await first.count(0);
  scenarios.push("first SSE snapshot");
  const second = await subscriber();
  scenarios.push("second SSE headers");
  try {
    await second.count(0);
    scenarios.push("second SSE snapshot");
    await request("/actions/counter/increment", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: "{}",
    });
    await Promise.all([first.count(1), second.count(1)]);
  } finally {
    await first.close();
    await second.close();
  }
  scenarios.push(
    "unchanged live counter two subscribers and commit-before-publish",
  );
}

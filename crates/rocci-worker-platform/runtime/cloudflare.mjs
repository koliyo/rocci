import { Guest } from "./guest.mjs";

const methods = new Map([
  ["CONNECT", 0],
  ["DELETE", 1],
  ["GET", 3],
  ["HEAD", 4],
  ["OPTIONS", 5],
  ["PATCH", 6],
  ["POST", 7],
  ["PUT", 8],
  ["TRACE", 9],
  ["QUERY", 10],
]);

export class WorkerObject {
  constructor(module, context, env, profile) {
    this.context = context;
    this.module = module;
    this.env = env;
    this.profile = profile;
    this.reset();
  }
  reset() {
    this.guest = new Guest(this.module, {
      env: this.env,
      sql: this.context.storage.sql,
      storage: this.context.storage,
      aliases: this.profile.sqliteAliases,
      outboundFetch: this.profile.outboundFetch,
      sources: this.profile.sources,
    });
    this.ready = this.guest.invoke(0);
  }
  async fetch(request) {
    try {
      if (this.guest.closed) {
        if (!this.guest.settled)
          return new Response("Worker continuation is still settling", {
            status: 503,
            headers: { "Retry-After": "1" },
          });
        this.reset();
      }
      await this.ready;
      return await respond(this.guest, request, () =>
        this.context.storage.sync(),
      );
    } catch (error) {
      console.error("Rocci Worker request failed", String(error));
      return failure(error);
    }
  }
}

export async function respond(guest, request, durable = async () => {}) {
  const url = new URL(request.url);
  const encoder = new TextEncoder();
  if (
    encoder.encode(url.pathname + url.search).length >
    (guest.limits?.targetBytes ?? 8192)
  )
    return new Response("Request target too large", { status: 414 });
  const headers = [...request.headers];
  if (
    headers.length > (guest.limits?.headerFields ?? 100) ||
    headers.reduce(
      (size, [name, value]) =>
        size + encoder.encode(name).length + encoder.encode(value).length + 32,
      0,
    ) > (guest.limits?.headerBytes ?? 32768)
  )
    return new Response("Request headers too large", { status: 431 });
  const body = guest.add({ kind: "body", request, finished: false });
  let response;
  try {
    response = await guest.invoke(
      1,
      {
        method: methods.get(request.method) ?? 2,
        path: url.pathname,
        query: url.search ? url.search.slice(1) : undefined,
        headers,
        methodExt: methods.has(request.method) ? "" : request.method,
        body,
      },
      { signal: request.signal },
    );
  } finally {
    guest.finishBody(body);
  }
  if (response.kind === "response") {
    await durable();
    return new Response(
      request.method === "HEAD" || [204, 205, 304].includes(response.status)
        ? null
        : new Uint8Array(response.body),
      { status: response.status, headers: response.headers },
    );
  }
  if (response.kind !== "stream")
    throw new Error("Unsupported Roc response kind");
  const id = response.id;
  let closed = false,
    timer,
    wake;
  const finish = async () => {
    if (closed) return;
    closed = true;
    clearTimeout(timer);
    wake?.();
    if (!guest.closed) {
      try {
        await guest.invoke(3, { id });
      } catch (error) {
        guest.discard(error);
        throw error;
      }
    }
  };
  const stream = new ReadableStream(
    {
      async pull(controller) {
        try {
          clearTimeout(timer);
          // A Wait step must keep the pending pull alive. Returning without an
          // enqueue does not ask ReadableStream to pull again after an idle poll.
          while (!closed) {
            const step = await guest.invoke(2, { id });
            if (closed) return;
            if (step.kind === "end") {
              closed = true;
              controller.close();
              return;
            }
            if (step.kind === "emit") {
              const bytes = new Uint8Array(step.body);
              if (bytes.length > (guest.limits?.eventBytes ?? 1024 * 1024))
                throw new Error("SSE event exceeds 1 MiB");
              await durable();
              if (closed) return;
              controller.enqueue(bytes);
            }
            if (step.wait)
              await new Promise((resolve) => {
                wake = resolve;
                timer = setTimeout(resolve, Math.min(step.wait, 30_000));
              });
            wake = undefined;
            if (closed) return;
            if (step.kind === "emit") {
              timer = setTimeout(() => {
                finish().catch(() => {});
                try {
                  controller.error(new Error("SSE reader stalled"));
                } catch {}
              }, 30_000);
              return;
            }
            // Avoid a busy loop even if an extension returns a zero-delay Wait.
            if (!step.wait)
              await new Promise((resolve) => {
                wake = resolve;
                timer = setTimeout(resolve, 1);
              });
            wake = undefined;
          }
        } catch (error) {
          await finish();
          controller.error(error);
        }
      },
      cancel: finish,
    },
    { highWaterMark: 1 },
  );
  return new Response(stream, {
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
    },
  });
}

function headers(response, profile) {
  const copy = new Response(response.body, response);
  if (profile.csp) copy.headers.set("Content-Security-Policy", profile.csp);
  return copy;
}
function isAsset(request, profile) {
  const path = new URL(request.url).pathname;
  return profile.assetPaths?.includes(path) ?? path.startsWith("/assets/");
}
export function objectWorker(binding, objectKey, profile = {}) {
  return {
    async fetch(request, env) {
      if (isAsset(request, profile) && env.ASSETS)
        return headers(await env.ASSETS.fetch(request), profile);
      return headers(
        await env[binding].getByName(objectKey).fetch(request),
        profile,
      );
    },
  };
}
export function statelessWorker(module, profile = {}) {
  let guest, ready;
  return {
    async fetch(request, env) {
      if (isAsset(request, profile) && env.ASSETS)
        return headers(await env.ASSETS.fetch(request), profile);
      if (!guest || guest.closed) {
        if (guest && !guest.settled)
          return new Response("Worker continuation is still settling", {
            status: 503,
          });
        guest = new Guest(module, {
          env,
          outboundFetch: profile.outboundFetch,
          sources: profile.sources,
        });
        ready = guest.invoke(0);
      }
      try {
        await ready;
        return headers(await respond(guest, request), profile);
      } catch (error) {
        console.error("Rocci Worker request failed", String(error));
        return failure(error);
      }
    },
  };
}

function failure(error) {
  const unavailable =
    /queue saturated|capacity exhausted|still settling|instance discarded/.test(
      String(error),
    );
  return new Response(unavailable ? "Worker busy" : "Internal Server Error", {
    status: unavailable ? 503 : 500,
    headers: unavailable ? { "Retry-After": "1" } : {},
  });
}

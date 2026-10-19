// Explicit local identity fixture. The production config never uses this file.
import { route, Counter, Env } from '../src/index';
import { csrf } from '../src/auth';
export { Counter };
export default {
  async fetch(request: Request, env: Env) {
    if (!['127.0.0.1', 'localhost'].includes(new URL(request.url).hostname)) return new Response('Local fixture only', { status: 403 });
    try {
      csrf(request);
      const response = await route(request, env, 'fixture-owner');
      // Fault injection exists only in this unshipped local fixture.
      if (request.headers.get('x-test-drop-reply') === '1') return new Response('Simulated lost reply', { status: 503 });
      return response;
    }
    catch (error) { return new Response(String(error), { status: (error as { status?: number }).status ?? 500 }); }
  },
};

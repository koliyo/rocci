import { createRemoteJWKSet, jwtVerify } from 'jose';
import { HttpError } from './protocol';
export type AuthEnv = {
  COUNTER_HOSTNAME: string; ACCESS_TEAM_DOMAIN: string; ACCESS_AUD: string; OWNER_EMAIL: string;
};
const keysets = new Map<string, ReturnType<typeof createRemoteJWKSet>>();
export function issuer(env: AuthEnv): string {
  if (!/^[a-z0-9-]+\.cloudflareaccess\.com$/.test(env.ACCESS_TEAM_DOMAIN)
    || !env.ACCESS_AUD || !env.OWNER_EMAIL) throw new Error('Incomplete Access configuration');
  return `https://${env.ACCESS_TEAM_DOMAIN}`;
}
export async function authenticate(request: Request, env: AuthEnv): Promise<string> {
  const url = new URL(request.url);
  if (url.protocol !== 'https:' || url.host !== env.COUNTER_HOSTNAME) throw new HttpError(403, 'Unknown origin');
  const token = request.headers.get('Cf-Access-Jwt-Assertion');
  if (!token || token.length > 16384) throw new HttpError(401, 'Sign in through Cloudflare Access');
  const iss = issuer(env);
  let keys = keysets.get(iss);
  if (!keys) {
    keys = createRemoteJWKSet(new URL(`${iss}/cdn-cgi/access/certs`), { timeoutDuration: 5000, cacheMaxAge: 300000 });
    keysets.set(iss, keys);
  }
  try {
    const { payload } = await jwtVerify(token, keys, {
      issuer: iss, audience: env.ACCESS_AUD, algorithms: ['RS256'], requiredClaims: ['exp', 'iat', 'sub', 'email'],
    });
    if (payload.email !== env.OWNER_EMAIL || typeof payload.sub !== 'string' || !payload.sub || payload.sub.length > 256)
      throw new HttpError(403, 'Owner access only');
    return payload.sub;
  } catch (error) {
    if (error instanceof HttpError) throw error;
    throw new HttpError(401, 'Invalid or expired Access session');
  }
}
export function csrf(request: Request): void {
  if (request.method === 'GET' || request.method === 'HEAD') return;
  const url = new URL(request.url);
  if (request.headers.get('origin') !== url.origin
    || (request.headers.has('sec-fetch-site') && request.headers.get('sec-fetch-site') !== 'same-origin'))
    throw new HttpError(403, 'Same-origin request required');
}

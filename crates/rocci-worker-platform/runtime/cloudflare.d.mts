/** Additive provider adapter API. Application values remain ordinary HTTP. */
export interface WorkerProfile {
  sqliteAliases?: readonly string[];
  sources?: {
    fallback?: string;
    init?: string;
    routes?: readonly { method: string; path: string; location: string }[];
  };
  assetPaths?: readonly string[];
  csp?: string;
  /** Optional service binding adapter. Subject to guest byte/time/transaction limits. */
  outboundFetch?: (request: Request) => Promise<Response>;
}
export interface ObjectContext {
  storage: {
    sql: { exec(query: string, ...bindings: unknown[]): unknown };
    transaction<T>(callback: (storage: unknown) => Promise<T>): Promise<T>;
    sync(): Promise<void>;
  };
}
export class WorkerObject {
  constructor(
    module: WebAssembly.Module,
    context: ObjectContext,
    env: Record<string, unknown>,
    profile: WorkerProfile,
  );
  fetch(request: Request): Promise<Response>;
}
export function objectWorker(
  binding: string,
  objectKey: string,
  profile?: WorkerProfile,
): { fetch(request: Request, env: Record<string, any>): Promise<Response> };
export function statelessWorker(
  module: WebAssembly.Module,
  profile?: WorkerProfile,
): { fetch(request: Request, env: Record<string, any>): Promise<Response> };

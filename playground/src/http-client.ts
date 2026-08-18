import type {
  CompileRequest,
  CompileResponse,
  Language,
  VirtualWorkspace,
} from "./protocol";
import type { ClientStatus } from "./worker-client";

export interface HttpClientOptions {
  compileUrl: string;
  debounceMs?: number;
  timeoutMs?: number;
  onResponse?: (response: CompileResponse, durationMs: number) => void;
  onError?: (error: string, revision?: number) => void;
  onStatusChange?: (status: ClientStatus) => void;
}

export class HttpCompileClient {
  private options: HttpClientOptions;
  private status: ClientStatus = "uninitialized";
  private currentRevision = 0;
  private latestCompletedRevision = 0;
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;
  private timeoutTimer: ReturnType<typeof setTimeout> | null = null;
  private pendingRequest: CompileRequest | null = null;
  private inFlightController: AbortController | null = null;

  constructor(options: HttpClientOptions) {
    this.options = {
      debounceMs: 120,
      timeoutMs: 120000,
      ...options,
    };
    this.setStatus("ready");
  }

  private setStatus(newStatus: ClientStatus) {
    this.status = newStatus;
    this.options.onStatusChange?.(newStatus);
  }

  public requestCompile(
    filename: string,
    source: string,
    language?: Language,
    workspace?: VirtualWorkspace
  ): number {
    this.currentRevision += 1;
    const revision = this.currentRevision;
    this.pendingRequest = {
      protocol_version: 1,
      revision,
      filename,
      language,
      source,
      workspace,
    };

    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }
    const debounceMs = this.options.debounceMs ?? 120;
    if (debounceMs <= 0) {
      this.dispatchPending();
    } else {
      this.debounceTimer = setTimeout(() => {
        this.debounceTimer = null;
        this.dispatchPending();
      }, debounceMs);
    }
    return revision;
  }

  private dispatchPending() {
    if (!this.pendingRequest) {
      return;
    }
    if (this.inFlightController) {
      this.inFlightController.abort();
      this.inFlightController = null;
    }

    const req = this.pendingRequest;
    this.pendingRequest = null;
    this.setStatus("compiling");
    this.startTimeoutTimer(req.revision);

    const controller = new AbortController();
    this.inFlightController = controller;
    const started = performance.now();

    fetch(this.options.compileUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req),
      signal: controller.signal,
    })
      .then(async (resp) => {
        const text = await resp.text();
        if (!resp.ok) {
          throw new Error(`compile failed (${resp.status}): ${text.slice(0, 200)}`);
        }
        return JSON.parse(text) as CompileResponse;
      })
      .then((compileResponse) => {
        this.clearTimeoutTimer();
        if (
          compileResponse.revision <= this.latestCompletedRevision ||
          compileResponse.revision < this.currentRevision
        ) {
          return;
        }
        if (compileResponse.error) {
          this.latestCompletedRevision = compileResponse.revision;
          this.setStatus("error");
          this.options.onError?.(compileResponse.error, compileResponse.revision);
          return;
        }
        this.latestCompletedRevision = compileResponse.revision;
        this.setStatus("ready");
        this.options.onResponse?.(compileResponse, performance.now() - started);
        if (this.pendingRequest) {
          this.dispatchPending();
        }
      })
      .catch((err: unknown) => {
        if (controller.signal.aborted) {
          return;
        }
        this.clearTimeoutTimer();
        this.setStatus("error");
        const msg = err instanceof Error ? err.message : String(err);
        this.options.onError?.(msg, req.revision);
      })
      .finally(() => {
        if (this.inFlightController === controller) {
          this.inFlightController = null;
        }
      });
  }

  private clearTimeoutTimer() {
    if (this.timeoutTimer) {
      clearTimeout(this.timeoutTimer);
      this.timeoutTimer = null;
    }
  }

  private startTimeoutTimer(revision: number) {
    this.clearTimeoutTimer();
    const timeoutMs = this.options.timeoutMs || 120000;
    this.timeoutTimer = setTimeout(() => {
      this.inFlightController?.abort();
      this.setStatus("error");
      this.options.onError?.(`Compilation revision ${revision} timed out after ${timeoutMs}ms`, revision);
    }, timeoutMs);
  }

  public dispose() {
    if (this.debounceTimer) clearTimeout(this.debounceTimer);
    this.clearTimeoutTimer();
    this.inFlightController?.abort();
    this.inFlightController = null;
  }
}

import type {
  CompileRequest,
  CompileResponse,
  Language,
  VirtualWorkspace,
  WorkerRequest,
  WorkerResponse,
} from "./protocol";

export type ClientStatus = "uninitialized" | "ready" | "compiling" | "error" | "crashed";

export interface WorkerClientOptions {
  wasmUrl: string;
  workerUrl?: string;
  workerFactory?: () => Worker;
  debounceMs?: number;
  timeoutMs?: number;
  onResponse?: (response: CompileResponse, durationMs: number) => void;
  onError?: (error: string, revision?: number) => void;
  onStatusChange?: (status: ClientStatus) => void;
}

export class PlaygroundWorkerClient {
  private options: WorkerClientOptions;
  private worker: Worker | null = null;
  private status: ClientStatus = "uninitialized";
  private currentRevision = 0;
  private latestCompletedRevision = 0;
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;
  private timeoutTimer: ReturnType<typeof setTimeout> | null = null;
  private pendingRequest: CompileRequest | null = null;
  private inFlightRequest: CompileRequest | null = null;

  constructor(options: WorkerClientOptions) {
    this.options = {
      debounceMs: 120,
      timeoutMs: 8000,
      ...options,
    };
    this.initWorker();
  }

  private setStatus(newStatus: ClientStatus) {
    this.status = newStatus;
    this.options.onStatusChange?.(newStatus);
  }

  public getStatus(): ClientStatus {
    return this.status;
  }

  public getCurrentRevision(): number {
    return this.currentRevision;
  }

  public getLatestCompletedRevision(): number {
    return this.latestCompletedRevision;
  }

  private initWorker() {
    if (this.worker) {
      try {
        this.worker.terminate();
      } catch {
        // ignore
      }
      this.worker = null;
    }

    try {
      if (this.options.workerFactory) {
        this.worker = this.options.workerFactory();
      } else if (this.options.workerUrl) {
        this.worker = new Worker(this.options.workerUrl, { type: "module" });
      } else {
        throw new Error("Neither workerUrl nor workerFactory provided");
      }

      this.worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
        this.handleWorkerMessage(event.data);
      };

      this.worker.onerror = (err) => {
        this.handleWorkerError(err instanceof Error ? err.message : "Worker encountered an unhandled error");
      };

      const initMsg: WorkerRequest = {
        type: "init",
        wasmUrl: this.options.wasmUrl,
      };
      this.worker.postMessage(initMsg);
    } catch (e: unknown) {
      this.setStatus("error");
      const msg = e instanceof Error ? e.message : String(e);
      this.options.onError?.(`Failed to spawn worker: ${msg}`);
    }
  }

  private handleWorkerMessage(data: WorkerResponse) {
    if (!data) return;

    if (data.type === "init_ok") {
      this.setStatus("ready");
      if (this.pendingRequest) {
        this.dispatchPending();
      }
    } else if (data.type === "init_error") {
      this.setStatus("error");
      this.options.onError?.(`Worker WASM initialization failed: ${data.error}`);
    } else if (data.type === "compile_ok") {
      this.clearTimeoutTimer();
      const resp = data.response;
      // Stale response check: Drop responses older than latest submitted or completed revision
      if (resp.revision <= this.latestCompletedRevision || resp.revision < this.currentRevision) {
        // Stale response ignored
        return;
      }

      this.latestCompletedRevision = resp.revision;
      this.inFlightRequest = null;
      this.setStatus("ready");
      this.options.onResponse?.(resp, data.durationMs);

      if (this.pendingRequest) {
        this.dispatchPending();
      }
    } else if (data.type === "compile_error") {
      this.clearTimeoutTimer();
      if (data.revision >= this.latestCompletedRevision) {
        this.latestCompletedRevision = data.revision;
        this.inFlightRequest = null;
        this.setStatus("error");
        this.options.onError?.(data.error, data.revision);
      }
    }
  }

  private handleWorkerError(errorMessage: string) {
    this.clearTimeoutTimer();
    this.setStatus("crashed");
    this.options.onError?.(`Worker crashed: ${errorMessage}`);
    // Attempt automatic restart after brief delay
    setTimeout(() => {
      this.initWorker();
    }, 200);
  }

  private clearTimeoutTimer() {
    if (this.timeoutTimer) {
      clearTimeout(this.timeoutTimer);
      this.timeoutTimer = null;
    }
  }

  private startTimeoutTimer(revision: number) {
    this.clearTimeoutTimer();
    const timeoutMs = this.options.timeoutMs || 8000;
    this.timeoutTimer = setTimeout(() => {
      this.handleWorkerError(`Compilation revision ${revision} timed out after ${timeoutMs}ms`);
    }, timeoutMs);
  }

  /**
   * Request a compilation of source text. Automatically increments revision and debounces.
   */
  public requestCompile(
    filename: string,
    source: string,
    language?: Language,
    workspace?: VirtualWorkspace
  ): number {
    this.currentRevision += 1;
    const revision = this.currentRevision;

    const request: CompileRequest = {
      protocol_version: 1,
      revision,
      filename,
      language,
      source,
      workspace,
    };

    this.pendingRequest = request;

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
    if (!this.pendingRequest || !this.worker || this.status === "uninitialized" || this.status === "error" || this.status === "crashed") {
      return;
    }

    const req = this.pendingRequest;
    this.pendingRequest = null;
    this.inFlightRequest = req;
    this.setStatus("compiling");
    this.startTimeoutTimer(req.revision);

    const msg: WorkerRequest = {
      type: "compile",
      request: req,
    };
    this.worker.postMessage(msg);
  }

  public dispose() {
    if (this.debounceTimer) clearTimeout(this.debounceTimer);
    this.clearTimeoutTimer();
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
    }
  }
}

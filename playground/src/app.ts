import type {
  CompileResponse,
  Language,
  PlaygroundBootstrap,
  PlaygroundBootstrapDocument,
  PlaygroundDiagnostic,
} from "./protocol";
import { CodeEditor } from "./editor";
import { PlaygroundWorkerClient } from "./worker-client";

export type OutputMode = "roc" | "AST" | "html";

export interface PlaygroundAppOptions {
  container: HTMLElement;
  bootstrap: PlaygroundBootstrap;
}

export class PlaygroundApp {
  private container: HTMLElement;
  private bootstrap: PlaygroundBootstrap;
  private documents: Map<string, PlaygroundBootstrapDocument> = new Map();
  private selectedDocId: string;
  private currentMode: OutputMode = "roc";
  private splitPercent = 50;

  private workerClient: PlaygroundWorkerClient;
  private sourceEditor!: CodeEditor;
  private outputEditor!: CodeEditor;

  private lastResponse: CompileResponse | null = null;
  private lastDiagnostics: PlaygroundDiagnostic[] = [];

  // DOM Elements
  private tabsContainer!: HTMLElement;
  private modeSelect!: HTMLSelectElement;
  private copyBtn!: HTMLButtonElement;
  private sourceContainer!: HTMLElement;
  private outputContainer!: HTMLElement;
  private outputEditorWrapper!: HTMLElement;
  private htmlCardWrapper!: HTMLElement;
  private diagnosticsPanel!: HTMLElement;
  private statusDot!: HTMLElement;
  private statusText!: HTMLElement;
  private errorCountText!: HTMLElement;
  private timingText!: HTMLElement;
  private ariaLiveRegion!: HTMLElement;

  constructor(options: PlaygroundAppOptions) {
    this.container = options.container;
    this.bootstrap = options.bootstrap;

    for (const doc of this.bootstrap.documents) {
      this.documents.set(doc.id, doc);
    }
    this.selectedDocId = this.bootstrap.selected_document || this.bootstrap.documents[0]?.id || "doc1";

    this.renderLayout();
    this.initEditors();

    this.workerClient = new PlaygroundWorkerClient({
      wasmUrl: this.bootstrap.compiler_wasm_url,
      workerUrl: this.bootstrap.worker_url,
      onResponse: (response, durationMs) => this.handleCompileResponse(response, durationMs),
      onError: (err) => this.handleCompileError(err),
      onStatusChange: (status) => this.handleStatusChange(status),
    });

    this.loadDocument(this.selectedDocId);
  }

  private renderLayout() {
    this.container.innerHTML = `
      <div class="playground-app">
        <header class="playground-toolbar" role="toolbar" aria-label="Playground toolbar">
          <div class="toolbar-left">
            <nav class="file-tabs" role="tablist" aria-label="Example documents"></nav>
          </div>
          <div class="toolbar-right">
            <div class="selector-group">
              <label for="output-mode-select" class="sr-only">Output representation</label>
              <select id="output-mode-select" class="mode-select" aria-label="Select output mode">
                <option value="roc">roc</option>
                <option value="AST">AST</option>
                <option value="html">html</option>
              </select>
              <button id="copy-output-btn" class="action-btn" aria-label="Copy output to clipboard">Copy</button>
            </div>
          </div>
        </header>

        <main class="playground-workbench">
          <section class="pane-container source-pane" style="width: ${this.splitPercent}%">
            <div class="pane-header">
              <span id="source-filename">Source</span>
            </div>
            <div class="source-editor-wrapper" style="flex: 1; overflow: hidden; position: relative;"></div>
            <div class="diagnostics-panel" role="region" aria-label="Diagnostics" style="display: none;"></div>
          </section>

          <div class="splitter" role="separator" tabindex="0" aria-orientation="vertical" aria-valuenow="${this.splitPercent}" aria-label="Resize workbench panes"></div>

          <section class="pane-container output-pane" style="width: ${100 - this.splitPercent}%">
            <div class="pane-header">
              <span id="output-title">Generated Roc</span>
            </div>
            <div class="output-editor-wrapper" style="flex: 1; overflow: hidden; position: relative;"></div>
            <div class="html-card-wrapper" style="display: none; flex: 1; overflow: auto;"></div>
          </section>
        </main>

        <footer class="playground-footer" role="status">
          <div class="status-left">
            <span class="status-dot"></span>
            <span class="status-text">Initializing WASM...</span>
          </div>
          <div class="status-right">
            <span class="error-count-text">0 errors</span>
            <span class="timing-text"></span>
          </div>
          <div class="aria-live-region sr-only" aria-live="polite" aria-atomic="true"></div>
        </footer>
      </div>
    `;

    this.tabsContainer = this.container.querySelector(".file-tabs")!;
    this.modeSelect = this.container.querySelector("#output-mode-select")!;
    this.copyBtn = this.container.querySelector("#copy-output-btn")!;
    this.sourceContainer = this.container.querySelector(".source-editor-wrapper")!;
    this.outputContainer = this.container.querySelector(".output-pane")!;
    this.outputEditorWrapper = this.container.querySelector(".output-editor-wrapper")!;
    this.htmlCardWrapper = this.container.querySelector(".html-card-wrapper")!;
    this.diagnosticsPanel = this.container.querySelector(".diagnostics-panel")!;
    this.statusDot = this.container.querySelector(".status-dot")!;
    this.statusText = this.container.querySelector(".status-text")!;
    this.errorCountText = this.container.querySelector(".error-count-text")!;
    this.timingText = this.container.querySelector(".timing-text")!;
    this.ariaLiveRegion = this.container.querySelector(".aria-live-region")!;

    this.renderTabs();
    this.setupEvents();
  }

  private renderTabs() {
    this.tabsContainer.innerHTML = "";
    for (const doc of this.documents.values()) {
      const tabBtn = document.createElement("button");
      tabBtn.className = `file-tab ${doc.id === this.selectedDocId ? "active" : ""}`;
      tabBtn.setAttribute("role", "tab");
      tabBtn.setAttribute("aria-selected", String(doc.id === this.selectedDocId));
      tabBtn.textContent = doc.filename;
      tabBtn.addEventListener("click", () => this.loadDocument(doc.id));
      this.tabsContainer.appendChild(tabBtn);
    }
  }

  private initEditors() {
    this.sourceEditor = new CodeEditor({
      parent: this.sourceContainer,
      onChange: (value) => this.onSourceChanged(value),
    });

    this.outputEditor = new CodeEditor({
      parent: this.outputEditorWrapper,
      readOnly: true,
    });
  }

  private setupEvents() {
    // Mode select change
    this.modeSelect.addEventListener("change", () => {
      this.setMode(this.modeSelect.value as OutputMode);
    });

    // Copy button
    this.copyBtn.addEventListener("click", () => {
      this.copyCurrentOutput();
    });

    // Splitter resizing
    const splitter = this.container.querySelector(".splitter") as HTMLElement;
    let isDragging = false;

    splitter.addEventListener("mousedown", (e) => {
      e.preventDefault();
      isDragging = true;
      document.body.style.cursor = "col-resize";
    });

    window.addEventListener("mousemove", (e) => {
      if (!isDragging) return;
      const workbench = this.container.querySelector(".playground-workbench") as HTMLElement;
      const rect = workbench.getBoundingClientRect();
      const percent = Math.max(15, Math.min(85, ((e.clientX - rect.left) / rect.width) * 100));
      this.setSplitPercent(percent);
    });

    window.addEventListener("mouseup", () => {
      if (isDragging) {
        isDragging = false;
        document.body.style.cursor = "";
      }
    });

    // Splitter keyboard navigation
    splitter.addEventListener("keydown", (e) => {
      if (e.key === "ArrowLeft") {
        this.setSplitPercent(Math.max(15, this.splitPercent - 2));
      } else if (e.key === "ArrowRight") {
        this.setSplitPercent(Math.min(85, this.splitPercent + 2));
      } else if (e.key === "Home") {
        this.setSplitPercent(20);
      } else if (e.key === "End") {
        this.setSplitPercent(80);
      } else if (e.key === "Enter" || e.key === " ") {
        this.setSplitPercent(50);
      }
    });
  }

  private setSplitPercent(percent: number) {
    this.splitPercent = percent;
    const sourcePane = this.container.querySelector(".source-pane") as HTMLElement;
    const outputPane = this.container.querySelector(".output-pane") as HTMLElement;
    const splitter = this.container.querySelector(".splitter") as HTMLElement;

    if (sourcePane && outputPane) {
      sourcePane.style.width = `${percent}%`;
      outputPane.style.width = `${100 - percent}%`;
      splitter.setAttribute("aria-valuenow", String(Math.round(percent)));
    }
  }

  public loadDocument(docId: string) {
    const doc = this.documents.get(docId);
    if (!doc) return;

    this.selectedDocId = docId;
    this.renderTabs();

    const filenameSpan = this.container.querySelector("#source-filename")!;
    filenameSpan.textContent = doc.filename;

    this.sourceEditor.setValue(doc.source);
    this.triggerCompile();
  }

  private onSourceChanged(value: string) {
    const currentDoc = this.documents.get(this.selectedDocId);
    if (currentDoc) {
      currentDoc.source = value;
    }
    this.triggerCompile();
  }

  private triggerCompile() {
    const currentDoc = this.documents.get(this.selectedDocId);
    if (!currentDoc) return;

    this.workerClient.requestCompile(
      currentDoc.filename,
      currentDoc.source,
      currentDoc.language
    );
  }

  private handleCompileResponse(response: CompileResponse, durationMs: number) {
    this.lastResponse = response;
    this.lastDiagnostics = response.diagnostics || [];

    // Apply source decorations and diagnostics
    if (response.highlights?.source) {
      this.sourceEditor.setHighlights(response.highlights.source);
    }
    this.sourceEditor.setDiagnostics(this.lastDiagnostics);

    // Update diagnostics panel
    this.renderDiagnostics(this.lastDiagnostics);

    // Update output according to current mode
    this.updateOutputView();

    // Update status bar
    const errCount = this.lastDiagnostics.filter((d) => d.severity === "error").length;
    this.errorCountText.textContent = `${errCount} error${errCount === 1 ? "" : "s"}`;
    this.timingText.textContent = `${durationMs.toFixed(1)} ms`;

    if (errCount > 0) {
      this.announceAria(`Compilation complete with ${errCount} error${errCount === 1 ? "" : "s"}`);
    } else {
      this.announceAria("Compilation successful");
    }
  }

  private handleCompileError(errorMsg: string) {
    this.statusText.textContent = errorMsg;
    this.statusDot.className = "status-dot error";
  }

  private handleStatusChange(status: string) {
    this.statusDot.className = `status-dot ${status}`;
    if (status === "ready") {
      this.statusText.textContent = "WASM ready";
    } else if (status === "compiling") {
      this.statusText.textContent = "Compiling...";
    } else if (status === "error" || status === "crashed") {
      this.statusText.textContent = "Compiler error";
    }
  }

  public setMode(mode: OutputMode) {
    this.currentMode = mode;
    this.modeSelect.value = mode;
    this.updateOutputView();
  }

  private updateOutputView() {
    const titleSpan = this.container.querySelector("#output-title")!;
    if (this.currentMode === "roc") {
      titleSpan.textContent = "Generated Roc";
      this.outputEditorWrapper.style.display = "block";
      this.htmlCardWrapper.style.display = "none";
      this.copyBtn.style.display = "inline-flex";

      const rocText = this.lastResponse?.roc || "";
      this.outputEditor.setValue(rocText);
      if (this.lastResponse?.highlights?.roc) {
        this.outputEditor.setHighlights(this.lastResponse.highlights.roc);
      }
    } else if (this.currentMode === "AST") {
      titleSpan.textContent = "Formatted AST";
      this.outputEditorWrapper.style.display = "block";
      this.htmlCardWrapper.style.display = "none";
      this.copyBtn.style.display = "inline-flex";

      const astText = this.lastResponse?.ast || "";
      this.outputEditor.setValue(astText);
      if (this.lastResponse?.highlights?.ast) {
        this.outputEditor.setHighlights(this.lastResponse.highlights.ast);
      }
    } else if (this.currentMode === "html") {
      titleSpan.textContent = "HTML Preview";
      this.outputEditorWrapper.style.display = "none";
      this.htmlCardWrapper.style.display = "flex";
      this.copyBtn.style.display = "none";

      const reason =
        this.lastResponse?.capabilities?.html?.reason ||
        this.bootstrap.html_runtime?.reason ||
        "HTML preview is not available yet. Rocci can parse and lower this file in Rust/WASM, but rendering the generated Roc also requires a Roc runtime in WebAssembly.";

      this.htmlCardWrapper.innerHTML = `
        <div class="html-unavailable-card">
          <div class="html-unavailable-icon" aria-hidden="true">⚙️</div>
          <h2 class="html-unavailable-title">HTML Preview Unavailable</h2>
          <p class="html-unavailable-body">${reason}</p>
        </div>
      `;
    }
  }

  private renderDiagnostics(diagnostics: PlaygroundDiagnostic[]) {
    if (diagnostics.length === 0) {
      this.diagnosticsPanel.style.display = "none";
      this.diagnosticsPanel.innerHTML = "";
      return;
    }

    this.diagnosticsPanel.style.display = "flex";
    this.diagnosticsPanel.innerHTML = "";

    for (const diag of diagnostics) {
      const item = document.createElement("div");
      item.className = "diagnostic-item";
      item.setAttribute("tabindex", "0");
      item.setAttribute("role", "button");
      item.innerHTML = `
        <span class="diag-badge ${diag.severity}">${diag.severity}</span>
        <span class="diag-msg">${diag.message}</span>
        <span class="diag-loc">${diag.from}:${diag.to}</span>
      `;
      item.addEventListener("click", () => {
        this.sourceEditor.setCursor(diag.from, diag.to);
      });
      item.addEventListener("keydown", (e) => {
        if (e.key === "Enter" || e.key === " ") {
          this.sourceEditor.setCursor(diag.from, diag.to);
        }
      });
      this.diagnosticsPanel.appendChild(item);
    }
  }

  private copyCurrentOutput() {
    const text = this.outputEditor.getValue();
    if (navigator.clipboard) {
      navigator.clipboard.writeText(text).then(() => {
        const orig = this.copyBtn.textContent;
        this.copyBtn.textContent = "Copied!";
        setTimeout(() => {
          this.copyBtn.textContent = orig;
        }, 1500);
      });
    }
  }

  private announceAria(message: string) {
    if (this.ariaLiveRegion) {
      this.ariaLiveRegion.textContent = message;
    }
  }

  public dispose() {
    this.workerClient.dispose();
    this.sourceEditor.destroy();
    this.outputEditor.destroy();
  }
}

import type {
  CompileResponse,
  Language,
  PlaygroundBootstrap,
  PlaygroundBootstrapDocument,
  PlaygroundDiagnostic,
} from "./protocol";
import { CodeEditor } from "./editor";
import { HttpCompileClient } from "./http-client";
import { PlaygroundWorkerClient, type ClientStatus } from "./worker-client";

export type OutputMode = "roc" | "AST" | "html";

export interface PlaygroundAppOptions {
  container: HTMLElement;
  bootstrap: PlaygroundBootstrap;
}

interface SourceBuffer {
  filename: string;
  source: string;
  exampleId: string | null;
}

function untitledFilename(language: Language): string {
  return language === "rocci" ? "untitled.rocci" : "untitled.rocdown";
}

function emptyBuffer(language: Language): SourceBuffer {
  return {
    filename: untitledFilename(language),
    source: "",
    exampleId: null,
  };
}

export class PlaygroundApp {
  private container: HTMLElement;
  private bootstrap: PlaygroundBootstrap;
  private documents: Map<string, PlaygroundBootstrapDocument> = new Map();
  private buffers: Record<Language, SourceBuffer>;
  private currentLanguage: Language;
  private currentMode: OutputMode = "roc";
  private splitPercent = 50;
  private nativeLanguages: Language[];

  private httpClient: HttpCompileClient | null = null;
  private wasmClient: PlaygroundWorkerClient | null = null;
  private sourceEditor!: CodeEditor;
  private outputEditor!: CodeEditor;
  private isLocalMode: boolean;

  private lastResponse: CompileResponse | null = null;
  private lastDiagnostics: PlaygroundDiagnostic[] = [];

  private tabsContainer!: HTMLElement;
  private languageSelect!: HTMLSelectElement;
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
    this.isLocalMode = this.bootstrap.mode === "local";
    this.nativeLanguages = this.bootstrap.native_languages ?? [];

    for (const doc of this.bootstrap.documents) {
      this.documents.set(doc.id, doc);
    }

    const selected = this.documents.get(this.bootstrap.selected_document);
    this.currentLanguage = selected?.language ?? this.bootstrap.documents[0]?.language ?? "rocci";
    this.buffers = {
      rocci: this.bufferForLanguage("rocci"),
      rocdown: this.bufferForLanguage("rocdown"),
    };

    this.renderLayout();
    this.initEditors();
    this.initCompilers();
    this.applyBuffer();
  }

  private bufferForLanguage(language: Language): SourceBuffer {
    const selected = this.documents.get(this.bootstrap.selected_document);
    const match =
      selected?.language === language
        ? selected
        : [...this.documents.values()].find((doc) => doc.language === language);
    if (!match) {
      return emptyBuffer(language);
    }
    return {
      filename: match.filename,
      source: match.source,
      exampleId: match.id,
    };
  }

  private examplesFor(language: Language): PlaygroundBootstrapDocument[] {
    return [...this.documents.values()].filter((doc) => doc.language === language);
  }

  private renderLayout() {
    this.container.innerHTML = `
      <div class="playground-app">
        <header class="playground-toolbar" role="toolbar" aria-label="Playground toolbar">
          <div class="toolbar-left">
            <div class="selector-group">
              <label for="source-language-select" class="sr-only">Source language</label>
              <select id="source-language-select" class="mode-select" aria-label="Select source language">
                <option value="rocci">rocci</option>
                <option value="rocdown">rocdown</option>
              </select>
            </div>
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
            <div class="html-card-wrapper"></div>
          </section>
        </main>

        <footer class="playground-footer" role="status">
          <div class="status-left">
            <span class="status-dot"></span>
            <span class="status-text">${this.isLocalMode ? "Connecting..." : "Initializing WASM..."}</span>
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
    this.languageSelect = this.container.querySelector("#source-language-select")!;
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

    this.languageSelect.value = this.currentLanguage;
    this.renderTabs();
    this.setupEvents();
  }

  private renderTabs() {
    this.tabsContainer.innerHTML = "";
    const examples = this.examplesFor(this.currentLanguage);
    if (examples.length <= 1) {
      this.tabsContainer.style.display = "none";
      return;
    }
    this.tabsContainer.style.display = "flex";
    const activeId = this.buffers[this.currentLanguage].exampleId;
    for (const doc of examples) {
      const tabBtn = document.createElement("button");
      tabBtn.className = `file-tab ${doc.id === activeId ? "active" : ""}`;
      tabBtn.setAttribute("role", "tab");
      tabBtn.setAttribute("aria-selected", String(doc.id === activeId));
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

  private initCompilers() {
    const onResponse = (response: CompileResponse, durationMs: number) =>
      this.handleCompileResponse(response, durationMs);
    const onError = (err: string) => this.handleCompileError(err);

    if (this.isLocalMode && this.bootstrap.compile_url && this.nativeLanguages.length > 0) {
      this.httpClient = new HttpCompileClient({
        compileUrl: this.bootstrap.compile_url,
        onResponse,
        onError,
        onStatusChange: (status) => this.handleStatusChange(status, "native"),
      });
    }

    const needsWasm = !this.httpClient || this.nativeLanguages.length < 2;
    if (needsWasm) {
      this.wasmClient = new PlaygroundWorkerClient({
        wasmUrl: this.bootstrap.compiler_wasm_url,
        workerUrl: this.bootstrap.worker_url,
        onResponse,
        onError,
        onStatusChange: (status) => this.handleStatusChange(status, "wasm"),
      });
    }
  }

  private usesNative(language: Language): boolean {
    return Boolean(this.httpClient) && this.nativeLanguages.includes(language);
  }

  private compilerFor(language: Language): HttpCompileClient | PlaygroundWorkerClient | null {
    if (this.usesNative(language)) {
      return this.httpClient;
    }
    return this.wasmClient;
  }

  private setupEvents() {
    this.languageSelect.addEventListener("change", () => {
      this.setLanguage(this.languageSelect.value as Language);
    });

    this.modeSelect.addEventListener("change", () => {
      this.setMode(this.modeSelect.value as OutputMode);
    });

    this.copyBtn.addEventListener("click", () => {
      this.copyCurrentOutput();
    });

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

  public setLanguage(language: Language) {
    if (language === this.currentLanguage) {
      return;
    }
    this.syncEditorToBuffer();
    this.currentLanguage = language;
    this.languageSelect.value = language;
    this.applyBuffer();
  }

  public loadDocument(docId: string) {
    const doc = this.documents.get(docId);
    if (!doc) return;

    this.syncEditorToBuffer();
    this.currentLanguage = doc.language;
    this.languageSelect.value = doc.language;
    this.buffers[doc.language] = {
      filename: doc.filename,
      source: doc.source,
      exampleId: doc.id,
    };
    this.applyBuffer();
  }

  private syncEditorToBuffer() {
    this.buffers[this.currentLanguage].source = this.sourceEditor.getValue();
  }

  private applyBuffer() {
    const buffer = this.buffers[this.currentLanguage];
    const filenameSpan = this.container.querySelector("#source-filename")!;
    filenameSpan.textContent = buffer.filename;
    this.renderTabs();
    this.sourceEditor.setValue(buffer.source);
    this.sourceEditor.setHighlights([]);
    this.sourceEditor.setDiagnostics([]);
    if (!buffer.source.trim()) {
      this.clearOutput();
      return;
    }
    this.triggerCompile();
  }

  private onSourceChanged(value: string) {
    const buffer = this.buffers[this.currentLanguage];
    buffer.source = value;
    if (!value.trim()) {
      this.clearOutput();
      return;
    }
    this.triggerCompile();
  }

  private triggerCompile() {
    const buffer = this.buffers[this.currentLanguage];
    if (!buffer.source.trim()) {
      return;
    }
    const client = this.compilerFor(this.currentLanguage);
    if (!client) {
      this.handleCompileError("No compiler is available for this language.");
      return;
    }
    client.requestCompile(buffer.filename, buffer.source, this.currentLanguage);
  }

  private clearOutput() {
    this.lastResponse = null;
    this.lastDiagnostics = [];
    this.sourceEditor.setHighlights([]);
    this.sourceEditor.setDiagnostics([]);
    this.renderDiagnostics([]);
    this.errorCountText.textContent = "0 errors";
    this.timingText.textContent = "";
    this.updateOutputView();
  }

  private handleCompileResponse(response: CompileResponse, durationMs: number) {
    if (response.language !== this.currentLanguage) {
      return;
    }
    this.lastResponse = response;
    this.lastDiagnostics = response.diagnostics || [];

    if (response.highlights?.source) {
      this.sourceEditor.setHighlights(response.highlights.source);
    }
    this.sourceEditor.setDiagnostics(this.lastDiagnostics);

    this.renderDiagnostics(this.lastDiagnostics);
    this.updateOutputView();

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

  private handleStatusChange(status: ClientStatus, origin: "native" | "wasm") {
    const expected = this.usesNative(this.currentLanguage) ? "native" : "wasm";
    if (origin !== expected) {
      return;
    }
    this.statusDot.className = `status-dot ${status}`;
    if (status === "ready") {
      this.statusText.textContent = origin === "native" ? "Local compiler ready" : "WASM ready";
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
      } else {
        this.outputEditor.setHighlights([]);
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
      } else {
        this.outputEditor.setHighlights([]);
      }
    } else if (this.currentMode === "html") {
      titleSpan.textContent = "HTML Preview";
      const html = this.lastResponse?.html || "";
      const available = this.lastResponse?.capabilities?.html?.available && html.length > 0;
      this.copyBtn.style.display = available ? "inline-flex" : "none";

      if (available) {
        this.outputEditorWrapper.style.display = "none";
        this.htmlCardWrapper.style.display = "flex";
        this.htmlCardWrapper.replaceChildren();
        const preview = document.createElement("div");
        preview.className = "html-preview";
        const iframe = document.createElement("iframe");
        iframe.setAttribute("sandbox", "");
        iframe.setAttribute("title", "HTML preview");
        iframe.srcdoc = html;
        preview.appendChild(iframe);
        this.htmlCardWrapper.appendChild(preview);
      } else {
        this.outputEditorWrapper.style.display = "none";
        this.htmlCardWrapper.style.display = "flex";
        const reason = this.lastResponse
          ? this.lastResponse.capabilities?.html?.reason ||
            this.bootstrap.html_runtime?.reason ||
            "HTML preview is not available."
          : "Type a component to preview HTML, or switch to rocci if this language has no snapshot yet.";
        this.htmlCardWrapper.replaceChildren();
        const card = document.createElement("div");
        card.className = "html-unavailable-card";
        card.innerHTML = `
          <div class="html-unavailable-icon" aria-hidden="true">⚙️</div>
          <h2 class="html-unavailable-title">HTML Preview Unavailable</h2>
          <p class="html-unavailable-body"></p>
        `;
        card.querySelector(".html-unavailable-body")!.textContent = reason;
        this.htmlCardWrapper.appendChild(card);
      }
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
    const text =
      this.currentMode === "html"
        ? this.lastResponse?.html || ""
        : this.outputEditor.getValue();
    if (!text || !navigator.clipboard) {
      return;
    }
    navigator.clipboard.writeText(text).then(() => {
      const orig = this.copyBtn.textContent;
      this.copyBtn.textContent = "Copied!";
      setTimeout(() => {
        this.copyBtn.textContent = orig;
      }, 1500);
    });
  }

  private announceAria(message: string) {
    if (this.ariaLiveRegion) {
      this.ariaLiveRegion.textContent = message;
    }
  }

  public dispose() {
    this.httpClient?.dispose();
    this.wasmClient?.dispose();
    this.sourceEditor.destroy();
    this.outputEditor.destroy();
  }
}

async function bootFromMount() {
  const root = document.getElementById("playground-root");
  if (!(root instanceof HTMLElement)) {
    return;
  }
  const sessionUrl = root.dataset.session;
  if (!sessionUrl) {
    return;
  }
  try {
    const resp = await fetch(sessionUrl);
    if (!resp.ok) {
      throw new Error(`Failed to load playground session (${resp.status})`);
    }
    const bootstrap = (await resp.json()) as PlaygroundBootstrap;
    new PlaygroundApp({ container: root, bootstrap });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    root.textContent = "Failed to load playground session: " + message;
  }
}

void bootFromMount();

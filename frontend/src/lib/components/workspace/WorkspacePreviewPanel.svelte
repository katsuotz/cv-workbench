<script lang="ts">
  import { tick } from 'svelte';
  import { Button } from '../base';
  import CodeEditor from '../CodeEditor.svelte';
  import PreviewPane from '../PreviewPane.svelte';
  import type { PreviewState } from '$lib/workspace/previewController';

  export let state: PreviewState;
  export let advanced: boolean;
  export let lastGeneratedSource: string;
  export let rendering = false;
  export let controllerReady = false;
  export let diagnosticLine: number | null = null;
  export let diagnosticColumn: number | null = null;
  export let hidden = false;
  export let onCopySource: () => void;
  export let onDownloadText: () => void;
  export let onDownloadPdf: () => void;
  export let onRefresh: () => void;
  export let onToggleAdvanced: () => void;
  export let onDiagnosticSelect: (line: number, column: number) => void;

  let fullPage = false;
  let fullPageDialog: HTMLDialogElement;

  function openFullPage() {
    fullPage = true;
    void tick().then(() => {
      if (!fullPageDialog?.open) fullPageDialog?.showModal();
      fullPageDialog?.focus();
    });
  }

  function closeFullPage() {
    fullPageDialog?.close();
    fullPage = false;
  }
</script>

<section class="preview-panel" class:mobile-hidden={hidden} aria-label="Rendered preview">
  {#if advanced}
    <div class="source-panel">
      <div class="source-header">
        <div>
          <h2>LaTeX source</h2>
        </div>
        <div class="source-actions">
          <Button variant="secondary" className="mobile-source-toggle" onClick={onToggleAdvanced}>
            Preview
          </Button>
          <Button
            variant="secondary"
            className="source-action"
            onClick={onCopySource}
            disabled={!lastGeneratedSource}>
            Copy
          </Button>
          <Button
            variant="secondary"
            className="source-action"
            onClick={onDownloadText}
            disabled={!lastGeneratedSource}>
            Download .tex
          </Button>
        </div>
      </div>
      {#if lastGeneratedSource}
        <div class="source-code">
          <CodeEditor
            value={lastGeneratedSource}
            onChange={() => undefined}
            onCompile={() => undefined}
            {diagnosticLine}
            {diagnosticColumn}
            colorScheme="dark"
            readOnly />
        </div>
      {:else}
        <pre class="source-code">Preview your CV to inspect its exact LaTeX source.</pre>
      {/if}
    </div>
  {:else}
    <div class="preview-header">
      <div class="preview-title">
        <h2>Preview</h2>
        <Button
          variant="secondary"
          className="refresh-button"
          onClick={onRefresh}
          disabled={!controllerReady || rendering || state.status === 'loading'}
          aria-label="Refresh preview"
          title="Refresh preview">
          <svg viewBox="0 0 20 20" aria-hidden="true" focusable="false">
            <path d="M16 5v4h-4" />
            <path d="M16 9a6.5 6.5 0 1 0 1 4" />
          </svg>
        </Button>
      </div>
      <div class="preview-actions">
        <Button
          variant="secondary"
          className="mobile-source-toggle"
          onClick={onToggleAdvanced}
          disabled={!lastGeneratedSource}>
          Source
        </Button>
        {#if state.lastSuccess}
          <Button variant="secondary" onClick={openFullPage}>Full page</Button>
        {/if}
        {#if state.lastSuccess?.representation === 'pdf'}
          <Button variant="text" onClick={onDownloadPdf}>Download PDF</Button>
        {/if}
      </div>
    </div>
    <div class="preview-body">
      <PreviewPane {state} {onDiagnosticSelect} />
    </div>
  {/if}
</section>

{#if fullPage}
  <dialog
    class="preview-modal"
    aria-labelledby="full-page-preview-title"
    aria-modal="true"
    bind:this={fullPageDialog}
    on:cancel|preventDefault={closeFullPage}
    on:close={() => (fullPage = false)}>
    <div class="preview-modal-header">
      <h2 id="full-page-preview-title">Full-page preview</h2>
      <Button variant="secondary" onClick={closeFullPage}>Close</Button>
    </div>
    <div class="preview-modal-body">
      <PreviewPane {state} {onDiagnosticSelect} />
    </div>
  </dialog>
{/if}

<style>
  .preview-panel {
    display: flex;
    min-height: 0;
    flex-direction: column;
    background: #e5eaf0;
  }

  .preview-header,
  .source-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid var(--rule);
    padding: 14px 20px;
    background: var(--surface-subtle);
  }

  .preview-header h2,
  .source-header h2 {
    margin: 0;
  }

  .preview-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  :global(.refresh-button) {
    width: 36px;
    height: 36px;
    padding: 0;
  }

  :global(.refresh-button svg) {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-width: 1.7;
  }

  .preview-header h2,
  .source-header h2 {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .preview-actions,
  .source-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  :global(.mobile-source-toggle) {
    display: none;
  }

  .preview-body {
    min-height: 0;
    flex: 1;
  }

  .preview-modal {
    display: flex;
    width: min(1180px, calc(100vw - 32px));
    height: min(92vh, 960px);
    max-width: none;
    margin: auto;
    flex-direction: column;
    border: 1px solid var(--rule-strong);
    padding: 0;
    background: var(--surface);
    box-shadow: 0 24px 80px rgb(23 33 43 / 24%);
    color: var(--ink);
  }

  .preview-modal::backdrop {
    background: rgb(23 33 43 / 54%);
  }

  .preview-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid var(--rule);
    padding: 14px 20px;
    background: var(--surface-subtle);
  }

  .preview-modal-header h2 {
    margin: 0;
    font-size: 20px;
    letter-spacing: -0.03em;
  }

  .preview-modal-body {
    min-height: 0;
    flex: 1;
  }

  .source-panel {
    display: flex;
    min-height: 0;
    height: 100%;
    flex-direction: column;
    background: #18222d;
    color: #f1f5f8;
  }

  .source-panel .source-header {
    border-color: #354554;
    background: #202d39;
  }

  .source-panel .source-header h2 {
    color: #fff;
  }

  :global(.source-panel .source-action) {
    border-color: #66798a;
    background: transparent;
    color: #e8f1ff;
  }

  .source-code {
    min-height: 0;
    flex: 1;
    margin: 0;
    overflow: auto;
    padding: 22px;
    color: #d5e2ee;
    font-family: var(--mono);
    font-size: 11px;
    line-height: 1.65;
    white-space: pre-wrap;
  }

  @media (max-width: 900px) {
    .mobile-hidden {
      display: none !important;
    }

    .preview-panel {
      height: 100%;
    }
  }

  @media (max-width: 560px) {
    :global(.mobile-source-toggle) {
      display: inline-flex;
    }

    .preview-header,
    .source-header {
      align-items: flex-start;
      flex-direction: column;
      padding: 13px 16px;
    }

    .preview-actions,
    .source-actions {
      width: 100%;
      justify-content: space-between;
    }

    .preview-modal {
      width: 100vw;
      height: 100vh;
      border: 0;
    }

    .preview-modal-header {
      padding: 13px 16px;
    }
  }
</style>

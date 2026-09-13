<script lang="ts">
  import Diagnostics from './Diagnostics.svelte';
  import PdfPreview from './PdfPreview.svelte';
  import type { PreviewState } from '$lib/workspace/previewController';

  export let state: PreviewState;
  export let onDiagnosticSelect: (line: number, column: number) => void;
</script>

<div class="preview-root" aria-label="Rendered document preview">
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div class="preview-scroll" tabindex="0" role="region" aria-label="Preview scroll area">
    {#if state.status === 'loading'}
      {#if state.lastSuccess}
        <PdfPreview
          data={state.lastSuccess.data.slice(0)}
          pageCount={state.lastSuccess.pageCount} />
      {/if}
      <div class="preview-loading" role="status" aria-live="polite">
        <span class="loading-mark" aria-hidden="true"></span>
        <strong>Preparing preview</strong>
        <span>Compiling the latest source…</span>
      </div>
    {:else if state.status === 'failure'}
      {#if state.lastSuccess}
        <PdfPreview
          data={state.lastSuccess.data.slice(0)}
          pageCount={state.lastSuccess.pageCount} />
      {:else}
        <div class="preview-empty is-error" role="status">
          <span class="empty-mark" aria-hidden="true">!</span>
          <h3>Preview unavailable</h3>
          <p>Use the compiler notes below to resolve the issue, then preview again.</p>
        </div>
      {/if}
    {:else if state.lastSuccess}
      <PdfPreview data={state.lastSuccess.data.slice(0)} pageCount={state.lastSuccess.pageCount} />
    {:else}
      <div class="preview-empty" role="status">
        <span class="empty-mark" aria-hidden="true">—</span>
        <h3>No preview yet</h3>
        <p>Preview your CV to turn the structured form into a rendered document.</p>
      </div>
    {/if}
  </div>

  <Diagnostics diagnostics={state.diagnostics} onSelect={onDiagnosticSelect} />
</div>

<style>
  .preview-root {
    display: flex;
    height: 100%;
    min-height: 0;
    flex-direction: column;
  }

  .preview-scroll {
    position: relative;
    display: flex;
    min-height: 0;
    flex: 1;
    align-items: flex-start;
    justify-content: center;
    overflow: auto;
    padding: clamp(24px, 4vw, 48px);
    outline: none;
  }

  .preview-loading {
    position: absolute;
    inset: 0;
    z-index: 3;
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 7px;
    background: rgb(229 234 240 / 82%);
    color: var(--ink);
    backdrop-filter: blur(2px);
  }

  .preview-loading strong {
    font-size: 16px;
  }

  .preview-loading span:last-child {
    color: var(--muted-ink);
    font-size: 13px;
  }

  .loading-mark {
    width: 28px;
    height: 28px;
    border: 2px solid #b8cde9;
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .preview-empty {
    max-width: 300px;
    align-self: center;
    margin: auto;
    text-align: center;
  }

  .empty-mark {
    display: grid;
    width: 34px;
    height: 34px;
    margin: 0 auto 14px;
    place-items: center;
    border: 1px solid var(--rule-strong);
    border-radius: 50%;
    color: var(--blue);
    font-family: var(--mono);
    font-weight: 700;
  }

  .preview-empty.is-error .empty-mark {
    border-color: #efc3bf;
    color: var(--danger);
  }

  .preview-empty h3 {
    margin: 0 0 7px;
    color: var(--ink);
    font-size: 19px;
  }

  .preview-empty p {
    margin: 0;
    color: var(--muted-ink);
    font-size: 14px;
    line-height: 1.5;
  }

  @media (max-width: 560px) {
    .preview-scroll {
      padding: 14px;
    }
  }
</style>

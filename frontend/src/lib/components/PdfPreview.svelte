<script lang="ts">
  import { onDestroy } from 'svelte';

  type PdfModule = typeof import('pdfjs-dist/legacy/build/pdf.mjs');
  type PdfLoadingTask = ReturnType<PdfModule['getDocument']>;
  type PdfDocument = Awaited<PdfLoadingTask['promise']>;

  export let data: ArrayBuffer;
  export let pageCount: number | null = null;
  export let firstPageOnly = false;
  export let compact = false;
  export let onError: (message: string) => void = () => undefined;

  let host: HTMLDivElement;
  let renderId = 0;
  let document: PdfDocument | undefined;
  let loadingTask: PdfLoadingTask | undefined;
  let renderError = '';
  let isLoading = true;

  $: if (data) queueRender(data);

  function queueRender(source: ArrayBuffer) {
    const currentRender = ++renderId;
    renderError = '';
    isLoading = true;
    void renderPdf(source, currentRender).catch((error) => {
      if (currentRender !== renderId || isCancellation(error)) return;
      renderError =
        error instanceof Error ? error.message : 'The PDF preview could not be displayed.';
      isLoading = false;
      onError(renderError);
    });
  }

  async function renderPdf(source: ArrayBuffer, currentRender: number) {
    const pdfjs = await import('pdfjs-dist/legacy/build/pdf.mjs');
    const worker = await import('pdfjs-dist/legacy/build/pdf.worker.min.mjs?url');
    if (currentRender !== renderId) return;
    pdfjs.GlobalWorkerOptions.workerSrc = worker.default;
    await loadingTask?.destroy();
    await document?.cleanup();
    if (currentRender !== renderId) return;
    document = undefined;
    loadingTask = undefined;
    host?.replaceChildren();

    loadingTask = pdfjs.getDocument({ data: new Uint8Array(source) });
    const loaded = await loadingTask.promise;
    if (currentRender !== renderId) {
      await loaded.cleanup();
      return;
    }
    document = loaded;

    const lastPage = firstPageOnly ? 1 : loaded.numPages;
    for (let pageNumber = 1; pageNumber <= lastPage; pageNumber += 1) {
      if (currentRender !== renderId) return;
      const page = await loaded.getPage(pageNumber);
      const initialViewport = page.getViewport({ scale: 1 });
      const availableWidth = Math.max(280, (host?.clientWidth ?? 760) - 32);
      const scale = Math.min(1.75, availableWidth / initialViewport.width);
      const viewport = page.getViewport({ scale });
      const canvas = window.document.createElement('canvas');
      canvas.className =
        'block h-auto max-w-full bg-white shadow-[0_2px_2px_rgba(23,33,43,0.12),0_15px_38px_rgba(23,33,43,0.14)]';
      canvas.width = Math.ceil(viewport.width);
      canvas.height = Math.ceil(viewport.height);
      canvas.setAttribute('aria-label', `PDF page ${pageNumber}`);
      host.appendChild(canvas);
      await page.render({ canvas, viewport }).promise;
    }
    isLoading = false;
  }

  function isCancellation(error: unknown) {
    if (!(error instanceof Error)) return false;
    return /abort|cancel|destroy/i.test(`${error.name} ${error.message}`);
  }

  onDestroy(() => {
    renderId += 1;
    void loadingTask?.destroy().catch(() => undefined);
    void document?.cleanup().catch(() => undefined);
  });
</script>

<div
  class:compact
  class="pdf-pages"
  bind:this={host}
  role="img"
  aria-label={pageCount ? `PDF preview, ${pageCount} pages` : 'PDF preview'}>
  {#if renderError || isLoading}<div class="pdf-status" aria-live="polite">
      {renderError || 'Loading preview…'}
    </div>{/if}
</div>

<style>
  .pdf-pages {
    display: grid;
    width: min(100%, 820px);
    justify-items: center;
    gap: 24px;
  }

  .pdf-pages.compact {
    width: 100%;
    gap: 0;
  }

  .pdf-status {
    padding: 18px;
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .pdf-pages.compact .pdf-status {
    padding: 12px;
    font-size: 10px;
  }
</style>

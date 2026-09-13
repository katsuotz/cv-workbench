<script lang="ts">
  import { tick } from 'svelte';
  import type { CvTemplateSummary } from '$lib/api';
  import { RadioCard } from './base';

  export let templates: CvTemplateSummary[] = [];
  export let selectedId = '';
  export let onSelect: (templateId: string) => void;

  const templateImages: Record<string, string> = {
    'editorial-v1': '/templates/editorial-v1.webp',
    'compact-v1': '/templates/compact-v1.webp',
    'modern-v1': '/templates/modern-v1.webp'
  };

  let previewTemplateId: string | null = null;
  let previewDialog: HTMLDialogElement;
  let previewTemplate: CvTemplateSummary | undefined;
  let previewImage = '';

  $: previewTemplate = templates.find((template) => template.id === previewTemplateId);
  $: previewImage = previewTemplate ? (templateImages[previewTemplate.id] ?? '') : '';

  function openPreview(templateId: string) {
    if (!templateImages[templateId]) return;
    previewTemplateId = templateId;
    void tick().then(() => {
      if (previewDialog && !previewDialog.open) {
        if (typeof previewDialog.showModal === 'function') previewDialog.showModal();
        else previewDialog.setAttribute('open', '');
      }
      previewDialog?.focus();
    });
  }

  function closePreview() {
    if (previewDialog && typeof previewDialog.close === 'function') previewDialog.close();
    else previewDialog?.removeAttribute('open');
    previewTemplateId = null;
  }
</script>

<fieldset class="template-picker" aria-describedby="template-picker-help">
  <legend>Choose a template</legend>
  <p id="template-picker-help" class="template-picker-help">
    Choose a layout, then preview your CV.
  </p>
  <div class="template-grid" role="radiogroup" aria-label="CV templates">
    {#each templates as template (template.id)}
      <div class="template-option">
        <RadioCard
          className="template-card"
          label={template.name}
          description={template.description}
          name="cv-template"
          value={template.id}
          checked={selectedId === template.id}
          aria-label={`Use ${template.name} template`}
          onChange={() => onSelect(template.id)}>
          <span class="template-preview">
            {#if templateImages[template.id]}
              <img
                src={templateImages[template.id]}
                alt={`${template.name} CV template preview`}
                width="612"
                height="792"
                loading="eager"
                decoding="async" />
            {:else}
              <span class="template-fallback">
                <span class="fallback-mark">{template.name.slice(0, 1).toUpperCase()}</span>
                <span>No preview available</span>
              </span>
            {/if}
          </span>
        </RadioCard>
        {#if templateImages[template.id]}
          <button
            class="template-preview-action"
            type="button"
            aria-label={`View larger preview of ${template.name}`}
            on:click={() => openPreview(template.id)}>
            View preview
          </button>
        {/if}
      </div>
    {/each}
  </div>
</fieldset>

{#if previewTemplate && previewImage}
  <dialog
    class="template-preview-modal"
    aria-labelledby="template-preview-title"
    aria-modal="true"
    bind:this={previewDialog}
    on:cancel|preventDefault={closePreview}
    on:close={() => (previewTemplateId = null)}>
    <div class="template-preview-modal-header">
      <div>
        <h2 id="template-preview-title">{previewTemplate.name}</h2>
        <p>{previewTemplate.description}</p>
      </div>
      <button class="template-preview-close" type="button" on:click={closePreview}>Close</button>
    </div>
    <div class="template-preview-modal-body">
      <img
        class="template-preview-modal-image"
        src={previewImage}
        alt={`${previewTemplate.name} CV template preview`}
        width="612"
        height="792"
        decoding="async" />
    </div>
  </dialog>
{/if}

<style>
  .template-picker {
    min-width: 0;
    margin: 0 0 28px;
    border: 0;
    padding: 0;
  }

  legend {
    margin-bottom: 7px;
    color: var(--ink);
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .template-picker-help {
    margin: 0 0 14px;
    color: var(--muted-ink);
    font-size: 13px;
    line-height: 1.45;
  }

  .template-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
  }

  :global(.template-card) {
    position: relative;
    display: grid;
    min-width: 0;
    border: 1px solid var(--rule-strong);
    border-radius: 8px;
    padding: 8px;
    background: var(--surface-raised);
    color: var(--ink);
    cursor: pointer;
    text-align: left;
    transition:
      border-color 180ms ease,
      box-shadow 180ms ease,
      transform 180ms ease;
  }

  :global(.template-card:hover) {
    border-color: var(--blue);
    transform: translateY(-1px);
  }

  :global(.template-card:focus-within) {
    border-color: var(--blue);
    box-shadow: 0 0 0 3px rgb(23 105 210 / 14%);
  }

  :global(.template-card.selected) {
    border-color: var(--blue);
    box-shadow: 0 0 0 3px rgb(23 105 210 / 14%);
  }

  .template-preview {
    display: grid;
    height: 280px;
    place-items: center;
    overflow: hidden;
    border: 1px solid var(--rule);
    background: #e5eaf0;
  }

  .template-preview img {
    display: block;
    width: min(100%, 230px);
    height: 100%;
    object-fit: contain;
    border: 1px solid rgb(23 33 43 / 10%);
    box-shadow: 0 5px 14px rgb(23 33 43 / 15%);
  }

  .template-option {
    min-width: 0;
  }

  .template-preview-action {
    display: block;
    width: 100%;
    margin-top: 8px;
    border: 1px solid var(--rule-strong);
    border-radius: 6px;
    padding: 8px 10px;
    background: var(--surface);
    color: var(--blue-dark);
    cursor: pointer;
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    transition:
      border-color 180ms ease,
      background-color 180ms ease,
      color 180ms ease;
  }

  .template-preview-action:hover {
    border-color: var(--blue);
    background: var(--blue-soft);
    color: var(--blue);
  }

  .template-preview-action:focus-visible,
  .template-preview-close:focus-visible {
    outline: 3px solid rgb(23 105 210 / 24%);
    outline-offset: 2px;
  }

  .template-fallback {
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    letter-spacing: 0.05em;
    text-align: center;
    text-transform: uppercase;
  }

  .template-fallback {
    display: grid;
    justify-items: center;
    gap: 10px;
  }

  .fallback-mark {
    display: grid;
    width: 42px;
    height: 42px;
    place-items: center;
    border: 1px solid var(--rule-strong);
    color: var(--blue-dark);
    font-size: 19px;
    font-weight: 700;
  }

  .template-preview-modal {
    display: flex;
    width: min(960px, calc(100vw - 32px));
    height: min(92vh, 920px);
    max-width: none;
    margin: auto;
    flex-direction: column;
    border: 1px solid var(--rule-strong);
    padding: 0;
    background: var(--surface);
    box-shadow: 0 24px 80px rgb(23 33 43 / 24%);
    color: var(--ink);
  }

  .template-preview-modal::backdrop {
    background: rgb(23 33 43 / 54%);
  }

  .template-preview-modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    border-bottom: 1px solid var(--rule);
    padding: 14px 20px;
    background: var(--surface-subtle);
  }

  .template-preview-modal-header h2 {
    margin: 0;
    font-size: 20px;
    letter-spacing: -0.03em;
  }

  .template-preview-modal-header p {
    max-width: 560px;
    margin: 4px 0 0;
    color: var(--muted-ink);
    font-size: 13px;
    line-height: 1.4;
  }

  .template-preview-close {
    flex: 0 0 auto;
    border: 1px solid var(--rule-strong);
    border-radius: 6px;
    padding: 8px 12px;
    background: var(--surface);
    color: var(--blue-dark);
    cursor: pointer;
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .template-preview-close:hover {
    border-color: var(--blue);
    color: var(--blue);
  }

  .template-preview-modal-body {
    min-height: 0;
    flex: 1;
    overflow: auto;
    padding: clamp(20px, 4vw, 40px);
    background: var(--canvas);
  }

  .template-preview-modal-image {
    display: block;
    width: min(100%, 612px);
    height: auto;
    margin: 0 auto;
    border: 1px solid rgb(23 33 43 / 10%);
    box-shadow: 0 12px 18px rgb(23 33 43 / 14%);
  }

  @media (max-width: 560px) {
    .template-grid {
      grid-template-columns: 1fr;
    }

    :global(.template-card) {
      grid-template-columns: 108px minmax(0, 1fr);
      column-gap: 12px;
    }

    :global(.template-card .radio-card-content) {
      display: contents;
    }

    .template-preview {
      grid-row: span 3;
      height: 148px;
    }

    .template-preview-modal {
      width: 100vw;
      height: 100vh;
      border: 0;
    }

    .template-preview-modal-header {
      align-items: flex-start;
      padding: 13px 16px;
    }

    .template-preview-modal-header p {
      display: none;
    }

    .template-preview-modal-body {
      padding: 14px;
    }
  }
</style>

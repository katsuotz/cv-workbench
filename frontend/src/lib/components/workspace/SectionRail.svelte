<script lang="ts">
  import { Button } from '../base';
  import type { CvSectionId } from '$lib/cv/model';

  export let sections: CvSectionId[] = [];
  export let labels: Record<CvSectionId, string>;
  export let activeSection: CvSectionId;
  export let sectionErrors: (section: CvSectionId) => number;
  export let onSelect: (section: CvSectionId) => void;
  export let previewStatus: 'idle' | 'loading' | 'success' | 'empty' | 'failure';
  export let rendering = false;
  export let controllerReady = false;
  export let onPreview: () => void;
</script>

<nav class="section-rail" aria-label="CV sections">
  <div class="section-nav-list">
    {#each sections as section, index}
      {@const errorCount = sectionErrors(section)}
      <Button
        variant="text"
        className={`section-nav-button ${activeSection === section ? 'is-active' : ''} ${errorCount > 0 ? 'has-error' : ''}`}
        onClick={() => onSelect(section)}
        aria-current={activeSection === section ? 'step' : undefined}>
        <span>{labels[section]}</span>
        <span class="section-index">
          {errorCount ? `!${errorCount}` : String(index + 1).padStart(2, '0')}
        </span>
      </Button>
    {/each}
  </div>
  <div class="section-rail-actions">
    <Button
      variant="primary"
      className="rail-preview-button"
      onClick={onPreview}
      disabled={!controllerReady || rendering || previewStatus === 'loading'}>
      {rendering || previewStatus === 'loading' ? 'Previewing…' : 'Preview'}
    </Button>
  </div>
</nav>

<style>
  .section-rail {
    display: flex;
    height: 100%;
    flex-direction: column;
    padding: 27px 14px;
    border-right: 1px solid var(--rule);
    background: var(--surface-subtle);
  }

  .section-nav-list {
    display: grid;
    gap: 2px;
  }

  .section-rail-actions {
    margin-top: 18px;
  }

  :global(.rail-preview-button) {
    width: 100%;
  }

  .section-index {
    color: var(--quiet-ink);
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    line-height: 1.3;
    text-transform: uppercase;
  }

  :global(.section-nav-button) {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2px;
    border: 0;
    border-radius: 6px;
    border-left: 2px solid transparent;
    padding: 11px 10px;
    background: transparent;
    color: var(--muted-ink);
    font-size: 14px;
    text-align: left;
    transition:
      background-color 180ms ease,
      border-color 180ms ease,
      color 180ms ease;
  }

  :global(.section-nav-button.text-button) {
    font-size: 13px;
  }

  :global(.section-nav-button:hover) {
    background: var(--surface);
    color: var(--ink);
  }

  :global(.section-nav-button.is-active) {
    border-left-color: var(--blue);
    background: var(--blue-soft);
    color: var(--blue-dark);
    font-weight: 650;
  }

  :global(.section-nav-button.has-error .section-index) {
    color: var(--danger);
  }

  @media (max-width: 900px) {
    .section-rail {
      display: none;
    }
  }
</style>

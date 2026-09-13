<script lang="ts">
  import { Button } from './base';

  export let status: 'idle' | 'loading' | 'success' | 'empty' | 'failure';
  export let dirty: boolean;
  export let onCompile: () => void;
</script>

<header class="toolbar" aria-label="Document toolbar">
  <div class="toolbar-brand">
    <span class="toolbar-mark" aria-hidden="true">CV</span>
    <div>
      <p>CV Workbench</p>
      <span>
        {dirty ? 'Unpreviewed changes' : status === 'success' ? 'Preview complete' : 'Draft'}
      </span>
    </div>
  </div>
  <div class="toolbar-status" role="status">
    {status === 'loading'
      ? 'Setting type'
      : status === 'failure'
        ? 'Needs attention'
        : status === 'success'
          ? 'Preview complete'
          : 'Draft'}
  </div>
  <Button
    variant="primary"
    className="toolbar-compile"
    onClick={onCompile}
    disabled={status === 'loading'}>
    {status === 'loading' ? 'Rendering…' : 'Preview'}
  </Button>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    min-height: 72px;
    border-bottom: 1px solid var(--rule);
    padding: 0 28px;
    background: var(--surface);
  }

  .toolbar-brand,
  .toolbar-brand > div {
    display: flex;
    align-items: center;
  }

  .toolbar-brand {
    gap: 10px;
  }

  .toolbar-brand > div {
    align-items: flex-start;
    flex-direction: column;
    gap: 2px;
  }

  .toolbar-mark {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid var(--blue);
    color: var(--blue);
    font-weight: 700;
  }

  .toolbar p,
  .toolbar span {
    margin: 0;
  }

  .toolbar p {
    font-size: 17px;
    font-weight: 700;
  }

  .toolbar-brand span:last-child,
  .toolbar-status,
  :global(.toolbar-compile) {
    font-family: var(--mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .toolbar-brand span:last-child,
  .toolbar-status {
    color: var(--muted-ink);
  }

  :global(.toolbar-compile) {
    min-height: 36px;
    border: 0;
    border-radius: 7px;
    padding: 0 14px;
    background: var(--blue);
    color: #fff;
    font-weight: 600;
  }

  :global(.toolbar-compile:disabled) {
    opacity: 0.5;
  }
</style>

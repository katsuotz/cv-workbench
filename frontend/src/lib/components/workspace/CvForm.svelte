<script lang="ts">
  import type { CvData, CvSectionId, CvValidationError } from '$lib/cv/model';
  import type { CvTemplateSummary } from '$lib/api';
  import { Button } from '../base';
  import SummarySection from './SummarySection.svelte';
  import ExperienceSection from './ExperienceSection.svelte';
  import AchievementsSection from './AchievementsSection.svelte';
  import SkillsSection from './SkillsSection.svelte';
  import EducationSection from './EducationSection.svelte';
  import CertificatesSection from './CertificatesSection.svelte';
  import ProjectsSection from './ProjectsSection.svelte';

  type EntrySection = Exclude<CvSectionId, 'summary'>;

  export let data: CvData;
  export let templates: CvTemplateSummary[] = [];
  export let templateId = '';
  export let onSelectTemplate: (templateId: string) => void;
  export let activeSection: CvSectionId;
  export let presentation: 'intake' | 'workspace';
  export let sections: CvSectionId[];
  export let sectionIndex: number;
  export let labels: Record<CvSectionId, string>;
  export let descriptions: Record<CvSectionId, string>;
  export let errors: CvValidationError[] = [];
  export let notice = '';
  export let fieldError: (path: string) => string | undefined;
  export let linkedinEnabled = true;
  export let onInput: () => void;
  export let onAdd: (section: EntrySection) => void;
  export let onRemove: (section: EntrySection, id: string) => void;
  export let onMove: (section: EntrySection, index: number, direction: -1 | 1) => void;
  export let onAddProfile: () => void;
  export let onRemoveProfile: (id: string) => void;
  export let onMoveProfile: (index: number, direction: -1 | 1) => void;
  export let importBusy = false;
  export let importNotice = '';
  export let onImportFromLinkedIn: (event: MouseEvent) => void | Promise<void>;
  export let onPrevious: () => void;
  export let onNext: () => void;

  const sectionErrorCount = (section: CvSectionId) =>
    errors.filter((error) => error.section === section).length;
</script>

<div class="builder-scroll" on:input={onInput}>
  <div class="form-content">
    <div class="form-heading">
      <p class="section-kicker">Section {sectionIndex + 1} of {sections.length}</p>
      <h2>
        {presentation === 'intake' && activeSection === 'summary'
          ? 'Start with the essentials'
          : labels[activeSection]}
      </h2>
      <p>
        {presentation === 'intake' && activeSection === 'summary'
          ? 'Add the details that make your CV yours. You can fill the rest in as you go.'
          : descriptions[activeSection]}
      </p>
    </div>
    {#if errors.length}
      <p class="form-alert" role="alert">
        <strong>Review this section.</strong>
        {errors[0].message}
      </p>
    {/if}
    {#if notice}<p class="notice" role="status">{notice}</p>{/if}

    {#if activeSection === 'summary'}
      <SummarySection
        {data}
        {templates}
        selectedTemplateId={templateId}
        {onSelectTemplate}
        {fieldError}
        {onAddProfile}
        {onRemoveProfile}
        {onMoveProfile}
        {linkedinEnabled}
        {importBusy}
        {importNotice}
        {onImportFromLinkedIn} />
    {:else if activeSection === 'experience'}
      <ExperienceSection
        {data}
        {fieldError}
        onMove={(index, direction) => onMove('experience', index, direction)}
        onRemove={(id) => onRemove('experience', id)}
        onAdd={() => onAdd('experience')} />
    {:else if activeSection === 'achievements'}
      <AchievementsSection
        {data}
        {fieldError}
        onMove={(index, direction) => onMove('achievements', index, direction)}
        onRemove={(id) => onRemove('achievements', id)}
        onAdd={() => onAdd('achievements')} />
    {:else if activeSection === 'skills'}
      <SkillsSection
        {data}
        {fieldError}
        onMove={(index, direction) => onMove('skills', index, direction)}
        onRemove={(id) => onRemove('skills', id)}
        onAdd={() => onAdd('skills')} />
    {:else if activeSection === 'education'}
      <EducationSection
        {data}
        {fieldError}
        onMove={(index, direction) => onMove('education', index, direction)}
        onRemove={(id) => onRemove('education', id)}
        onAdd={() => onAdd('education')} />
    {:else if activeSection === 'certificates'}
      <CertificatesSection
        {data}
        {fieldError}
        onMove={(index, direction) => onMove('certificates', index, direction)}
        onRemove={(id) => onRemove('certificates', id)}
        onAdd={() => onAdd('certificates')} />
    {:else if activeSection === 'projects'}
      <ProjectsSection
        {data}
        {fieldError}
        onMove={(index, direction) => onMove('projects', index, direction)}
        onRemove={(id) => onRemove('projects', id)}
        onAdd={() => onAdd('projects')} />
    {/if}

    <footer class="section-footer">
      <Button variant="text" onClick={onPrevious} disabled={sectionIndex === 0}>← Previous</Button>
      <span class="page-count">{sectionIndex + 1} / {sections.length}</span>
      <Button variant="text" onClick={onNext} disabled={sectionIndex === sections.length - 1}>
        Next →
      </Button>
    </footer>
  </div>
</div>

<style>
  .builder-scroll {
    min-height: 0;
    overflow-y: auto;
    padding: 44px clamp(28px, 5vw, 84px) 52px;
  }

  .form-content {
    width: min(100%, 760px);
    margin: 0 auto;
  }

  .form-heading {
    margin-bottom: 30px;
  }

  .form-heading h2,
  .form-heading p {
    margin: 0;
  }

  .section-kicker,
  .page-count {
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    line-height: 1.3;
    text-transform: uppercase;
  }

  .section-kicker {
    margin-bottom: 10px !important;
    color: var(--blue-dark);
    font-size: 11px;
  }

  .form-heading h2 {
    font-size: 34px;
    font-weight: 700;
    letter-spacing: -0.04em;
    line-height: 1.08;
  }

  .form-heading p:not(.section-kicker) {
    max-width: unset;
    margin-top: 10px;
    color: var(--muted-ink);
    font-size: 16px;
    line-height: 1.5;
  }

  .form-alert,
  .notice {
    margin: 0 0 24px;
    border: 1px solid var(--rule);
    border-radius: 7px;
    padding: 12px 14px;
    font-size: 13px;
    line-height: 1.45;
  }

  .form-alert {
    border-color: #efc3bf;
    background: var(--danger-soft);
    color: var(--danger);
  }

  .notice {
    background: var(--surface-subtle);
    color: var(--muted-ink);
  }

  .section-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 34px;
    border-top: 1px solid var(--rule);
    padding-top: 16px;
  }

  .page-count {
    color: var(--quiet-ink);
    font-size: 10px;
  }

  @media (max-width: 900px) {
    .builder-scroll {
      padding: 32px 24px 44px;
    }

    .form-content {
      width: min(100%, 700px);
    }
  }

  @media (max-width: 560px) {
    .builder-scroll {
      padding: 28px 16px 36px;
    }

    .form-heading h2 {
      font-size: 30px;
    }
  }
</style>

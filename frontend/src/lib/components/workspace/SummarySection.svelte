<script lang="ts">
  import type { CvData, ProfileLink } from '$lib/cv/model';
  import type { CvTemplateSummary } from '$lib/api';
  import { Button, SelectField, TextField, TextareaField } from '../base';
  import Field from '../cv/Field.svelte';
  import TemplatePicker from '../TemplatePicker.svelte';

  export let data: CvData;
  export let templates: CvTemplateSummary[] = [];
  export let selectedTemplateId = '';
  export let onSelectTemplate: (templateId: string) => void;
  export let fieldError: (path: string) => string | undefined;
  export let onAddProfile: () => void;
  export let onRemoveProfile: (id: string) => void;
  export let onMoveProfile: (index: number, direction: -1 | 1) => void;

  const profileTypes = [
    ['website', 'Website'],
    ['linkedin', 'LinkedIn'],
    ['github', 'GitHub'],
    ['portfolio', 'Portfolio'],
    ['x', 'X'],
    ['other', 'Other']
  ];

  const profileError = (profile: ProfileLink, suffix: 'type' | 'url') =>
    fieldError(`profiles.${profile.id}.${suffix}`);
  $: contactRequired =
    !data.identity.email.trim() &&
    !data.identity.phone.trim() &&
    !data.identity.profiles.some((profile) => profile.url.trim());
</script>

{#if templates.length}
  <TemplatePicker {templates} selectedId={selectedTemplateId} onSelect={onSelectTemplate} />
{/if}

<div class="form-grid">
  <TextField
    wide
    label="Full name"
    required
    path="identity.fullName"
    error={fieldError('identity.fullName')}
    autocomplete="name"
    placeholder="Ada Lovelace"
    bind:value={data.identity.fullName} />
  <TextField
    label="Professional titles"
    placeholder="Design engineer · Researcher"
    bind:value={data.identity.professionalTitles} />
  <TextField
    label="Location"
    autocomplete="address-level2"
    placeholder="London, UK"
    bind:value={data.identity.location} />
  <TextField
    label="Email"
    type="email"
    path="identity.email"
    error={fieldError('identity.email')}
    required={contactRequired}
    autocomplete="email"
    placeholder="ada@example.com"
    bind:value={data.identity.email} />
  <TextField
    label="Phone"
    autocomplete="tel"
    placeholder="+44 20 0000 0000"
    bind:value={data.identity.phone} />
  <TextareaField
    wide
    label="Professional summary"
    placeholder="A concise account of the work you do and the value you create."
    bind:value={data.summary} />
</div>

<div class="profile-links">
  <div class="profile-links-heading">
    <h3>Profile links</h3>
    <Button variant="text" onClick={onAddProfile}>+ Add link</Button>
  </div>
  {#each data.identity.profiles as profile, profileIndex (profile.id)}
    <div class="entry-card profile-link-card">
      <SelectField
        label="Type"
        className="profile-type"
        options={profileTypes.map(([value, label]) => ({ value, label }))}
        error={profileError(profile, 'type')}
        path={`profiles.${profile.id}.type`}
        bind:value={profile.type} />
      <Field label="Label" bind:value={profile.label} />
      <Field
        label="URL"
        type="url"
        required
        path={`profiles.${profile.id}.url`}
        error={profileError(profile, 'url')}
        bind:value={profile.url} />
      <div class="profile-link-actions">
        <Button
          variant="text"
          type="button"
          aria-label="Move profile link up"
          disabled={profileIndex === 0}
          onClick={() => onMoveProfile(profileIndex, -1)}>
          ↑
        </Button>
        <Button
          variant="text"
          type="button"
          aria-label="Move profile link down"
          disabled={profileIndex === data.identity.profiles.length - 1}
          onClick={() => onMoveProfile(profileIndex, 1)}>
          ↓
        </Button>
        <Button
          variant="text"
          className="remove-button"
          type="button"
          aria-label="Remove profile link"
          onClick={() => onRemoveProfile(profile.id)}>
          ×
        </Button>
      </div>
    </div>
  {/each}
</div>

<style>
  .profile-links {
    margin-top: 36px;
    border-top: 1px solid var(--rule);
    padding-top: 24px;
  }

  .profile-links-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  .profile-links-heading h3 {
    margin: 0;
    font-size: 20px;
  }

  .profile-link-card {
    display: grid;
    grid-template-columns: 130px 1fr 1.5fr auto;
    gap: 12px;
  }

  .profile-link-actions {
    display: flex;
    align-items: center;
  }

  @media (max-width: 620px) {
    .profile-link-card {
      grid-template-columns: 1fr;
    }
  }
</style>

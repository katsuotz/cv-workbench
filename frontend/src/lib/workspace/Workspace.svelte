<script lang="ts">
  import { onMount, tick } from 'svelte';
  import {
    BackendApiError,
    createBackendApi,
    type AuthUser,
    type CvSessionDraft,
    type CvSessionResponse,
    type CvTemplateSummary,
    type LinkedInPendingImport
  } from '$lib/api';
  import {
    blankCv,
    entryId,
    newEntry,
    SECTION_ORDER,
    validateCv,
    type CvData,
    type CvSectionId,
    type CvValidationError
  } from '$lib/cv/model';
  import { fingerprintCv } from '$lib/cv/storage';
  import { BackendPreviewAdapter } from '$lib/preview/backendPreviewAdapter';
  import { AccountService } from './accountService';
  import {
    copySource as copySourceToClipboard,
    downloadPdf as downloadPdfFile,
    downloadText as downloadTextFile
  } from './downloads';
  import { PreviewController, type PreviewState } from '$lib/workspace/previewController';
  import { setupPreview } from './previewSetup';
  import { SessionController } from './sessionController';
  import WorkspaceHeader from '../components/workspace/WorkspaceHeader.svelte';
  import WorkspaceMobilePaneNav from '../components/workspace/WorkspaceMobilePaneNav.svelte';
  import { Button } from '../components/base';
  import SectionRail from '../components/workspace/SectionRail.svelte';
  import CvForm from '../components/workspace/CvForm.svelte';
  import WorkspacePreviewPanel from '../components/workspace/WorkspacePreviewPanel.svelte';

  type EntrySection = Exclude<CvSectionId, 'summary'>;
  const labels: Record<CvSectionId, string> = {
    summary: 'Summary',
    experience: 'Experience',
    achievements: 'Achievements',
    skills: 'Skills',
    education: 'Education',
    certificates: 'Certificates',
    projects: 'Projects'
  };
  const descriptions: Record<CvSectionId, string> = {
    summary: 'The headline and human story.',
    experience: 'Roles, impact, and the tools behind it.',
    achievements: 'Recognition and measurable milestones.',
    skills: 'Capabilities grouped for quick reading.',
    education: 'Formal study and qualifications.',
    certificates: 'Credentials that support your practice.',
    projects: 'Selected work beyond your role history.'
  };
  let data: CvData = blankCv();
  let templateCatalog: CvTemplateSummary[] = [];
  let templateId = '';
  let generatedTemplateId: string | null = null;
  let activeSection: CvSectionId = 'summary';
  let mobilePane: 'form' | 'preview' = 'form';
  let presentation: 'intake' | 'workspace' = 'intake';
  let advanced = false;
  let lastGeneratedSource = '';
  let generatedFingerprint = '';
  let generatedAt: string | null = null;
  let autosaveStatus: 'idle' | 'saving' | 'saved' | 'error' = 'idle';
  let authUser: AuthUser | null = null;
  let authOpen = false;
  let authMode: 'login' | 'register' = 'login';
  let authEmail = '';
  let authPassword = '';
  let authName = '';
  let authBusy = false;
  let authNotice = '';
  let importBusy = false;
  let importNotice = '';
  let pendingLinkedInImport: LinkedInPendingImport | null = null;
  let importDialog: HTMLElement;
  let errors: CvValidationError[] = [];
  let notice = '';
  let state: PreviewState = {
    status: 'idle',
    requestedSource: '',
    lastSuccess: null,
    diagnostics: []
  };
  const backendApi = createBackendApi();
  const backendAdapter = new BackendPreviewAdapter(backendApi.documents, backendApi.compilation);
  let controller: PreviewController | null = null;
  let controllerReady = false;
  let rendering = false;
  let diagnosticLine: number | null = null;
  let diagnosticColumn: number | null = null;
  let sessionController: SessionController;
  let accountService: AccountService;
  let workspaceBootstrapPending: Promise<void> | null = null;
  $: currentFingerprint = fingerprintCv(data);
  $: dirty =
    currentFingerprint !== generatedFingerprint ||
    (!!lastGeneratedSource && templateId !== generatedTemplateId);
  $: sectionIndex = SECTION_ORDER.indexOf(activeSection);

  function sessionDraft(): CvSessionDraft {
    return {
      schemaVersion: 1,
      data: structuredClone(data),
      templateId,
      generatedTemplateId,
      lastGeneratedSource,
      generatedAt,
      fingerprint: generatedFingerprint
    };
  }

  sessionController = new SessionController(backendApi.cvSession, {
    getDraft: sessionDraft,
    applySession: hydrateSession,
    onStatus: (status) => (autosaveStatus = status),
    onNotice: (message) => (notice = message)
  });
  accountService = new AccountService(
    backendApi.auth,
    backendApi.cvSession,
    sessionDraft,
    (session) => sessionController.hydrate(session)
  );

  function flushAutosave(force = false) {
    return sessionController.flush(force);
  }

  function scheduleAutosave() {
    sessionController.schedule();
  }

  function changed() {
    data = structuredClone(data);
    if (errors.length) errors = validateCv(data);
    notice = '';
    scheduleAutosave();
  }
  function add(section: EntrySection) {
    const next = structuredClone(data) as unknown as Record<
      EntrySection,
      Array<Record<string, unknown>>
    >;
    next[section].push(newEntry(section) as unknown as Record<string, unknown>);
    data = next as unknown as CvData;
    changed();
  }
  function remove(section: EntrySection, id: string) {
    const next = structuredClone(data) as unknown as Record<
      EntrySection,
      Array<Record<string, unknown>>
    >;
    next[section] = next[section].filter((entry) => entry.id !== id);
    data = next as unknown as CvData;
    changed();
  }
  function move(section: EntrySection, index: number, direction: -1 | 1) {
    const target = index + direction;
    const next = structuredClone(data) as unknown as Record<
      EntrySection,
      Array<Record<string, unknown>>
    >;
    if (target < 0 || target >= next[section].length) return;
    [next[section][index], next[section][target]] = [next[section][target], next[section][index]];
    data = next as unknown as CvData;
    changed();
  }
  function addProfile() {
    data.identity.profiles.push({ id: entryId(), type: 'website', label: '', url: '' });
    changed();
  }
  function removeProfile(id: string) {
    data.identity.profiles = data.identity.profiles.filter((profile) => profile.id !== id);
    changed();
  }
  function moveProfile(index: number, direction: -1 | 1) {
    const target = index + direction;
    if (target < 0 || target >= data.identity.profiles.length) return;
    [data.identity.profiles[index], data.identity.profiles[target]] = [
      data.identity.profiles[target],
      data.identity.profiles[index]
    ];
    changed();
  }
  const fieldError = (path: string) => errors.find((error) => error.path === path)?.message;
  const sectionErrors = (section: CvSectionId) =>
    errors.filter((error) => error.section === section).length;

  async function generate() {
    if (rendering || state.status === 'loading') return;
    if (!controllerReady || !controller) {
      notice = 'The preview workspace is still loading. Try Preview again in a moment.';
      return;
    }
    errors = validateCv(data);
    if (errors.length) {
      activeSection = errors[0].section;
      mobilePane = 'form';
      notice = 'Review the highlighted field before previewing.';
      requestAnimationFrame(() =>
        document.querySelector<HTMLElement>(`[data-path="${errors[0].path}"]`)?.focus()
      );
      return;
    }
    rendering = true;
    try {
      const rendered = await backendApi.cvRender.render({
        templateId,
        data: structuredClone(data)
      });
      lastGeneratedSource = rendered.generatedSource;
      generatedTemplateId = rendered.templateId;
      generatedAt = rendered.generatedAt;
      generatedFingerprint = currentFingerprint;
      presentation = 'workspace';
      mobilePane = 'preview';
      const saved = await flushAutosave(true);
      notice = saved
        ? 'Source generated. Loading preview…'
        : 'Remote save is unavailable; loading preview in this tab…';
      await controller.compile(lastGeneratedSource);
      notice =
        state.status === 'success'
          ? ''
          : saved
            ? 'Source generated, but the preview could not be completed.'
            : 'Source generated in this tab, but the draft could not be saved.';
    } catch (error) {
      notice = error instanceof Error ? error.message : 'Could not prepare the preview.';
    } finally {
      rendering = false;
    }
  }

  function selectTemplate(nextTemplateId: string) {
    if (nextTemplateId === templateId) return;
    templateId = nextTemplateId;
    notice = '';
    scheduleAutosave();
  }
  function downloadText() {
    if (!lastGeneratedSource) return;
    try {
      downloadTextFile(lastGeneratedSource, data.identity.fullName);
    } catch {
      notice = 'The download could not be started in this browser.';
    }
  }
  function downloadPdf() {
    if (state.lastSuccess?.representation !== 'pdf') return;
    try {
      downloadPdfFile(state.lastSuccess, data.identity.fullName);
    } catch {
      notice = 'The download could not be started in this browser.';
    }
  }
  async function copySource() {
    if (!lastGeneratedSource) return;
    try {
      await copySourceToClipboard(lastGeneratedSource);
      notice = 'Exact LaTeX source copied.';
    } catch {
      notice = 'Clipboard access was denied. Download the .tex file instead.';
    }
  }
  function toggleAdvanced() {
    advanced = !advanced;
    if (advanced) mobilePane = 'preview';
  }
  function selectDiagnostic(line: number, column: number) {
    diagnosticLine = line;
    diagnosticColumn = column;
    advanced = true;
  }
  function go(direction: -1 | 1) {
    activeSection =
      SECTION_ORDER[Math.max(0, Math.min(SECTION_ORDER.length - 1, sectionIndex + direction))];
    document.querySelector('.builder-scroll')?.scrollTo({ top: 0, behavior: 'smooth' });
  }

  function hydrateSession(session: CvSessionResponse) {
    data = session.data;
    templateId = session.templateId;
    generatedTemplateId = session.generatedTemplateId;
    lastGeneratedSource = session.lastGeneratedSource;
    generatedAt = session.generatedAt;
    generatedFingerprint = session.fingerprint || (lastGeneratedSource ? fingerprintCv(data) : '');
    presentation = lastGeneratedSource ? 'workspace' : 'intake';
  }

  function resetPreview() {
    state = { status: 'idle', requestedSource: '', lastSuccess: null, diagnostics: [] };
    advanced = false;
    mobilePane = 'form';
    diagnosticLine = null;
    diagnosticColumn = null;
  }

  function openAuth(mode: 'login' | 'register') {
    authMode = mode;
    authOpen = true;
    authNotice = '';
  }

  async function continueWithGoogle(event: MouseEvent) {
    event.preventDefault();
    if (authBusy) return;

    authBusy = true;
    authNotice = '';
    try {
      if (!sessionController.id && workspaceBootstrapPending) {
        await workspaceBootstrapPending;
      }
      if (!sessionController.id || !(await flushAutosave(true))) {
        authNotice = 'Could not save your CV before Google sign-in. Try again.';
        return;
      }
      window.location.assign(accountService.googleStartUrl());
    } catch (error) {
      authNotice = error instanceof Error ? error.message : 'Google sign-in could not be started.';
    } finally {
      authBusy = false;
    }
  }

  async function continueWithLinkedIn(event: MouseEvent) {
    event.preventDefault();
    if (authBusy) return;

    authBusy = true;
    authNotice = '';
    try {
      if (!sessionController.id && workspaceBootstrapPending) {
        await workspaceBootstrapPending;
      }
      if (!sessionController.id || !(await flushAutosave(true))) {
        authNotice = 'Could not save your CV before LinkedIn sign-in. Try again.';
        return;
      }
      window.location.assign(accountService.linkedInStartUrl('login'));
    } catch (error) {
      authNotice =
        error instanceof Error ? error.message : 'LinkedIn sign-in could not be started.';
    } finally {
      authBusy = false;
    }
  }

  async function importFromLinkedIn(event: MouseEvent) {
    event.preventDefault();
    if (importBusy) return;

    importBusy = true;
    importNotice = '';
    notice = '';
    try {
      if (!sessionController.id && workspaceBootstrapPending) {
        await workspaceBootstrapPending;
      }
      if (!sessionController.id || !(await flushAutosave(true))) {
        importNotice = 'Could not save your CV before LinkedIn import. Try again.';
        return;
      }
      window.location.assign(accountService.linkedInStartUrl('import'));
    } catch (error) {
      importNotice =
        error instanceof Error ? error.message : 'LinkedIn import could not be started.';
    } finally {
      importBusy = false;
    }
  }

  type CallbackState = {
    auth: 'success' | 'error' | null;
    provider: string | null;
    importReady: boolean;
  };

  function consumeAuthCallback(): CallbackState {
    const url = new URL(window.location.href);
    const result = url.searchParams.get('auth');
    const provider = url.searchParams.get('provider');
    const importReady = provider === 'linkedin' && url.searchParams.get('import') === 'ready';
    const hasCallback =
      result === 'success' ||
      result === 'error' ||
      provider === 'linkedin' ||
      url.searchParams.get('import') === 'ready';
    if (!hasCallback) return { auth: null, provider: null, importReady: false };

    const message = url.searchParams.get('message');
    for (const key of ['auth', 'message', 'provider', 'import', 'code', 'state']) {
      url.searchParams.delete(key);
    }
    window.history.replaceState({}, '', `${url.pathname}${url.search}${url.hash}`);

    if (result === 'error') {
      authMode = 'login';
      authOpen = true;
      authNotice =
        message ||
        (provider === 'linkedin'
          ? 'LinkedIn sign-in could not be completed. Try again.'
          : 'Google sign-in could not be completed. Try again.');
    }
    return {
      auth: result === 'success' || result === 'error' ? result : null,
      provider,
      importReady
    };
  }

  async function reviewLinkedInImport() {
    try {
      const pending = await backendApi.cvImport.getPending();
      if (!pending) {
        notice = 'No pending LinkedIn import is available. Try importing again.';
        return;
      }
      pendingLinkedInImport = pending;
      await tick();
      importDialog?.querySelector<HTMLElement>('[data-import-confirm]')?.focus();
    } catch (error) {
      notice = error instanceof Error ? error.message : 'Could not load the LinkedIn import.';
    }
  }

  function closeImportDialog() {
    pendingLinkedInImport = null;
    importNotice = '';
  }

  async function discardLinkedInImport() {
    if (!pendingLinkedInImport || importBusy) return;
    importBusy = true;
    importNotice = '';
    try {
      await backendApi.cvImport.discard(pendingLinkedInImport.id);
      closeImportDialog();
      notice = 'LinkedIn import discarded. Your current CV was not changed.';
    } catch (error) {
      importNotice = error instanceof Error ? error.message : 'Could not discard this import.';
    } finally {
      importBusy = false;
    }
  }

  async function applyLinkedInImport() {
    if (!pendingLinkedInImport || importBusy) return;
    importBusy = true;
    importNotice = '';
    try {
      const session = await backendApi.cvImport.apply(
        pendingLinkedInImport.id,
        sessionController.currentVersion
      );
      sessionController.hydrate(session);
      pendingLinkedInImport = null;
      resetPreview();
      activeSection = 'summary';
      presentation = 'intake';
      notice = 'LinkedIn import applied. Your current CV was replaced.';
    } catch (error) {
      importNotice =
        error instanceof BackendApiError && error.status === 409
          ? 'Your CV changed while this import was open. The current CV was not replaced.'
          : error instanceof Error
            ? error.message
            : 'Could not apply this LinkedIn import.';
    } finally {
      importBusy = false;
    }
  }

  async function submitAuth() {
    authBusy = true;
    authNotice = '';
    try {
      const result = await accountService.authenticate(authMode, authEmail, authPassword, authName);
      authUser = result.user;
      authOpen = false;
      authPassword = '';
      notice =
        authMode === 'login'
          ? 'Signed in; your CV session is synced.'
          : 'Account created; your CV session is synced.';
    } catch (error) {
      authNotice =
        error instanceof Error ? error.message : 'Account action could not be completed.';
    } finally {
      authBusy = false;
    }
  }

  async function logout() {
    try {
      await accountService.logout();
      authUser = null;
      notice = 'Signed out. You can keep editing anonymously.';
    } catch (error) {
      notice = error instanceof Error ? error.message : 'Could not sign out.';
    }
  }

  onMount(() => {
    let active = true;
    const callback = consumeAuthCallback();
    workspaceBootstrapPending = (async () => {
      try {
        const user = await accountService.currentUser();
        if (!active) return;
        authUser = user;

        templateCatalog = await backendApi.templates.list();
        if (!templateCatalog.length) throw new Error('No CV templates are available.');
        templateId = templateCatalog[0].id;
        await sessionController.bootstrap(sessionDraft());
        if (!active) return;
        controller = await setupPreview(
          backendAdapter,
          lastGeneratedSource,
          (next) => (state = next)
        );
        if (!active) return;
        controllerReady = true;
        if (callback.importReady) {
          await reviewLinkedInImport();
        } else if (callback.auth === 'success' && user) {
          notice =
            callback.provider === 'linkedin'
              ? 'Signed in with LinkedIn. Your CV session is synced.'
              : 'Signed in with Google. Your CV session is synced.';
        } else if (callback.auth === 'success') {
          notice = 'Sign-in could not be verified. Please try again.';
        }
      } catch (error) {
        if (!active) return;
        controllerReady = false;
        state = {
          ...state,
          status: 'failure',
          diagnostics: [
            {
              severity: 'error',
              message:
                error instanceof BackendApiError
                  ? error.message
                  : 'The preview service could not be loaded.',
              line: 1,
              column: 1,
              code: 'BACKEND_UNAVAILABLE'
            }
          ]
        };
      }
    })();
    void workspaceBootstrapPending;
    return () => {
      active = false;
      sessionController.dispose();
      controller?.dispose();
      void backendAdapter?.cancel();
    };
  });
</script>

<svelte:window
  on:keydown={(event) => {
    if (event.key === 'Escape' && pendingLinkedInImport && !importBusy) {
      event.preventDefault();
      closeImportDialog();
      return;
    }
    if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
      event.preventDefault();
      if (state.status !== 'loading') void generate();
    }
  }} />

<main class={`workspace-shell ${presentation === 'workspace' ? 'is-workspace' : 'is-intake'}`}>
  <WorkspaceHeader
    {presentation}
    {authUser}
    {authOpen}
    {authMode}
    {authEmail}
    {authPassword}
    {authName}
    {authBusy}
    {authNotice}
    googleStartUrl={accountService.googleStartUrl()}
    linkedInStartUrl={accountService.linkedInStartUrl('login')}
    {advanced}
    onAuthMode={openAuth}
    onAuthOpenChange={(open) => (authOpen = open)}
    onEmailChange={(value) => (authEmail = value)}
    onPasswordChange={(value) => (authPassword = value)}
    onNameChange={(value) => (authName = value)}
    onSubmitAuth={submitAuth}
    onGoogleAuth={continueWithGoogle}
    onLinkedInAuth={continueWithLinkedIn}
    onLogout={logout}
    onToggleAdvanced={toggleAdvanced} />

  {#if presentation === 'workspace'}
    <WorkspaceMobilePaneNav
      active={mobilePane}
      diagnosticsCount={state.diagnostics.length}
      onSelect={(pane) => (mobilePane = pane)} />
  {/if}

  <div class="workspace-content">
    <section
      class={`form-panel ${presentation === 'intake' ? 'intake-panel' : ''} ${mobilePane !== 'form' ? 'mobile-hidden' : ''}`}
      aria-label="CV form builder">
      <SectionRail
        sections={SECTION_ORDER}
        {labels}
        {activeSection}
        {sectionErrors}
        onSelect={(section) => (activeSection = section)}
        previewStatus={state.status}
        {rendering}
        {controllerReady}
        onPreview={generate} />
      <CvForm
        {data}
        templates={templateCatalog}
        {templateId}
        onSelectTemplate={selectTemplate}
        {activeSection}
        {presentation}
        sections={SECTION_ORDER}
        {sectionIndex}
        {labels}
        {descriptions}
        {errors}
        {notice}
        {fieldError}
        {importBusy}
        {importNotice}
        onImportFromLinkedIn={importFromLinkedIn}
        onInput={changed}
        onAdd={add}
        onRemove={remove}
        onMove={move}
        onAddProfile={addProfile}
        onRemoveProfile={removeProfile}
        onMoveProfile={moveProfile}
        onPrevious={() => go(-1)}
        onNext={() => go(1)} />
      <div class="mobile-preview-action">
        <Button
          variant="primary"
          onClick={generate}
          disabled={!controllerReady || rendering || state.status === 'loading'}>
          {rendering || state.status === 'loading' ? 'Previewing…' : 'Preview'}
        </Button>
      </div>
    </section>

    {#if presentation === 'workspace'}
      <WorkspacePreviewPanel
        {state}
        {advanced}
        {lastGeneratedSource}
        {rendering}
        {controllerReady}
        {diagnosticLine}
        {diagnosticColumn}
        hidden={mobilePane !== 'preview'}
        onCopySource={copySource}
        onDownloadText={downloadText}
        onDownloadPdf={downloadPdf}
        onRefresh={generate}
        onToggleAdvanced={toggleAdvanced}
        onDiagnosticSelect={selectDiagnostic} />
    {/if}
  </div>

  {#if pendingLinkedInImport}
    <div class="import-dialog-backdrop">
      <div
        bind:this={importDialog}
        class="import-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="linkedin-import-title"
        aria-describedby="linkedin-import-description">
        <div class="import-dialog-heading">
          <h2 id="linkedin-import-title">Review LinkedIn import</h2>
          <Button
            variant="text"
            aria-label="Close LinkedIn import review"
            onClick={closeImportDialog}
            disabled={importBusy}>
            ×
          </Button>
        </div>
        <p id="linkedin-import-description" class="import-dialog-warning">
          The current CV will be replaced with the LinkedIn profile when you confirm.
        </p>
        <div class="import-review">
          <strong>{pendingLinkedInImport.data.identity.fullName || 'Unnamed profile'}</strong>
          {#if pendingLinkedInImport.data.identity.email}
            <span>{pendingLinkedInImport.data.identity.email}</span>
          {/if}
          {#if pendingLinkedInImport.data.summary}
            <p>{pendingLinkedInImport.data.summary}</p>
          {/if}
        </div>
        {#if importNotice}<p class="import-dialog-error" role="alert">{importNotice}</p>{/if}
        <div class="import-dialog-actions">
          <Button variant="text" onClick={closeImportDialog} disabled={importBusy}>
            Keep current CV
          </Button>
          <Button variant="text" onClick={discardLinkedInImport} disabled={importBusy}>
            {importBusy ? 'Working…' : 'Discard import'}
          </Button>
          <Button
            variant="primary"
            data-import-confirm
            onClick={applyLinkedInImport}
            disabled={importBusy}>
            {importBusy ? 'Replacing…' : 'Replace current CV'}
          </Button>
        </div>
      </div>
    </div>
  {/if}

  <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
    {state.status === 'loading'
      ? 'Preview rendering started.'
      : state.status === 'success'
        ? 'Preview rendering complete.'
        : state.status === 'failure'
          ? `Preview failed with ${state.diagnostics.length} diagnostic.`
          : ''}
  </div>
</main>

<style>
  :global(html) {
    background: var(--canvas);
  }

  .workspace-shell {
    display: grid;
    grid-template-rows: 72px minmax(0, 1fr);
    height: 100vh;
    min-height: 0;
    background: var(--canvas);
    color: var(--ink);
  }

  .workspace-shell.is-workspace {
    grid-template-rows: 72px minmax(0, 1fr);
  }

  .workspace-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 0 28px;
    border-bottom: 1px solid var(--rule);
    background: var(--surface);
  }

  .brand-lockup,
  .header-actions,
  .preview-actions,
  .source-actions {
    display: flex;
    align-items: center;
  }

  .brand-lockup {
    gap: 12px;
  }

  .brand-mark {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid var(--blue);
    color: var(--blue);
    font-size: 14px;
    font-weight: 700;
    letter-spacing: -0.05em;
  }

  :global(.brand-lockup h1),
  :global(.form-heading h2),
  :global(.form-heading p),
  :global(.preview-header h2),
  :global(.preview-header p),
  :global(.source-header h2),
  :global(.source-header p) {
    margin: 0;
  }

  :global(.brand-lockup h1) {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .section-kicker,
  .section-index,
  .page-count,
  :global(.field-label),
  :global(.form-label) {
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.08em;
    line-height: 1.3;
    text-transform: uppercase;
  }

  .header-actions {
    gap: 12px;
  }

  .account-controls {
    position: relative;
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .account-button {
    min-height: 32px;
    padding: 0 10px;
    font-size: 11px;
  }

  .account-label {
    max-width: 150px;
    overflow: hidden;
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .import-dialog-backdrop {
    position: fixed;
    z-index: 20;
    inset: 0;
    display: grid;
    place-items: center;
    padding: 16px;
    background: rgb(23 33 43 / 34%);
  }

  .import-dialog {
    width: min(520px, 100%);
    max-height: calc(100vh - 32px);
    overflow-y: auto;
    border: 1px solid var(--rule-strong);
    border-radius: 10px;
    padding: 24px;
    background: var(--surface);
    box-shadow: 0 18px 42px rgb(23 33 43 / 16%);
  }

  .import-dialog-heading,
  .import-dialog-actions {
    display: flex;
    align-items: center;
  }

  .import-dialog-heading {
    justify-content: space-between;
    gap: 16px;
  }

  .import-dialog h2,
  .import-dialog p {
    margin: 0;
  }

  .import-dialog h2 {
    font-size: 22px;
    letter-spacing: -0.03em;
  }

  .import-dialog-warning {
    margin-top: 18px !important;
    color: var(--ink);
    font-size: 15px;
    line-height: 1.5;
  }

  .import-review {
    display: grid;
    gap: 6px;
    margin-top: 18px;
    border-top: 1px solid var(--rule);
    border-bottom: 1px solid var(--rule);
    padding: 16px 0;
  }

  .import-review span,
  .import-review p {
    color: var(--muted-ink);
    font-size: 14px;
    line-height: 1.45;
  }

  .import-dialog-error {
    margin-top: 16px !important;
    color: var(--danger);
    font-size: 14px;
    line-height: 1.45;
  }

  .import-dialog-actions {
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 22px;
  }

  @media (max-width: 560px) {
    .import-dialog-backdrop {
      place-items: end center;
      padding: 8px;
    }

    .import-dialog {
      max-height: calc(100vh - 16px);
      padding: 20px 16px;
    }

    .import-dialog-actions {
      align-items: stretch;
      flex-direction: column-reverse;
    }

    .import-dialog-actions :global(.button),
    .import-dialog-actions :global(.text-button) {
      width: 100%;
    }
  }

  .account-panel {
    position: absolute;
    z-index: 10;
    top: calc(100% + 14px);
    right: 0;
    width: min(360px, calc(100vw - 32px));
    border: 1px solid var(--rule-strong);
    border-radius: 10px;
    padding: 20px;
    background: var(--surface);
    box-shadow: 0 18px 42px rgb(23 33 43 / 16%);
  }

  .account-panel-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  :global(.account-panel h2) {
    margin: 4px 0 0;
    font-size: 20px;
    letter-spacing: -0.03em;
  }

  .account-helper {
    margin: 10px 0 18px;
    color: var(--muted-ink);
    font-size: 12px;
    line-height: 1.45;
  }

  :global(.account-panel form) {
    display: grid;
    gap: 14px;
  }

  :global(.account-panel .field-label) {
    margin-bottom: 6px;
  }

  .account-submit {
    width: 100%;
    margin-top: 3px;
  }

  .account-error {
    margin: 0;
    color: var(--danger);
    font-size: 12px;
    line-height: 1.4;
  }

  .account-switch,
  .close-button {
    border: 0;
    background: transparent;
    color: var(--blue-dark);
    cursor: pointer;
  }

  .account-switch {
    margin-top: 15px;
    padding: 0;
    font-family: var(--mono);
    font-size: 11px;
    letter-spacing: 0.04em;
  }

  .account-switch:hover {
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  .close-button {
    width: 28px;
    height: 28px;
    color: var(--muted-ink);
    font-size: 22px;
    line-height: 1;
  }

  .button {
    min-height: 36px;
    border: 1px solid transparent;
    border-radius: 7px;
    padding: 0 14px;
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    line-height: 1;
    text-transform: uppercase;
    transition:
      background-color 180ms ease,
      border-color 180ms ease,
      color 180ms ease,
      transform 180ms ease;
  }

  .button:disabled,
  :global(.text-button:disabled),
  :global(.source-action:disabled) {
    cursor: not-allowed;
    opacity: 0.45;
  }

  .button-primary {
    background: var(--blue);
    color: #fff;
  }

  .button-primary:hover:not(:disabled) {
    background: var(--blue-dark);
    transform: translateY(-1px);
  }

  .button-secondary {
    border-color: var(--rule-strong);
    background: var(--surface);
    color: var(--ink);
  }

  .button-secondary:hover:not(:disabled) {
    border-color: var(--blue);
    color: var(--blue-dark);
  }

  .mobile-pane-nav {
    display: none;
    grid-template-columns: repeat(2, 1fr);
    border-bottom: 1px solid var(--rule);
    background: var(--surface-subtle);
  }

  :global(.mobile-pane-nav button) {
    position: relative;
    min-height: 44px;
    border: 0;
    background: transparent;
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }

  :global(.mobile-pane-nav button.active-tab) {
    color: var(--blue-dark);
  }

  :global(.mobile-pane-nav button.active-tab::after) {
    position: absolute;
    right: 24%;
    bottom: -1px;
    left: 24%;
    height: 2px;
    background: var(--blue);
    content: '';
  }

  .tab-count {
    display: inline-grid;
    min-width: 17px;
    height: 17px;
    margin-left: 5px;
    place-items: center;
    border-radius: 50%;
    background: var(--danger);
    color: #fff;
    font-size: 10px;
  }

  .workspace-content {
    display: grid;
    min-height: 0;
    grid-template-columns: minmax(520px, 56%) minmax(360px, 44%);
  }

  @media (min-width: 1200px) {
    .workspace-content {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  .is-intake .workspace-content {
    grid-template-columns: minmax(0, 1fr);
  }

  .form-panel {
    display: grid;
    min-height: 0;
    grid-template-columns: 176px minmax(0, 760px);
    column-gap: 28px;
    justify-content: center;
    border-right: 0;
    padding: 0 28px;
    background: var(--surface);
  }

  .mobile-preview-action {
    display: none;
  }

  .is-intake .form-panel {
    column-gap: 36px;
  }

  .is-intake .form-panel :global(.section-rail) {
    align-self: start;
    border-right: 0;
    background: transparent;
    padding: 44px 0;
  }

  .is-intake .form-panel :global(.builder-scroll) {
    padding-right: 0;
    padding-left: 0;
  }

  .intake-panel .builder-scroll {
    background: var(--surface);
  }

  .section-rail {
    padding: 27px 14px;
    border-right: 1px solid var(--rule);
    background: var(--surface-subtle);
  }

  .section-nav-button {
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

  .section-nav-button:hover {
    background: var(--surface);
    color: var(--ink);
  }

  .section-nav-button.is-active {
    border-left-color: var(--blue);
    background: var(--blue-soft);
    color: var(--blue-dark);
    font-weight: 650;
  }

  .section-nav-button.has-error .section-index {
    color: var(--danger);
  }

  .section-index {
    color: var(--quiet-ink);
    font-size: 10px;
  }

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

  .section-kicker {
    margin-bottom: 10px !important;
    color: var(--blue-dark);
    font-size: 11px;
  }

  :global(.form-heading h2) {
    font-size: 34px;
    font-weight: 700;
    letter-spacing: -0.04em;
    line-height: 1.08;
  }

  :global(.form-heading p:not(.section-kicker)) {
    max-width: 52ch;
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

  :global(.field-label),
  :global(.form-label) {
    display: block;
    margin-bottom: 7px;
    color: var(--muted-ink);
    font-size: 11px;
  }

  :global(.field-label b) {
    color: var(--blue);
  }

  :global(.cv-input) {
    width: 100%;
    min-height: 42px;
    border: 1px solid var(--rule-strong);
    border-radius: 6px;
    padding: 9px 11px;
    background: var(--surface);
    color: var(--ink);
    font-size: 15px;
    line-height: 1.4;
    outline: none;
    transition:
      border-color 180ms ease,
      box-shadow 180ms ease,
      background-color 180ms ease;
  }

  :global(.cv-input::placeholder) {
    color: #8b98a4;
  }

  :global(.cv-input:focus) {
    border-color: var(--blue);
    box-shadow: 0 0 0 3px rgb(23 105 210 / 14%);
  }

  :global(.cv-input:disabled) {
    background: var(--surface-subtle);
    color: var(--quiet-ink);
  }

  :global(.cv-textarea) {
    min-height: 96px;
    resize: vertical;
  }

  :global(.field-error) {
    display: block;
    margin-top: 6px;
    color: var(--danger);
    font-family: var(--mono);
    font-size: 11px;
    line-height: 1.4;
  }

  :global(.entry-card) {
    margin-bottom: 18px;
    border: 1px solid var(--rule);
    border-radius: 8px;
    padding: 18px;
    background: var(--surface-raised);
  }

  :global(.entry-head) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 18px;
    border-bottom: 1px solid var(--rule);
    padding-bottom: 12px;
  }

  :global(.entry-head h3) {
    margin: 0;
    font-size: 17px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  :global(.add-button) {
    width: 100%;
    margin-bottom: 18px;
    border: 1px dashed var(--rule-strong);
    border-radius: 7px;
    padding: 13px 16px;
    background: transparent;
    color: var(--blue-dark);
    cursor: pointer;
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
  }

  :global(.add-button:hover) {
    border-color: var(--blue);
    background: var(--blue-soft);
  }

  :global(.form-grid) {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 20px 22px;
  }

  :global(.text-button),
  :global(.source-action) {
    border: 0;
    background: transparent;
    padding: 7px 8px;
    color: var(--blue-dark);
    cursor: pointer;
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  :global(.text-button:hover),
  :global(.source-action:hover) {
    color: var(--blue);
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  :global(.remove-button) {
    align-self: center;
    border: 0;
    background: transparent;
    color: var(--danger);
    cursor: pointer;
    font-size: 22px;
  }

  :global(.source-action) {
    min-height: 34px;
    border: 1px solid var(--rule-strong);
    border-radius: 6px;
    color: var(--ink);
  }

  :global(.check) {
    align-self: end;
    padding: 10px 0;
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
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

  :global(.preview-header h2),
  :global(.source-header h2) {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .preview-actions,
  .source-actions {
    gap: 10px;
  }

  .source-panel {
    display: flex;
    min-height: 0;
    flex-direction: column;
    background: #18222d;
    color: #f1f5f8;
  }

  .source-panel .source-header {
    border-color: #354554;
    background: #202d39;
  }

  :global(.source-panel .source-header h2) {
    color: #fff;
  }

  .source-panel :global(.source-action) {
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

  :global(.field-label),
  :global(.form-label) {
    font-family: var(--mono);
  }

  @media (max-width: 620px) {
    .is-intake .form-panel :global(.form-grid) {
      grid-template-columns: 1fr;
    }

    .is-intake .form-panel :global(.form-grid > .col-span-2) {
      grid-column: auto;
    }
  }

  @media (max-width: 900px) {
    .workspace-shell.is-workspace {
      grid-template-rows: 68px 44px minmax(0, 1fr);
    }

    .mobile-pane-nav {
      display: grid;
    }

    .mobile-hidden {
      display: none !important;
    }

    .workspace-content {
      display: block;
    }

    .form-panel {
      display: grid;
      height: 100%;
      grid-template-columns: 1fr;
      grid-template-rows: minmax(0, 1fr) auto;
      border-right: 0;
      padding: 0;
    }

    .is-intake .form-panel {
      grid-template-columns: 1fr;
      padding: 0;
    }

    .is-intake .form-panel :global(.builder-scroll) {
      padding: 32px 24px 44px;
    }

    .mobile-preview-action {
      display: flex;
      border-top: 1px solid var(--rule);
      padding: 12px 16px 16px;
      background: var(--surface);
    }

    .mobile-preview-action :global(.button) {
      width: 100%;
    }

    .section-rail {
      display: none;
    }

    .preview-panel {
      height: 100%;
    }

    .workspace-header {
      padding: 0 16px;
    }

    .builder-scroll {
      padding: 32px 24px 44px;
    }

    .form-content {
      width: min(100%, 700px);
    }
  }

  @media (max-width: 560px) {
    .header-actions {
      gap: 7px;
    }

    .source-toggle {
      display: none;
    }

    .register-button,
    .account-label {
      display: none;
    }

    .account-panel {
      position: fixed;
      top: 68px;
      right: 16px;
    }

    .builder-scroll {
      padding: 28px 16px 36px;
    }

    .is-intake .form-panel :global(.builder-scroll) {
      padding: 28px 16px 36px;
    }

    :global(.form-heading h2) {
      font-size: 30px;
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
  }
</style>

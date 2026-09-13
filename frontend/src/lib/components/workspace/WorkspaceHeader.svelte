<script lang="ts">
  import type { AuthUser } from '$lib/api';
  import { Button, ButtonLink, TextField } from '../base';

  export let presentation: 'intake' | 'workspace';
  export let authUser: AuthUser | null;
  export let authOpen: boolean;
  export let authMode: 'login' | 'register';
  export let authEmail: string;
  export let authPassword: string;
  export let authName: string;
  export let authBusy: boolean;
  export let authNotice: string;
  export let googleStartUrl: string;
  export let linkedInStartUrl: string;
  export let linkedinEnabled = true;
  export let advanced: boolean;
  export let onAuthMode: (mode: 'login' | 'register') => void;
  export let onAuthOpenChange: (open: boolean) => void;
  export let onEmailChange: (value: string) => void;
  export let onPasswordChange: (value: string) => void;
  export let onNameChange: (value: string) => void;
  export let onSubmitAuth: () => void;
  export let onGoogleAuth: (event: MouseEvent) => void | Promise<void>;
  export let onLinkedInAuth: (event: MouseEvent) => void | Promise<void>;
  export let onLogout: () => void;
  export let onToggleAdvanced: () => void;
</script>

<header class="workspace-header" class:is-intake={presentation === 'intake'}>
  <div class="workspace-header-inner">
    <div class="brand-lockup">
      <div class="brand-mark" aria-hidden="true">CV</div>
      <h1>CV Workbench</h1>
    </div>
    <div class="header-actions">
      <div class="account-controls">
        {#if authUser}
          <span class="account-label" title={authUser.email}>{authUser.email}</span>
          <Button variant="secondary" className="account-button" onClick={onLogout}>Log out</Button>
        {:else}
          <Button
            variant="secondary"
            className="account-button"
            onClick={() => onAuthMode('login')}>
            Log in
          </Button>
          <Button
            variant="secondary"
            className="account-button register-button"
            onClick={() => onAuthMode('register')}>
            Register
          </Button>
        {/if}
        {#if authOpen}
          <div
            class="account-panel"
            role="dialog"
            aria-label={authMode === 'login' ? 'Log in' : 'Create account'}>
            <div class="account-panel-heading">
              <div>
                <h2>{authMode === 'login' ? 'Welcome back' : 'Save your CV everywhere'}</h2>
              </div>
              <Button
                variant="text"
                className="close-button"
                aria-label="Close account form"
                onClick={() => onAuthOpenChange(false)}>
                ×
              </Button>
            </div>
            <p class="account-helper">Anonymous editing stays available without signing in.</p>
            <form on:submit|preventDefault={onSubmitAuth}>
              {#if authMode === 'register'}
                <TextField
                  label="Name"
                  value={authName}
                  autocomplete="name"
                  on:input={(event) => onNameChange((event.target as HTMLInputElement).value)} />
              {/if}
              <TextField
                label="Email"
                value={authEmail}
                type="email"
                autocomplete="email"
                required
                on:input={(event) => onEmailChange((event.target as HTMLInputElement).value)} />
              <TextField
                label="Password"
                value={authPassword}
                type="password"
                autocomplete={authMode === 'login' ? 'current-password' : 'new-password'}
                minlength="12"
                required
                on:input={(event) => onPasswordChange((event.target as HTMLInputElement).value)} />
              {#if authNotice}<p class="account-error" role="alert">{authNotice}</p>{/if}
              <Button
                variant="primary"
                className="account-submit"
                type="submit"
                disabled={authBusy}>
                {authBusy ? 'Working…' : authMode === 'login' ? 'Log in' : 'Create account'}
              </Button>
            </form>
            <ButtonLink
              href={googleStartUrl}
              variant="secondary"
              className="google-auth"
              disabled={authBusy}
              onClick={onGoogleAuth}>
              Continue with Google
            </ButtonLink>
            {#if linkedinEnabled}
              <ButtonLink
                href={linkedInStartUrl}
                variant="secondary"
                className="linkedin-auth"
                disabled={authBusy}
                onClick={onLinkedInAuth}>
                Continue with LinkedIn
              </ButtonLink>
            {/if}
            <Button
              variant="text"
              className="account-switch"
              onClick={() => onAuthMode(authMode === 'login' ? 'register' : 'login')}>
              {authMode === 'login'
                ? 'Need an account? Register'
                : 'Already have an account? Log in'}
            </Button>
          </div>
        {/if}
      </div>
      {#if presentation === 'workspace'}
        <Button
          variant="secondary"
          className="source-toggle"
          onClick={onToggleAdvanced}
          aria-pressed={advanced}>
          Source
        </Button>
      {/if}
    </div>
  </div>
</header>

<style>
  .workspace-header {
    border-bottom: 1px solid var(--rule);
    background: var(--surface);
  }

  .workspace-header-inner {
    display: flex;
    height: 72px;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    width: 100%;
    margin: 0 auto;
    padding: 0 28px;
  }

  .workspace-header.is-intake .workspace-header-inner {
    width: min(calc(100% - 56px), 1028px);
    padding-right: 28px;
    padding-left: 28px;
  }

  .brand-lockup,
  .header-actions,
  .account-controls {
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

  .brand-lockup h1 {
    margin: 0;
  }

  .brand-lockup h1 {
    font-size: 18px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  :global(.field-label) {
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
    gap: 7px;
  }

  :global(.account-button) {
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
    display: block;
    margin-bottom: 6px;
  }

  :global(.account-submit) {
    width: 100%;
    margin-top: 3px;
  }

  :global(.google-auth) {
    width: 100%;
    margin-top: 12px;
    text-transform: none;
  }

  :global(.linkedin-auth) {
    width: 100%;
    margin-top: 8px;
    text-transform: none;
  }

  .account-error {
    margin: 0;
    color: var(--danger);
    font-size: 12px;
    line-height: 1.4;
  }

  :global(.account-switch),
  :global(.close-button) {
    border: 0;
    background: transparent;
    color: var(--blue-dark);
    cursor: pointer;
  }

  :global(.account-switch) {
    margin-top: 15px;
    padding: 0;
    font-family: var(--mono);
    font-size: 11px;
    letter-spacing: 0.04em;
  }

  :global(.account-switch:hover) {
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  :global(.close-button) {
    width: 28px;
    height: 28px;
    color: var(--muted-ink);
    font-size: 22px;
    line-height: 1;
  }

  @media (max-width: 900px) {
    .workspace-header-inner {
      height: 100%;
      padding: 0 16px;
    }

    .workspace-header.is-intake .workspace-header-inner {
      width: 100%;
      padding-right: 16px;
      padding-left: 16px;
    }
  }

  @media (max-width: 560px) {
    .header-actions {
      gap: 7px;
    }

    :global(.source-toggle),
    :global(.register-button),
    .account-label {
      display: none;
    }

    .account-panel {
      position: fixed;
      top: 68px;
      right: 16px;
    }
  }
</style>

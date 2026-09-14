<script lang="ts">
  import type { LandingCopy, Locale } from '$lib/i18n';

  export let copy: LandingCopy['header'];
  export let locale: Locale;
</script>

<header class="site-header">
  <a class="brand" href={locale === 'id' ? '/id' : '/'} aria-label={copy.homeLabel}>
    <span class="brand-mark">CV</span>
    <span class="brand-name">CV Workbench</span>
  </a>
  <nav class="site-nav" aria-label={copy.navigationLabel}>
    <a href="#how-it-works">{copy.howItWorks}</a>
    <a href="#templates">{copy.templates}</a>
    <div class="language-switcher" aria-label={copy.languageLabel}>
      <a href="/" aria-current={locale === 'en' ? 'page' : undefined}>EN</a>
      <span aria-hidden="true">/</span>
      <a href="/id" aria-current={locale === 'id' ? 'page' : undefined}>ID</a>
    </div>
    <a class="button button-primary" href="/app">
      {copy.openBuilder}
      <span aria-hidden="true">↗</span>
    </a>
  </nav>
</header>

<style>
  .site-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    min-height: 88px;
    border-bottom: 1px solid var(--rule);
  }

  .site-nav {
    gap: 28px;
  }

  .site-nav > a:not(.button) {
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-decoration: none;
    text-transform: uppercase;
  }

  .site-nav > a:not(.button):hover {
    color: var(--blue-dark);
  }

  .language-switcher {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--quiet-ink);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
  }

  .language-switcher a {
    color: inherit;
    text-decoration: none;
  }

  .language-switcher a[aria-current='page'] {
    color: var(--blue-dark);
  }

  @media (max-width: 700px) {
    .site-header {
      min-height: 72px;
    }

    .site-nav {
      gap: 13px;
    }

    .site-nav > a:not(.button) {
      display: none;
    }

    .language-switcher {
      font-size: 10px;
    }

    .site-nav .button {
      min-height: 34px;
      padding: 0 11px;
      font-size: 10px;
    }
  }
</style>

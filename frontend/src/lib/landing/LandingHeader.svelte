<header class="site-header">
  <a class="brand" href="/" aria-label="CV Workbench home">
    <span class="brand-mark">CV</span>
    <span class="brand-name">CV Workbench</span>
  </a>
  <nav class="site-nav" aria-label="Main navigation">
    <a href="#how-it-works">How it works</a>
    <a href="#templates">Templates</a>
    <a class="button button-primary" href="/app">
      Open builder <span aria-hidden="true">↗</span>
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

  .site-nav a:not(.button) {
    color: var(--muted-ink);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-decoration: none;
    text-transform: uppercase;
  }

  .site-nav a:not(.button):hover {
    color: var(--blue-dark);
  }

  @media (max-width: 700px) {
    .site-header {
      min-height: 72px;
    }

    .site-nav {
      gap: 13px;
    }

    .site-nav a:not(.button) {
      display: none;
    }

    .site-nav .button {
      min-height: 34px;
      padding: 0 11px;
      font-size: 10px;
    }
  }
</style>

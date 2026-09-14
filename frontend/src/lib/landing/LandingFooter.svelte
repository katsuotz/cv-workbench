<script lang="ts">
  import type { LandingCopy } from '$lib/i18n';

  export let copy: LandingCopy['footer'];
</script>

<footer class="site-footer section-frame">
  <a class="brand footer-brand" href="/" aria-label={copy.homeLabel}>
    <span class="brand-mark">CV</span>
    <span class="brand-name">CV Workbench</span>
  </a>
  <p class="footer-credit">
    {copy.builtBy}
    <a href="https://katsuotz.com">katsuotz.com</a>
  </p>
</footer>

<style>
  .site-footer {
    justify-content: space-between;
    min-height: 112px;
    gap: 24px;
    padding-top: 25px;
    padding-bottom: 25px;
  }

  .footer-brand {
    flex-shrink: 0;
  }

  .footer-credit {
    margin: 0;
    color: var(--quiet-ink);
    font-size: 13px;
    line-height: 1.4;
  }

  .footer-credit a {
    color: var(--ink);
    font-weight: 600;
    text-underline-offset: 3px;
    transition: color 180ms ease;
  }

  .footer-credit a:hover {
    color: var(--blue);
  }

  @media (max-width: 700px) {
    .site-footer {
      align-items: flex-start;
      flex-direction: column;
      justify-content: center;
      min-height: 132px;
    }
  }
</style>

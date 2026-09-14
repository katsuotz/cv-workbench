<script lang="ts">
  import { page } from '$app/stores';
  import PageSeo from '$lib/seo/PageSeo.svelte';
  import { absoluteUrl, LANDING_SEO, resolveSiteOrigin } from '$lib/seo';
  import LandingCta from './LandingCta.svelte';
  import LandingFooter from './LandingFooter.svelte';
  import LandingHeader from './LandingHeader.svelte';
  import LandingTemplates from './LandingTemplates.svelte';
  import LandingWorkflow from './LandingWorkflow.svelte';
  import HeroProofDesk from './HeroProofDesk.svelte';

  const pageTitle = LANDING_SEO.title;
  const pageDescription = LANDING_SEO.description;
  const socialImagePath = '/templates/editorial-v1.webp';

  const templates = [
    {
      id: 'editorial-v1',
      name: 'Editorial dossier',
      description: 'A quiet, structured page for thoughtful work.',
      image: '/templates/editorial-v1.webp'
    },
    {
      id: 'compact-v1',
      name: 'Compact signal',
      description: 'A denser layout for broad experience.',
      image: '/templates/compact-v1.webp'
    },
    {
      id: 'modern-v1',
      name: 'Modern hierarchy',
      description: 'A contemporary layout with a stronger accent.',
      image: '/templates/modern-v1.webp'
    }
  ];

  $: siteOrigin = resolveSiteOrigin($page.url.origin);
  $: canonicalUrl = absoluteUrl(siteOrigin, '/');
  $: socialImageUrl = absoluteUrl(siteOrigin, socialImagePath);
  $: structuredData = {
    '@context': 'https://schema.org',
    '@graph': [
      {
        '@type': 'Organization',
        '@id': `${siteOrigin}/#organization`,
        name: 'CV Workbench',
        url: canonicalUrl,
        logo: {
          '@type': 'ImageObject',
          url: `${siteOrigin}/favicon.svg`
        }
      },
      {
        '@type': 'WebSite',
        '@id': `${siteOrigin}/#website`,
        name: 'CV Workbench',
        url: canonicalUrl,
        description: pageDescription,
        inLanguage: 'en-US',
        publisher: { '@id': `${siteOrigin}/#organization` }
      },
      {
        '@type': 'WebApplication',
        '@id': `${siteOrigin}/#application`,
        name: 'CV Workbench',
        url: canonicalUrl,
        description: pageDescription,
        applicationCategory: 'BusinessApplication',
        applicationSubCategory: 'ATS-friendly CV builder',
        operatingSystem: 'Web browser',
        browserRequirements: 'Requires JavaScript',
        isAccessibleForFree: true,
        image: socialImageUrl,
        inLanguage: 'en-US',
        featureList: ['ATS-friendly CV structure', 'Exact LaTeX source', 'Rendered PDF'],
        creator: { '@id': `${siteOrigin}/#organization` },
        potentialAction: {
          '@type': 'CreateAction',
          target: {
            '@type': 'EntryPoint',
            urlTemplate: `${siteOrigin}/app`
          }
        }
      }
    ]
  };
</script>

<PageSeo
  title={pageTitle}
  description={pageDescription}
  indexable
  canonicalPath="/"
  {socialImagePath}
  {structuredData} />

<svelte:head>
  {#each templates as template}
    <link rel="preload" as="image" href={template.image} />
  {/each}
</svelte:head>

<main class="landing">
  <div class="proof-thread" aria-hidden="true"></div>
  <LandingHeader />
  <HeroProofDesk />
  <LandingTemplates {templates} />
  <LandingWorkflow />
  <LandingCta />
  <LandingFooter />
</main>

<style>
  :global(html:has(.landing)),
  :global(body:has(.landing)) {
    background: var(--canvas);
  }

  :global(body:has(.landing)) {
    overflow: auto;
  }

  .landing {
    position: relative;
    overflow: hidden;
    color: var(--ink);
  }

  .proof-thread {
    position: absolute;
    z-index: 0;
    top: 415px;
    bottom: 175px;
    left: max(22px, calc((100vw - 1180px) / 2));
    width: 1px;
    background: linear-gradient(
      to bottom,
      transparent,
      var(--rule-strong) 9%,
      var(--rule-strong) 91%,
      transparent
    );
    opacity: 0.75;
  }

  :global(.landing .section-frame),
  :global(.landing .site-header) {
    position: relative;
    z-index: 1;
    width: min(1180px, calc(100% - 64px));
    margin: 0 auto;
  }

  :global(.landing .brand),
  :global(.landing .site-nav),
  :global(.landing .hero-actions),
  :global(.landing .template-card-copy),
  :global(.landing .stage-heading),
  :global(.landing .source-meta),
  :global(.landing .site-footer) {
    display: flex;
    align-items: center;
  }

  :global(.landing .brand) {
    gap: 10px;
    color: var(--ink);
    text-decoration: none;
  }

  :global(.landing .brand-mark) {
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

  :global(.landing .brand-name) {
    font-size: 17px;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  :global(.landing .button) {
    display: inline-flex;
    min-height: 38px;
    align-items: center;
    justify-content: center;
    gap: 12px;
    border: 1px solid transparent;
    border-radius: 7px;
    padding: 0 16px;
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    line-height: 1;
    text-decoration: none;
    text-transform: uppercase;
    transition:
      background-color 180ms ease,
      border-color 180ms ease,
      color 180ms ease,
      transform 180ms ease;
  }

  :global(.landing .button:hover) {
    transform: translateY(-1px);
  }

  :global(.landing .button-primary) {
    background: var(--blue);
    color: #fff;
  }

  :global(.landing .button-primary:hover) {
    background: var(--blue-dark);
  }

  :global(.landing .button-light) {
    background: var(--surface);
    color: var(--blue-dark);
  }

  :global(.landing .button-light:hover) {
    background: var(--blue-soft);
  }

  :global(.landing .button-large) {
    min-height: 48px;
    padding: 0 21px;
  }

  :global(.landing .stage-index),
  :global(.landing .workflow-marker),
  :global(.landing .source-meta) {
    color: var(--quiet-ink);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.1em;
    line-height: 1.3;
    text-transform: uppercase;
  }

  :global(.landing h1),
  :global(.landing h2),
  :global(.landing h3),
  :global(.landing p) {
    margin-top: 0;
  }

  :global(.landing h1),
  :global(.landing h2) {
    margin-bottom: 0;
    letter-spacing: -0.055em;
  }

  :global(.landing h1 span),
  :global(.landing h2 span) {
    color: var(--blue);
  }

  :global(.landing a:focus-visible) {
    outline: 2px solid var(--blue);
    outline-offset: 4px;
  }

  @media (max-width: 700px) {
    :global(.landing .section-frame),
    :global(.landing .site-header) {
      width: min(100% - 32px, 560px);
    }

    .proof-thread {
      top: 335px;
      bottom: 214px;
      left: 16px;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    :global(.landing .button),
    :global(.landing .template-image-wrap) {
      transition: none;
    }
  }
</style>

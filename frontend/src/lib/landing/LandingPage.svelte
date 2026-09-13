<script lang="ts">
  import { page } from '$app/stores';
  import LandingCta from './LandingCta.svelte';
  import LandingFooter from './LandingFooter.svelte';
  import LandingHeader from './LandingHeader.svelte';
  import LandingTemplates from './LandingTemplates.svelte';
  import LandingWorkflow from './LandingWorkflow.svelte';
  import HeroProofDesk from './HeroProofDesk.svelte';

  const pageTitle = 'CV Workbench — ATS-Friendly CV Builder';
  const pageDescription =
    'Build an ATS-friendly CV with structured sections, clear formatting, exact LaTeX source, and a polished PDF for job applications.';
  const defaultSiteOrigin = import.meta.env.DEV ? '' : 'https://cvworkbench.com';
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

  $: siteOrigin = (
    import.meta.env.PUBLIC_SITE_URL ||
    defaultSiteOrigin ||
    $page.url.origin
  ).replace(/\/$/, '');
  $: canonicalUrl = `${siteOrigin}/`;
  $: socialImageUrl = `${siteOrigin}${socialImagePath}`;
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
  $: structuredDataMarkup = `<script type="application/ld+json">${JSON.stringify(structuredData)}\u003c/script>`;
</script>

<svelte:head>
  <title>{pageTitle}</title>
  <meta name="description" content={pageDescription} />
  <meta name="author" content="CV Workbench" />
  <meta
    name="robots"
    content="index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1" />
  <meta
    name="googlebot"
    content="index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1" />
  <meta name="application-name" content="CV Workbench" />
  <link rel="canonical" href={canonicalUrl} />
  <link rel="alternate" hreflang="en" href={canonicalUrl} />
  <link rel="alternate" hreflang="x-default" href={canonicalUrl} />
  <meta property="og:type" content="website" />
  <meta property="og:site_name" content="CV Workbench" />
  <meta property="og:locale" content="en_US" />
  <meta property="og:title" content={pageTitle} />
  <meta property="og:description" content={pageDescription} />
  <meta property="og:url" content={canonicalUrl} />
  <meta property="og:image" content={socialImageUrl} />
  <meta property="og:image:secure_url" content={socialImageUrl} />
  <meta property="og:image:type" content="image/webp" />
  <meta property="og:image:width" content="1020" />
  <meta property="og:image:height" content="1320" />
  <meta property="og:image:alt" content="ATS-friendly CV template preview from CV Workbench" />
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={pageTitle} />
  <meta name="twitter:description" content={pageDescription} />
  <meta name="twitter:image" content={socialImageUrl} />
  <meta name="twitter:image:alt" content="ATS-friendly CV template preview from CV Workbench" />
  {@html structuredDataMarkup}
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

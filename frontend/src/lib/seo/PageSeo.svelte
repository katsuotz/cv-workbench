<script lang="ts">
  import { page } from '$app/stores';
  import { absoluteUrl, INDEX_ROBOTS, resolveSiteOrigin, SITE_NAME } from '$lib/seo';

  export let title: string;
  export let description: string;
  export let robots = INDEX_ROBOTS;
  export let indexable = false;
  export let canonicalPath: string | undefined = undefined;
  export let socialImagePath = '/templates/editorial-v1.webp';
  export let socialImageAlt = 'ATS-friendly CV template preview from CV Workbench';
  export let structuredData: Record<string, unknown> | undefined = undefined;

  $: siteOrigin = resolveSiteOrigin($page.url.origin);
  $: canonicalUrl = canonicalPath ? absoluteUrl(siteOrigin, canonicalPath) : '';
  $: socialImageUrl = absoluteUrl(siteOrigin, socialImagePath);
  $: structuredDataMarkup = structuredData
    ? `<script type="application/ld+json">${JSON.stringify(structuredData)}\u003c/script>`
    : '';
</script>

<svelte:head>
  <title>{title}</title>
  <meta name="description" content={description} />
  <meta name="application-name" content={SITE_NAME} />
  <meta name="robots" content={robots} />
  <meta name="googlebot" content={robots} />
  {#if indexable && canonicalPath}
    <link rel="canonical" href={canonicalUrl} />
    <link rel="alternate" hreflang="en" href={canonicalUrl} />
    <link rel="alternate" hreflang="x-default" href={canonicalUrl} />
    <meta property="og:type" content="website" />
    <meta property="og:site_name" content={SITE_NAME} />
    <meta property="og:locale" content="en_US" />
    <meta property="og:title" content={title} />
    <meta property="og:description" content={description} />
    <meta property="og:url" content={canonicalUrl} />
    <meta property="og:image" content={socialImageUrl} />
    <meta property="og:image:secure_url" content={socialImageUrl} />
    <meta property="og:image:type" content="image/webp" />
    <meta property="og:image:width" content="1020" />
    <meta property="og:image:height" content="1320" />
    <meta property="og:image:alt" content={socialImageAlt} />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content={title} />
    <meta name="twitter:description" content={description} />
    <meta name="twitter:image" content={socialImageUrl} />
    <meta name="twitter:image:alt" content={socialImageAlt} />
  {/if}
  {#if structuredDataMarkup}{@html structuredDataMarkup}{/if}
</svelte:head>

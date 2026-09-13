<script lang="ts">
  import { onMount } from 'svelte';

  import { PUBLIC_GOOGLE_TAG_ID } from '$env/static/public';

  import '../app.css';

  onMount(() => {
    if (import.meta.env.DEV || !PUBLIC_GOOGLE_TAG_ID) return;

    const script = document.createElement('script');
    script.async = true;
    script.src = `https://www.googletagmanager.com/gtag/js?id=${encodeURIComponent(PUBLIC_GOOGLE_TAG_ID)}`;
    document.head.appendChild(script);

    const dataLayer = ((window as typeof window & { dataLayer?: unknown[] }).dataLayer ??= []);
    const gtag = (...args: unknown[]) => dataLayer.push(args);
    gtag('js', new Date());
    gtag('config', PUBLIC_GOOGLE_TAG_ID);
  });
</script>

<svelte:head>
  <meta name="theme-color" content="#eef1f4" />
</svelte:head>

<slot />

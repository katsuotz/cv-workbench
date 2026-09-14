<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { getLocaleFromPathname } from '$lib/i18n';

  $: htmlLang = getLocaleFromPathname($page.url.pathname);
  $: if (typeof document !== 'undefined') document.documentElement.lang = htmlLang;
</script>

<svelte:head>
  <meta name="theme-color" content="#eef1f4" />
</svelte:head>

<slot />

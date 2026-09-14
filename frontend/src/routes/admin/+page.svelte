<script lang="ts">
  import AdminPanel from '$lib/components/admin/AdminPanel.svelte';
  import PageSeo from '$lib/seo/PageSeo.svelte';
  import { ADMIN_SEO, NOINDEX_ROBOTS } from '$lib/seo';
</script>

<PageSeo title={ADMIN_SEO.title} description={ADMIN_SEO.description} robots={NOINDEX_ROBOTS} />

<AdminPanel />

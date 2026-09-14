<script lang="ts">
  import PageSeo from '$lib/seo/PageSeo.svelte';
  import { BUILDER_SEO, NOINDEX_ROBOTS } from '$lib/seo';
  import Workspace from '$lib/workspace/Workspace.svelte';
</script>

<PageSeo title={BUILDER_SEO.title} description={BUILDER_SEO.description} robots={NOINDEX_ROBOTS} />

<Workspace />

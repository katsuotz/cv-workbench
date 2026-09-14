export const SITE_NAME = 'CV Workbench';

export const BUILDER_SEO = {
  title: 'CV Workbench — CV Builder',
  description:
    'Build and refine your CV, generate exact LaTeX source, and review the rendered PDF in CV Workbench.'
};

export const ADMIN_SEO = {
  title: 'CV Workbench — Admin',
  description: 'Manage CV Workbench user accounts.'
};

export const NOINDEX_ROBOTS = 'noindex, nofollow, noarchive';
export const INDEX_ROBOTS =
  'index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1';

export function resolveSiteOrigin(pageOrigin: string) {
  return (
    import.meta.env.PUBLIC_SITE_URL ||
    (import.meta.env.DEV ? '' : 'https://cvworkbench.com') ||
    pageOrigin
  ).replace(/\/$/, '');
}

export function absoluteUrl(siteOrigin: string, path: string) {
  if (path === '/') return `${siteOrigin}/`;
  return `${siteOrigin}${path.startsWith('/') ? path : `/${path}`}`;
}

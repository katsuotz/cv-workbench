import type { Handle } from '@sveltejs/kit';
import { getLocaleFromPathname } from '$lib/i18n';

export const handle: Handle = async ({ event, resolve }) =>
  resolve(event, {
    transformPageChunk: ({ html }) =>
      html.replace('<html lang="en">', `<html lang="${getLocaleFromPathname(event.url.pathname)}">`)
  });

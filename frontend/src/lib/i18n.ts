export type Locale = 'en' | 'id';

export type LandingTemplate = {
  id: 'editorial-v1' | 'compact-v1' | 'modern-v1';
  name: string;
  description: string;
  image: string;
  imageAlt: string;
};

export type LandingCopy = {
  locale: Locale;
  htmlLang: 'en' | 'id';
  seo: {
    title: string;
    description: string;
    ogLocale: 'en_US' | 'id_ID';
    inLanguage: 'en-US' | 'id-ID';
    socialImageAlt: string;
    applicationSubCategory: string;
    featureList: string[];
  };
  header: {
    homeLabel: string;
    navigationLabel: string;
    languageLabel: string;
    howItWorks: string;
    templates: string;
    openBuilder: string;
  };
  hero: {
    titleLead: string;
    titleAccent: string;
    titleTail: string;
    titleResult: string;
    lede: string;
    startBuilding: string;
    seeTemplates: string;
    noAccount: string;
    proofLabel: string;
    stages: {
      facts: string;
      source: string;
      pdf: string;
      factsRows: Array<{ key: string; value: string }>;
      sourceSummary: string;
      sourceSummaryLine1: string;
      sourceSummaryLine2: string;
      sourceExperience: string;
      sourceExperienceValue: string;
      compiler: string;
      sourceFile: string;
    };
  };
  templates: {
    title: string;
    description: string;
    items: LandingTemplate[];
  };
  workflow: {
    title: string;
    steps: Array<{ title: string; description: string }>;
  };
  cta: {
    title: string;
    startBuilding: string;
  };
  footer: {
    homeLabel: string;
    builtBy: string;
  };
};

import en from './locales/en.json';
import id from './locales/id.json';

export const LANDING_COPY: Record<Locale, LandingCopy> = {
  en: en as LandingCopy,
  id: id as LandingCopy
};

export function getLandingCopy(locale: Locale): LandingCopy {
  return LANDING_COPY[locale];
}

export function getLocaleFromPathname(pathname: string): Locale {
  return /^\/id(?:\/|$)/.test(pathname) ? 'id' : 'en';
}

import fs from 'fs';
import { GetTextTranslations, po } from 'gettext-parser';
import Gettext from 'node-gettext';
import path from 'path';
import log from '../shared/logging';

const SOURCE_LANGUAGE = 'en';
const LOCALES_DIR = path.resolve(__dirname, '../../locales');

export function loadTranslations(
  currentLocale: string,
  catalogue: Gettext,
): GetTextTranslations | undefined {
  // First look for exact match of the current locale
  const preferredLocales = [];

  if (currentLocale !== SOURCE_LANGUAGE) {
    preferredLocales.push(currentLocale);
  }

  // In case of region bound locale like en-US, fallback to en.
  const language = Gettext.getLanguageCode(currentLocale);
  if (currentLocale !== language) {
    preferredLocales.push(language);
  }

  const domain = catalogue.domain;
  for (const locale of preferredLocales) {
    const parsedTranslations = parseTranslation(locale, domain, catalogue);
    if (parsedTranslations) {
      log.info(`Loaded translations ${locale}/${domain}`);
      catalogue.setLocale(locale);
      return parsedTranslations;
    }
  }

  // Reset the locale to source language if we couldn't load the catalogue for the requested locale
  // Add empty translations to suppress some of the warnings produces by node-gettext
  catalogue.addTranslations(SOURCE_LANGUAGE, domain, {});
  catalogue.setLocale(SOURCE_LANGUAGE);
  return;
}

function parseTranslation(
  locale: string,
  domain: string,
  catalogue: Gettext,
): GetTextTranslations | undefined {
  const filename = path.join(LOCALES_DIR, locale, `${domain}.po`);
  let contents: string;

  try {
    contents = fs.readFileSync(filename, { encoding: 'utf8' });
  } catch (e) {
    const error = e as NodeJS.ErrnoException;
    if (error.code !== 'ENOENT') {
      log.error(`Cannot read the gettext file "${filename}": ${error.message}`);
    }
    return undefined;
  }

  let translations: GetTextTranslations;
  try {
    translations = po.parse(contents);
  } catch (e) {
    const error = e as Error;
    log.error(`Cannot parse the gettext file "${filename}": ${error.message}`);
    return undefined;
  }

  catalogue.addTranslations(locale, domain, translations);

  return translations;
}

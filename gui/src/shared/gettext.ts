import log from 'electron-log';
import fs from 'fs';
import { po } from 'gettext-parser';
import Gettext from 'node-gettext';
import path from 'path';

const SOURCE_LANGUAGE = 'en';
let SELECTED_LANGUAGE = SOURCE_LANGUAGE;
const LOCALES_DIR = path.resolve(__dirname, '../../locales');

// `{debug: false}` option prevents Gettext from printing the warnings to console in development
// the errors are handled separately in the "error" handler below
const catalogue = new Gettext({ debug: false });
catalogue.setTextDomain('messages');
catalogue.on('error', (error: string) => {
  // Filter out the "no translation was found" errors for the source language
  if (SELECTED_LANGUAGE === SOURCE_LANGUAGE && error.indexOf('No translation was found') !== -1) {
    return;
  }

  log.warn(`Gettext error: ${error}`);
});

export function loadTranslations(currentLocale: string) {
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

  for (const locale of preferredLocales) {
    if (parseTranslation(locale, 'messages')) {
      log.info(`Loaded translations for ${locale}`);
      catalogue.setLocale(locale);

      SELECTED_LANGUAGE = locale;
      return;
    }
  }
}

function parseTranslation(locale: string, domain: string): boolean {
  const filename = path.join(LOCALES_DIR, locale, `${domain}.po`);
  let buffer: Buffer;

  try {
    buffer = fs.readFileSync(filename);
  } catch (error) {
    if (error.code !== 'ENOENT') {
      log.error(`Cannot read the gettext file "${filename}": ${error.message}`);
    }
    return false;
  }

  let translations: object;
  try {
    translations = po.parse(buffer);
  } catch (error) {
    log.error(`Cannot parse the gettext file "${filename}": ${error.message}`);
    return false;
  }

  catalogue.addTranslations(locale, domain, translations);

  return true;
}

export const gettext = (msgid: string): string => {
  return catalogue.gettext(msgid);
};
export const pgettext = (msgctx: string, msgid: string): string => {
  return catalogue.pgettext(msgctx, msgid);
};

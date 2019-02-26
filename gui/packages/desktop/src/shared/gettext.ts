import log from 'electron-log';
import fs from 'fs';
import { po } from 'gettext-parser';
import Gettext from 'node-gettext';
import path from 'path';

const SOURCE_LANGUAGE = 'en';
const LOCALES_DIR = path.resolve(__dirname, '../../locales');

const catalogue = new Gettext();
catalogue.setTextDomain('messages');

export function loadTranslations(currentLocale: string) {
  // First look for exact match of the current locale
  const preferredLocales = [];

  if (currentLocale !== SOURCE_LANGUAGE) {
    preferredLocales.push(currentLocale);
  }

  // In case of region bound locale like en-US, fallback to en.
  if (currentLocale.indexOf('-') !== -1) {
    const [language] = currentLocale.split('-');

    if (language.length > 0 && language !== SOURCE_LANGUAGE) {
      preferredLocales.push(language);
    }
  }

  for (const locale of preferredLocales) {
    if (parseTranslation(locale, 'messages')) {
      log.info(`Loaded translations for ${locale}`);
      catalogue.setLocale(locale);
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
    log.error(`Cannot read the gettext file "${filename}": ${error.message}`);
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

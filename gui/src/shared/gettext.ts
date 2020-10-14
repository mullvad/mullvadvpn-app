import log from 'electron-log';
import fs from 'fs';
import { po } from 'gettext-parser';
import Gettext from 'node-gettext';
import path from 'path';
import { LocalizationContexts } from './localization-contexts';

const SOURCE_LANGUAGE = 'en';
const LOCALES_DIR = path.resolve(__dirname, '../../locales');

export function loadTranslations(currentLocale: string, catalogue: Gettext) {
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
    if (parseTranslation(locale, domain, catalogue)) {
      log.info(`Loaded translations ${locale}/${domain}`);
      catalogue.setLocale(locale);
      return;
    }
  }

  // Reset the locale to source language if we couldn't load the catalogue for the requested locale
  // Add empty translations to suppress some of the warnings produces by node-gettext
  catalogue.addTranslations(SOURCE_LANGUAGE, domain, {});
  catalogue.setLocale(SOURCE_LANGUAGE);
}

function parseTranslation(locale: string, domain: string, catalogue: Gettext): boolean {
  const filename = path.join(LOCALES_DIR, locale, `${domain}.po`);
  let contents: string;

  try {
    contents = fs.readFileSync(filename, { encoding: 'utf8' });
  } catch (error) {
    if (error.code !== 'ENOENT') {
      log.error(`Cannot read the gettext file "${filename}": ${error.message}`);
    }
    return false;
  }

  let translations: ReturnType<typeof po.parse>;
  try {
    translations = po.parse(contents);
  } catch (error) {
    log.error(`Cannot parse the gettext file "${filename}": ${error.message}`);
    return false;
  }

  catalogue.addTranslations(locale, domain, translations);

  return true;
}

function setErrorHandler(catalogue: Gettext) {
  catalogue.on('error', (error) => {
    log.warn(`Gettext error: ${error}`);
  });
}

const gettextOptions = { sourceLocale: SOURCE_LANGUAGE };

declare class GettextWithAppContexts extends Gettext {
  pgettext(msgctxt: LocalizationContexts, msgid: string): string;
  npgettext(
    msgctxt: LocalizationContexts,
    msgid: string,
    msgidPlural: string,
    count: number,
  ): string;
}

export const messages = new Gettext(gettextOptions) as GettextWithAppContexts;
messages.setTextDomain('messages');
setErrorHandler(messages);

export const relayLocations = new Gettext(gettextOptions);
relayLocations.setTextDomain('relay-locations');
setErrorHandler(relayLocations);

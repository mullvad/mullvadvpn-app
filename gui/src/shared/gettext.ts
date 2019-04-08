import log from 'electron-log';
import fs from 'fs';
import { po } from 'gettext-parser';
import Gettext from 'node-gettext';
import path from 'path';

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

  for (const locale of preferredLocales) {
    if (parseTranslation(locale, 'messages', catalogue)) {
      log.info(`Loaded translations for ${locale}`);
      catalogue.setLocale(locale);
      return;
    }
  }
}

function parseTranslation(locale: string, domain: string, catalogue: Gettext): boolean {
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

function setErrorHandler(catalogue: Gettext) {
  catalogue.on('error', (error) => {
    // NOTE: locale is not publicly exposed
    const catalogueLocale = (catalogue as any)['locale'];

    // Filter out the "no translation was found" errors for the source language
    if (catalogueLocale === SOURCE_LANGUAGE && error.indexOf('No translation was found') !== -1) {
      return;
    }

    log.warn(`Gettext error: ${error}`);
  });
}

// `{debug: false}` option prevents Gettext from printing the warnings to console in development
// the errors are handled separately in the "error" handler below
export const messages = new Gettext({ debug: false });
messages.setTextDomain('messages');
setErrorHandler(messages);

export const countries = new Gettext({ debug: false });
countries.setTextDomain('countries');
setErrorHandler(countries);

export const cities = new Gettext({ debug: false });
cities.setTextDomain('cities');
setErrorHandler(cities);

export const relayLocations = new Gettext({ debug: false });
relayLocations.setTextDomain('relay-locations');
setErrorHandler(relayLocations);



import { GetTextTranslations } from 'gettext-parser';
import Gettext from 'node-gettext';
import log from '../../shared/logging';

const SOURCE_LANGUAGE = 'en';

export function loadTranslations(
  catalogue: Gettext,
  locale: string,
  translations?: GetTextTranslations,
) {
  if (translations) {
    catalogue.addTranslations(locale, catalogue.domain, translations);
    catalogue.setLocale(locale);
    log.info(`Loaded translations ${locale}/${catalogue.domain}`);
  } else {
    // Reset the locale to source language if we couldn't load the catalogue for the requested locale
    // Add empty translations to suppress some of the warnings produces by node-gettext
    catalogue.addTranslations(SOURCE_LANGUAGE, catalogue.domain, {});
    catalogue.setLocale(SOURCE_LANGUAGE);
  }
}

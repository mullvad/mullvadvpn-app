import fs from 'fs';
import { GetTextTranslations, po } from 'gettext-parser';
import Gettext from 'node-gettext';
import path from 'path';

import log from '../shared/logging';

const SOURCE_LANGUAGE = 'en';
const LOCALES_DIR = path.resolve(__dirname, 'locales');

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
  const aliasedLocale = getAliasedLocale(locale);
  const filename = path.join(LOCALES_DIR, aliasedLocale, `${domain}.po`);
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

  if (locale === 'sv-rö') {
    robberifyTranslations(translations);
  }

  catalogue.addTranslations(aliasedLocale, domain, translations);

  return translations;
}

function getAliasedLocale(locale: string): string {
  return locale === 'sv-rö' ? 'sv' : locale;
}

function robberifyTranslations(translations: GetTextTranslations) {
  for (const contextKey in translations.translations) {
    const context = translations.translations[contextKey];

    for (const messageKey in context) {
      const message = context[messageKey];

      if (!message.msgstr[0].startsWith('Content-Type:')) {
        message.msgstr = message.msgstr.map((msgstr) => robberifyString(msgstr));
      }
    }
  }
}

const CONSONANTS = 'bcdfghjklmnpqrstvxzBCDFGHJKLMNPQRSTVXZ';
const PLACEHOLDER_TYPES = ['s', 'd'];

function robberifyString(value: string): string {
  let robberValue = '';

  const chars = value.split('');
  let skipDueTo: string | null = null;
  for (let i = 0; i < chars.length; i++) {
    const char = chars[i];
    const nextChar = i < chars.length - 1 ? chars[i + 1] : '';

    if (char === '<') {
      skipDueTo = char;
      robberValue += char;
    } else if (skipDueTo === '<' && char === '>') {
      skipDueTo = null;
      robberValue += char;
    } else if (char === '%' && PLACEHOLDER_TYPES.includes(nextChar)) {
      robberValue += `${char}${nextChar}`;
      i++;
    } else if (char === '%' && nextChar === '(') {
      skipDueTo = char;
      robberValue += `${char}${nextChar}`;
      i++;
    } else if (skipDueTo === '%' && char === ')' && PLACEHOLDER_TYPES.includes(nextChar)) {
      skipDueTo = null;
      robberValue += `${char}${nextChar}`;
      i++;
    } else if (skipDueTo === null && CONSONANTS.includes(char)) {
      robberValue += `${char}o${char.toLowerCase()}`;
    } else {
      robberValue += char;
    }
  }

  return robberValue;
}

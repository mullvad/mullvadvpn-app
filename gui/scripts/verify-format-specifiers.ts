import fs from 'fs';
import { GetTextTranslation, po } from 'gettext-parser';
import path from 'path';

const LOCALES_DIR = path.join('..', 'locales');

function getLocales(): string[] {
  const localesContent = fs.readdirSync(LOCALES_DIR);
  const localeDirectories = localesContent.filter((item) =>
    fs.statSync(path.join(LOCALES_DIR, item)).isDirectory(),
  );
  return localeDirectories;
}

function parseTranslationsForLocale(locale: string): GetTextTranslation[] {
  const poFileContents = fs.readFileSync(path.join(LOCALES_DIR, locale, 'messages.po'));
  const contexts = po.parse(poFileContents).translations;

  const translations = Object.values(contexts)
    .flatMap((context) => Object.values(context))
    .filter((translation) => translation.msgid !== '');

  return translations;
}

function getFormatSpecifiers(text: string): string[] {
  // Matches both %(name)s and %s.
  return text.match(/%(\(.*?\))?[a-z]/g) ?? [];
}

function formatSpecifiersEquals(source: string[], translation: string[]): boolean {
  const sortedTranslation = translation.sort();
  return (
    source.length === translation.length &&
    source.sort().every((value, index) => value === sortedTranslation[index])
  );
}

function checkTranslationImpl(msgid: string, msgstr: string): boolean {
  const sourceFormatSpecifiers = getFormatSpecifiers(msgid);
  const translationFormatSpecifiers = getFormatSpecifiers(msgstr);
  return formatSpecifiersEquals(sourceFormatSpecifiers, translationFormatSpecifiers);
}

function checkTranslation(translation: GetTextTranslation): boolean {
  return translation.msgstr
    .map((msgstr) => {
      // Make sure that the translation matches either the singular or plural.
      const equal =
        checkTranslationImpl(translation.msgid, msgstr) ||
        (translation.msgid_plural && checkTranslationImpl(translation.msgid_plural, msgstr));

      if (!equal) {
        console.error(`Error in "${translation.msgid}", "${msgstr}"`);
      }

      return equal;
    })
    .every((result) => result);
}

const isCorrect = getLocales()
  .map(parseTranslationsForLocale)
  // Map first to output all errors
  .map((translations) => translations.every(checkTranslation))
  .every((result) => result);

if (isCorrect) {
  console.log('Looks good!');
} else {
  console.error('See above errors');
  process.exit(1);
}

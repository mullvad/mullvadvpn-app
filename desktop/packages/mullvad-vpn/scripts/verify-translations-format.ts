import fs from 'fs';
import { GetTextTranslation, po } from 'gettext-parser';
import path from 'path';

import { ALLOWED_TAGS } from '../src/renderer/lib/html-formatter';
import { ValueOfArray } from '../src/shared/utility-types';

const LOCALES_DIR = path.join('locales');

const ALLOWED_VOID_TAGS = ['br'];

// Make sure to report these strings to crowdin. View this as a temporary escape
// hatch, not a permanent solution.
const IGNORED_STRINGS: Set<string> = new Set([
  // German translation. Has been reported to Crowdin.
  'Daher verwenden wir automatisch Multihop, um %(daita)s mit jedem Server zu aktivieren.',
]);

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

function checkFormatSpecifiersImpl(msgid: string, msgstr: string): boolean {
  const sourceFormatSpecifiers = getFormatSpecifiers(msgid);
  const translationFormatSpecifiers = getFormatSpecifiers(msgstr);
  return formatSpecifiersEquals(sourceFormatSpecifiers, translationFormatSpecifiers);
}

function checkFormatSpecifiers(translation: GetTextTranslation): boolean {
  return translation.msgstr
    .map((msgstr) => {
      // Make sure that the translation matches either the singular or plural.
      const equal =
        checkFormatSpecifiersImpl(translation.msgid, msgstr) ||
        (translation.msgid_plural && checkFormatSpecifiersImpl(translation.msgid_plural, msgstr));

      if (!equal) {
        console.error(`Error in "${translation.msgid}", "${msgstr}"`);
      }

      return equal;
    })
    .every((result) => result);
}

function isAllowedTag(tag: string): tag is ValueOfArray<typeof ALLOWED_TAGS> {
  return ALLOWED_TAGS.some((allowedTag) => tag === allowedTag);
}

function checkHtmlTagsImpl(value: string): { correct: boolean; amount: number } {
  const tagsRegexp = new RegExp('<.*?>', 'g');
  const tags = value.match(tagsRegexp) ?? [];
  const tagTypes = tags.map((tag) => tag.slice(1, -1));

  // Make sure tags match by pushing start-tags to a stack and matching closing tags with the last
  // item.
  const tagStack: string[] = [];
  for (let tag of tagTypes) {
    const selfClosing = tag.endsWith('/');
    const endTag = tag.startsWith('/');
    tag = endTag ? tag.slice(1) : selfClosing ? tag.slice(0, -1).trim() : tag;

    if (!isAllowedTag(tag)) {
      console.error(`Tag "<${tag}>" not allowed: "${value}"`);
      return { correct: false, amount: NaN };
    }

    if (!ALLOWED_VOID_TAGS.includes(tag)) {
      if (endTag) {
        // End tags require a matching start tag.
        if (tag !== tagStack.pop()) {
          console.error(`Closing non-existent start-tag (</${tag}>) in "${value}"`);
          return { correct: false, amount: NaN };
        }
      } else {
        tagStack.push(tag);
      }
    }
  }

  if (tagStack.length > 0) {
    console.error(`Missing closing-tags (${tagStack}) in "${value}"`);
    return { correct: false, amount: NaN };
  }

  return { correct: true, amount: tags.length / 2 };
}

function checkHtmlTags(translation: GetTextTranslation): boolean {
  const { correct, amount: sourceAmount } = checkHtmlTagsImpl(translation.msgid);

  const translationsCorrect = translation.msgstr.every((value) => {
    const { correct, amount } = checkHtmlTagsImpl(value);
    // The amount doesn't make sense if the string isn't correctly formatted.
    if (correct && amount !== sourceAmount) {
      console.error(
        `Incorrect amount of tags in translation for "${translation.msgid}": "${value}"`,
      );
    }
    return correct && amount === sourceAmount;
  });

  return correct && translationsCorrect;
}

function checkTranslation(translation: GetTextTranslation): boolean {
  // Ignore some strings
  if (translation.msgstr.every((str) => IGNORED_STRINGS.has(str))) {
    return true;
  }
  return checkFormatSpecifiers(translation) && checkHtmlTags(translation);
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

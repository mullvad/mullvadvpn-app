const { GettextExtractor, JsExtractors, HtmlExtractors } = require('gettext-extractor');
const path = require('path');

const extractor = new GettextExtractor();
const outputPotFile = path.resolve(__dirname, '../locales/messages.pot');
const comments = {
  otherLineLeading: true,
  sameLineLeading: true,
  regex: /^TRANSLATORS:\s*(.*)$/,
};

extractor
  .createJsParser([
    JsExtractors.callExpression('gettext', {
      arguments: {
        text: 0,
      },
      comments,
    }),
    JsExtractors.callExpression('pgettext', {
      arguments: {
        context: 0,
        text: 1,
      },
      comments,
    }),
    JsExtractors.callExpression('ngettext', {
      arguments: {
        text: 0,
        textPlural: 1,
      },
      comments,
    }),
    JsExtractors.callExpression('npgettext', {
      arguments: {
        context: 0,
        text: 1,
        textPlural: 2,
      },
      comments,
    }),
  ])
  .parseFilesGlob('./src/**/*.@(ts|tsx)', {
    cwd: path.resolve(__dirname, '..'),
  });

extractor.savePotFile(outputPotFile);
extractor.printStats();

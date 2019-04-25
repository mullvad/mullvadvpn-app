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
    JsExtractors.callExpression('messages.gettext', {
      arguments: {
        text: 0,
      },
      comments,
    }),
    JsExtractors.callExpression('messages.pgettext', {
      arguments: {
        context: 0,
        text: 1,
      },
      comments,
    }),
    JsExtractors.callExpression('messages.ngettext', {
      arguments: {
        text: 0,
        textPlural: 1,
      },
      comments,
    }),
    JsExtractors.callExpression('messages.npgettext', {
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

// clean file references
extractor.getMessages().forEach((msg) => {
  msg.references = [];
});
extractor.savePotFile(outputPotFile);
extractor.printStats();

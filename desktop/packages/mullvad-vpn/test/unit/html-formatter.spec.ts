import { describe, it } from 'mocha';
import React from 'react';

import { formatHtml } from '../../src/renderer/lib/html-formatter';
import { createFragment, expectChildrenToMatch, expectChildrenToMatchElements } from './utils';

describe('Format html', () => {
  it('should format middle bold tag', () => {
    expectChildrenToMatch(formatHtml('Some <b>bold</b> text'), ['Some ', 'bold', ' text']);
  });
  it('should format starting bold tag', () => {
    expectChildrenToMatch(formatHtml('<b>Some</b> bold text'), ['Some', ' bold text']);
  });
  it('should format ending bold tag', () => {
    expectChildrenToMatch(formatHtml('Some bold <b>text</b>'), ['Some bold ', 'text']);
  });
  it('should format multiple bold tags', () => {
    expectChildrenToMatch(formatHtml('Some <b>bold</b> and <b>more bold</b> text'), [
      'Some ',
      'bold',
      ' and ',
      'more bold',
      ' text',
    ]);
  });
  it('should produce reliable output on each call', () => {
    expectChildrenToMatch(formatHtml('<b>Some</b> bold text'), ['Some', ' bold text']);
    expectChildrenToMatch(formatHtml('Some non bold text'), ['Some non bold text']);
    // Same string used as in first expectChildrenToMatch call
    expectChildrenToMatch(formatHtml('<b>Some</b> bold text'), ['Some', ' bold text']);
  });
  it('should format middle emphasis tag', () => {
    expectChildrenToMatch(formatHtml('Some <em>emphasized</em> text'), [
      'Some ',
      'emphasized',
      ' text',
    ]);
  });
  it('should format starting emphasis tag', () => {
    expectChildrenToMatch(formatHtml('<em>Some</em> emphasized text'), [
      'Some',
      ' emphasized text',
    ]);
  });
  it('should format ending emphasis tag', () => {
    expectChildrenToMatch(formatHtml('Some emphasized <em>text</em>'), [
      'Some emphasized ',
      'text',
    ]);
  });
  it('should format multiple emphasis tags', () => {
    expectChildrenToMatch(
      formatHtml('Some <em>emphasized</em> and <em>more emphasized</em> text'),
      ['Some ', 'emphasized', ' and ', 'more emphasized', ' text'],
    );
  });
  it('should format both bold and emphasis tags', () => {
    expectChildrenToMatch(formatHtml('Some <b>bold</b> and <em>emphasized</em> text'), [
      'Some ',
      'bold',
      ' and ',
      'emphasized',
      ' text',
    ]);
  });
  it('should format multiple bold and emphasis tags', () => {
    expectChildrenToMatch(
      formatHtml(
        'Some <b>bold</b> and <em>emphasized</em> text. Then another <b>bold text</b> and one more <em>text</em> which was emphasized.',
      ),
      [
        'Some ',
        'bold',
        ' and ',
        'emphasized',
        ' text. Then another ',
        'bold text',
        ' and one more ',
        'text',
        ' which was emphasized.',
      ],
    );
  });
  it('should format using custom transformer', () => {
    expectChildrenToMatch(
      formatHtml('Some <b>bold</b> text', {
        b: () => createFragment('override value'),
      }),
      ['Some ', 'override value', ' text'],
    );
  });
  it('should format using custom transformer and produce correct output', () => {
    expectChildrenToMatchElements(
      formatHtml('Some <b>bold</b> text', {
        b: (value) => React.createElement('div', null, value),
      }),
      [createFragment('Some '), React.createElement('div', null, 'bold'), createFragment(' text')],
    );
  });
});

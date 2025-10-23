import React, { JSX } from 'react';
import styled from 'styled-components';

import { type ValueOfArray } from '../../shared/utility-types';
import { colors } from './foundations';

const Bold = styled.span({ fontWeight: 700 });
const Emphasis = styled.em({ color: colors.white, fontWeight: 600 });

export const ALLOWED_TAGS = ['b', 'br', 'em'] as const;
export type AllowedTags = ValueOfArray<typeof ALLOWED_TAGS>;

export type TransformerComponent = React.ComponentType<{ children: React.ReactNode }>;
export type TransformerMap = Record<AllowedTags, TransformerComponent>;

const defaultTransformers: Partial<TransformerMap> = {
  b: Bold,
  em: Emphasis,
};

export function formatHtml(
  inputString: string,
  customTransformers?: Partial<TransformerMap>,
): React.ReactElement {
  const transformers = {
    ...defaultTransformers,
    ...customTransformers,
  };

  const inputStringArray: Array<string | JSX.Element> = [inputString];

  const transformedStrings = Object.entries(transformers).reduce((strings, [key, Component]) => {
    const newStrings = strings.flatMap((value) => {
      if (typeof value === 'string') {
        const matchPattern = `(<${key}>.*?</${key}>)`;
        const transformerMatcher = new RegExp(matchPattern, 'g');

        // If the value is a string we should see if our current transformer should transform it.
        return value
          .split(transformerMatcher)
          .filter((v) => v.length > 0)
          .map((substring) => {
            // Create a new RegExp object to avoid `lastIndex` side effects, see:
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/lastIndex#avoiding_side_effects

            // If the value is matched for the current transformer then it should be turned into a component
            if (transformerMatcher.test(substring)) {
              const replacePattern = `<${key}>|</${key}>`;
              const transformReplacer = new RegExp(replacePattern, 'g');

              const valueWithoutTags = substring.replaceAll(transformReplacer, '');

              return <Component key={substring}>{valueWithoutTags}</Component>;
            } else {
              // If the value is not a match for the current transformer we should return the string as is,
              // so it can be potentially manipulated by a later transformer
              return substring;
            }
          });
      } else {
        // If the value is not a string it has already been transformed into a component by a transformer in a previous iteration.
        return value;
      }
    });

    return newStrings;
  }, inputStringArray);

  // After all the transformers have been applied,
  // loop over all non-transformed strings and wrap them in fragments
  const htmlFormattedInputString = transformedStrings.map((value) =>
    typeof value === 'string' ? <React.Fragment key={value}>{value}</React.Fragment> : value,
  );

  return <React.Fragment>{htmlFormattedInputString}</React.Fragment>;
}

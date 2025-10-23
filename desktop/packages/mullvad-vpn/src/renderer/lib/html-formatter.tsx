import React, { JSX } from 'react';
import styled from 'styled-components';

import { colors } from './foundations';

const Bold = styled.span({ fontWeight: 700 });
const Emphasis = styled.em({ color: colors.white, fontWeight: 600 });

// When a new tag is added here, it must also be added to allowed tags in
// our verify translations script.
const testMap: Partial<
  Record<
    keyof JSX.IntrinsicElements,
    {
      test: RegExp;
      replace: RegExp;
    }
  >
> = {
  b: {
    test: /(<b>.*?<\/b>)/g,
    replace: /<b>|<\/b>/g,
  },
  em: {
    test: /(<em>.*?<\/em>)/g,
    replace: /<em>|<\/em>/g,
  },
} as const;

const componentMap: Partial<
  Record<keyof JSX.IntrinsicElements, React.ComponentType<{ children: React.ReactNode }>>
> = {
  b: Bold,
  em: Emphasis,
} as const;

export function formatHtml(inputString: string): React.ReactElement {
  const inputStringArray: Array<string | JSX.Element> = [inputString];

  const transformedStrings = Object.entries(testMap).reduce((strings, [key, { test, replace }]) => {
    const newStrings = strings.flatMap((value) => {
      // If the value is a string:
      //   it can be transformed by a transformer into a component.
      if (typeof value === 'string') {
        // If the value is a string we should see if our current transformer should transform it.
        return value
          .split(test)
          .filter((v) => v.length > 0)
          .map((substring) => {
            // Create a new RegExp object to avoid `lastIndex` side effects, see:
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/lastIndex#avoiding_side_effects
            const tester = new RegExp(test);

            // If the value is matched for the current transformer then it should be turned into a component
            if (tester.test(substring)) {
              const Component = componentMap[key as keyof typeof componentMap]!;
              const valueWithoutTags = substring.replaceAll(replace, '');

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

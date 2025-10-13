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
  const formattedString: JSX.Element[] = [];

  Object.entries(testMap).forEach(([key, { test, replace }]) => {
    const parts = inputString.split(test).filter((part) => part !== '');
    if (parts.length <= 1) {
      return;
    }

    parts.map((value, index) => {
      if (test.test(value)) {
        const Component = componentMap[key as keyof typeof componentMap]!;
        const valueWithoutTags = value.replaceAll(replace, '');

        formattedString.push(<Component key={index}>{valueWithoutTags}</Component>);
      } else {
        formattedString.push(<React.Fragment key={index}>{value}</React.Fragment>);
      }
    });
  });

  if (formattedString.length === 0) {
    formattedString.push(<React.Fragment key={inputString}>{inputString}</React.Fragment>);
  }

  return <>{formattedString}</>;
}

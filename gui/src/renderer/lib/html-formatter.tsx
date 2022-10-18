import React from 'react';
import styled from 'styled-components';

const boldSyntax = /(<b>.*?<\/b>)/g;
const Bold = styled.span({ fontWeight: 700 });

export function formatHtml(inputString: string): React.ReactElement {
  const formattedString = inputString.split(boldSyntax).map((value, index) => {
    if (boldSyntax.test(value)) {
      const valueWithoutTags = value.replaceAll(/<b>|<\/b>/g, '');
      return <Bold key={index}>{valueWithoutTags}</Bold>;
    } else {
      return <React.Fragment key={index}>{value}</React.Fragment>;
    }
  });

  return <>{formattedString}</>;
}

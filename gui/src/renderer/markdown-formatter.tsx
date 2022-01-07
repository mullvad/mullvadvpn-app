import React from 'react';
import styled from 'styled-components';

const boldSyntax = '**';
const Bold = styled.span({ fontWeight: 700 });

export function formatMarkdown(inputString: string): React.ReactElement {
  const formattedString = inputString
    .split(boldSyntax)
    .map((value, index) =>
      index % 2 === 0 ? (
        <React.Fragment key={index}>{value}</React.Fragment>
      ) : (
        <Bold key={index}>{value}</Bold>
      ),
    );

  return <>{formattedString}</>;
}

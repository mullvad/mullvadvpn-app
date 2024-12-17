import React from 'react';
import { createGlobalStyle } from 'styled-components';

import { colors, fontFamilies, fontSizes, fontWeights, lineHeights, radius, spacings } from './';

type VariablesProps = React.PropsWithChildren<object>;

const GlobalStyle = createGlobalStyle`
  :root {
    ${Object.entries({
      ...colors,
      ...spacings,
      ...radius,
      ...fontFamilies,
      ...fontSizes,
      ...fontWeights,
      ...lineHeights,
    }).reduce((styleString, [key, value]) => ({ ...styleString, [key]: value }), {})}
  }
`;

export const VariablesGlobalStyle = ({ children }: VariablesProps) => {
  return (
    <>
      <GlobalStyle />
      {children}
    </>
  );
};

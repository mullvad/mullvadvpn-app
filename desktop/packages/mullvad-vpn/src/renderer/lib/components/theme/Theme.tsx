import React from 'react';
import { createGlobalStyle } from 'styled-components';

import {
  colorPrimitives,
  colors,
  fontFamilies,
  fontSizes,
  fontWeights,
  lineHeights,
  radius,
  spacingPrimitives,
} from '../../foundations/variables';

type VariablesProps = React.PropsWithChildren<object>;

const GlobalStyle = createGlobalStyle`
  :root {
    ${Object.entries({
      ...spacingPrimitives,
      ...colorPrimitives,
      ...radius,
      ...fontFamilies,
      ...fontSizes,
      ...fontWeights,
      ...lineHeights,
    }).reduce((styleString, [key, value]) => ({ ...styleString, [key]: value }), {})}
  }

  body {
    background-color: ${colors.darkBlue};
  }
`;

export const Theme = ({ children }: VariablesProps) => {
  return (
    <>
      <GlobalStyle />
      {children}
    </>
  );
};

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

    // Keep the app's own theme under Windows forced-colors (high contrast) mode, since it
    // otherwise strips colors from custom-styled controls like the switch, making them invisible.
    forced-color-adjust: none;
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

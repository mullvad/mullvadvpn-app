import React from 'react';
import { createGlobalStyle } from 'styled-components';

import {
  colors,
  fontFamilies,
  fontSizes,
  fontWeights,
  lineHeights,
  radius,
  spacings,
} from '../../foundations/variables';

type VariablesProps = React.PropsWithChildren<object>;

const VariablesGlobalStyle = createGlobalStyle`
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

export const Theme = ({ children }: VariablesProps) => {
  return (
    <>
      <VariablesGlobalStyle />
      {children}
    </>
  );
};

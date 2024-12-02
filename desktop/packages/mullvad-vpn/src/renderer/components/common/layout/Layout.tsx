import styled from 'styled-components';

import { Spacings } from '../../../tokens';
import { m, margin } from './margin';
import { p, padding } from './padding';
import { LayoutSpacings } from './types';

interface LayoutProps {
  $margin?: Spacings | LayoutSpacings;
  $padding?: Spacings | LayoutSpacings;
}

export const Layout = styled.div<LayoutProps>`
  ${({ $margin }) => {
    if (!$margin) return '';
    if (typeof $margin === 'string') return m($margin);

    return Object.entries($margin)
      .map(([key, value]) => margin[key as keyof LayoutSpacings](value))
      .join(' ');
  }}

  ${({ $padding }) => {
    if (!$padding) return '';
    if (typeof $padding === 'string') return p($padding);

    return Object.entries($padding)
      .map(([key, value]) => padding[key as keyof LayoutSpacings](value))
      .join(' ');
  }}
`;

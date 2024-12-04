import styled from 'styled-components';

import { Spacings } from '../../../tokens';
import { margin } from './margin';
import { padding } from './padding';
import { LayoutSpacings } from './types';

export interface LayoutProps {
  $margin?: Spacings | LayoutSpacings;
  $padding?: Spacings | LayoutSpacings;
}

const combine = (
  funcs: Record<keyof LayoutSpacings, (value: Spacings) => React.CSSProperties>,
  spacings?: Spacings | LayoutSpacings,
): React.CSSProperties => {
  if (!spacings) return {};

  if (typeof spacings === 'string') return funcs.all(spacings);

  const result: React.CSSProperties = {};
  for (const [key, value] of Object.entries(spacings)) {
    Object.assign(result, funcs[key as keyof LayoutSpacings](value));
  }

  return result;
};

export const Layout = styled.div<LayoutProps>(({ $margin, $padding }) => ({
  ...combine(margin, $margin),
  ...combine(padding, $padding),
}));

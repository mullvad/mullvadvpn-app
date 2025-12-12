import styled from 'styled-components';

import { Spacings } from '../../foundations';
import { TransientProps } from '../../types';
import { margin } from './margin';
import { padding } from './padding';
import { LayoutSpacings } from './types';

export type LayoutProps = React.ComponentPropsWithRef<'div'> & {
  margin?: Spacings | LayoutSpacings;
  padding?: Spacings | LayoutSpacings;
};

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

const StyledLayout = styled.div<TransientProps<LayoutProps>>`
  ${({ $margin, $padding }) => ({
    ...combine(margin, $margin),
    ...combine(padding, $padding),
  })}
`;

export function Layout({ margin, padding, ...props }: LayoutProps) {
  return <StyledLayout $margin={margin} $padding={padding} {...props} />;
}

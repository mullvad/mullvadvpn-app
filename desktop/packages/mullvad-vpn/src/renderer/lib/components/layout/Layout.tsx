import styled from 'styled-components';

import { Spacings } from '../../foundations';
import { type PolymorphicProps, TransientProps } from '../../types';
import { margin } from './margin';
import { padding } from './padding';
import { LayoutSpacings } from './types';

type LayoutStyles = {
  margin?: Spacings | LayoutSpacings;
  padding?: Spacings | LayoutSpacings;
};

export type LayoutProps<T extends React.ElementType = 'div'> = PolymorphicProps<T, LayoutStyles>;

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

const StyledLayout = styled.div<TransientProps<LayoutStyles>>`
  ${({ $margin, $padding }) => ({
    ...combine(margin, $margin),
    ...combine(padding, $padding),
  })}
`;

export function Layout<T extends React.ElementType = 'div'>({
  margin,
  padding,
  ...props
}: LayoutProps<T>) {
  return <StyledLayout $margin={margin} $padding={padding} {...props} />;
}

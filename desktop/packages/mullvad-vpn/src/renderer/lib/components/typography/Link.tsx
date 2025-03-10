import styled from 'styled-components';
import { KnownTarget } from 'styled-components/dist/types';

import { Colors, Radius } from '../../foundations';
import { Text, TextProps } from './Text';

export type LinkProps<T extends KnownTarget = 'a'> = TextProps<T> & {
  onClick?: (e: React.MouseEvent<HTMLAnchorElement>) => void;
};

const StyledText = styled(Text)<{
  $hoverColor: Colors | undefined;
}>((props) => ({
  background: 'transparent',
  cursor: 'default',
  textDecoration: 'none',
  display: 'inline-flex',
  width: 'fit-content',

  '&&:hover': {
    textDecorationLine: 'underline',
    textUnderlineOffset: '2px',
    color: props.$hoverColor,
  },
  '&&:focus-visible': {
    borderRadius: Radius.radius4,
    outline: `2px solid ${Colors.white}`,
    outlineOffset: '2px',
  },
}));

const getHoverColor = (color: Colors | undefined) => {
  switch (color) {
    case Colors.white60:
      return Colors.white;
    default:
      return undefined;
  }
};

export const Link = <T extends KnownTarget = 'a'>({
  as = 'a' as T,
  children,
  color,
  ...props
}: LinkProps<T>) => {
  return (
    <StyledText forwardedAs={as} color={color} $hoverColor={getHoverColor(color)} {...props}>
      {children}
    </StyledText>
  );
};

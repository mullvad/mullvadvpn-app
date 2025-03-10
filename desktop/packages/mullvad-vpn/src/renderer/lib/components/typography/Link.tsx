import styled, { WebTarget } from 'styled-components';

import { Colors, Radius } from '../../foundations';
import { Text, TextProps } from './Text';

export type LinkProps<T extends WebTarget> = TextProps<T> & {
  onClick?: (e: React.MouseEvent<HTMLAnchorElement>) => void;
};

const StyledText = styled(Text)<{
  $hoverColor: Colors | undefined;
}>((props) => ({
  background: 'transparent',
  cursor: 'default',
  textDecoration: 'none',
  width: 'fit-content',

  '&:hover': {
    textDecorationLine: 'underline',
    textUnderlineOffset: '2px',
    color: props.$hoverColor,
  },
  '&:focus-visible': {
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

export const Link = <T extends WebTarget = 'a'>({
  as,
  children,
  color,
  ...props
}: LinkProps<T>) => {
  return (
    <StyledText forwardedAs={as ?? 'a'} color={color} $hoverColor={getHoverColor(color)} {...props}>
      {children}
    </StyledText>
  );
};

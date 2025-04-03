import React from 'react';
import styled from 'styled-components';

import { Colors, Radius } from '../../foundations';
import { Text, TextProps } from '../typography';
import { LinkIcon } from './components';

export type LinkProps<T extends React.ElementType = 'a'> = TextProps<T> & {
  onClick?: (e: React.MouseEvent<HTMLAnchorElement>) => void;
};

const StyledText = styled(Text)<{
  $hoverColor: Colors | undefined;
}>((props) => ({
  background: 'transparent',
  cursor: 'default',
  textDecoration: 'none',
  display: 'inline',

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

const Link = <T extends React.ElementType = 'a'>({
  as: forwardedAs,
  color,
  ...props
}: LinkProps<T>) => {
  // If `as` is provided we need to pass it as `forwardedAs` for it to
  // be correctly passed to the `Text` component.
  const componentProps = forwardedAs ? { ...props, forwardedAs } : props;
  return (
    <StyledText
      forwardedAs="a"
      color={color}
      $hoverColor={getHoverColor(color)}
      {...componentProps}
    />
  );
};

const LinkNamespace = Object.assign(Link, {
  Icon: LinkIcon,
});

export { LinkNamespace as Link };

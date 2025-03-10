import React, { useCallback } from 'react';
import styled from 'styled-components';

import { Colors, Radius } from '../../foundations';
import { useHistory } from '../../history';
import { RoutePath } from '../../routes';
import { buttonReset } from '../../styles';
import { Text, TextProps } from './Text';

export interface LinkProps extends Omit<TextProps<'button'>, 'color'> {
  to: RoutePath;
  color?: Colors;
}

const StyledText = styled(Text)<{
  $hoverColor: Colors | undefined;
}>((props) => ({
  ...buttonReset,
  background: 'transparent',

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

export const Link = ({ to, children, color, onClick, ...props }: LinkProps) => {
  const history = useHistory();
  const navigate = useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      if (onClick) {
        onClick(e);
      }
      return history.push(to);
    },
    [history, to, onClick],
  );
  return (
    <StyledText
      onClick={navigate}
      as={'button'}
      color={color}
      $hoverColor={getHoverColor(color)}
      {...props}>
      {children}
    </StyledText>
  );
};

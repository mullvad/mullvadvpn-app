import React, { useCallback } from 'react';
import styled from 'styled-components';

import { useHistory } from '../../../lib/history';
import { RoutePath } from '../../../lib/routes';
import { Colors, Radius } from '../../../tokens';
import { buttonReset } from '../mixins';
import { Text, TextProps } from './Text';

export interface LinkProps extends TextProps, Omit<React.HtmlHTMLAttributes<'button'>, 'color'> {
  to: RoutePath;
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
    (e: React.MouseEvent<'button'>) => {
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

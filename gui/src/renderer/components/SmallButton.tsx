import React from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { smallText } from './common-styles';

export enum SmallButtonColor {
  blue,
  red,
}

function getButtonColors(color?: SmallButtonColor, disabled?: boolean) {
  switch (color) {
    case SmallButtonColor.red:
      return {
        background: disabled ? colors.red60 : colors.red,
        backgroundHover: disabled ? colors.red60 : colors.red80,
      };
    case SmallButtonColor.blue:
    default:
      return {
        background: disabled ? colors.blue50 : colors.blue,
        backgroundHover: disabled ? colors.blue50 : colors.blue60,
      };
  }
}

const StyledSmallButton = styled.button<{ $color?: SmallButtonColor; disabled?: boolean }>(
  smallText,
  (props) => {
    const buttonColors = getButtonColors(props.$color, props.disabled);
    return {
      height: '32px',
      padding: '5px 16px',
      border: 'none',
      background: buttonColors.background,
      color: props.disabled ? colors.white50 : colors.white,
      borderRadius: '4px',
      marginLeft: '12px',

      [`${StyledSmallButtonGrid} &&`]: {
        marginLeft: 0,
      },

      '&&:hover': {
        background: buttonColors.backgroundHover,
      },
    };
  },
);

interface SmallButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'onClick' | 'color'> {
  onClick: () => void;
  children: string;
  color?: SmallButtonColor;
}

export function SmallButton(props: SmallButtonProps) {
  const { color, ...otherProps } = props;
  return <StyledSmallButton $color={props.color} {...otherProps} />;
}

export const SmallButtonGroup = styled.div<{ $noMarginTop?: boolean }>((props) => ({
  display: 'flex',
  justifyContent: 'end',
  margin: '0 23px',
  marginTop: props.$noMarginTop ? 0 : '30px',
}));

const StyledSmallButtonGrid = styled.div<{ $columns: number }>((props) => ({
  display: 'grid',
  gridTemplateColumns: `repeat(${props.$columns}, 1fr)`,
  gridColumnGap: '10px',
}));

interface SmallButtonGridProps {
  className?: string;
}

export function SmallButtonGrid(props: React.PropsWithChildren<SmallButtonGridProps>) {
  return (
    <StyledSmallButtonGrid
      $columns={React.Children.count(props.children)}
      className={props.className}>
      {props.children}
    </StyledSmallButtonGrid>
  );
}

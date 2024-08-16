import React from 'react';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { smallText } from './common-styles';
import { MultiButtonCompatibleProps } from './MultiButton';

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
    default:
      return {
        background: disabled ? colors.blue50 : colors.blue,
        backgroundHover: disabled ? colors.blue50 : colors.blue60,
      };
  }
}

const BUTTON_GROUP_GAP = 12;

interface StyledSmallButtonProps {
  $color?: SmallButtonColor;
  $textOffset?: number;
  disabled?: boolean;
}

const StyledSmallButton = styled.button<StyledSmallButtonProps>(smallText, (props) => {
  const buttonColors = getButtonColors(props.$color, props.disabled);

  const horizontalPadding = 16;
  const paddingLeft = horizontalPadding + Math.max(0, props.$textOffset ?? 0);
  const paddingRight = horizontalPadding + Math.abs(Math.min(0, props.$textOffset ?? 0));

  return {
    minHeight: '32px',
    padding: `5px ${paddingRight}px 5px ${paddingLeft}px`,
    border: 'none',
    background: buttonColors.background,
    color: props.disabled ? colors.white50 : colors.white,
    borderRadius: '4px',
    marginLeft: `${BUTTON_GROUP_GAP}px`,
    alignItems: 'center',
    justifyContent: 'center',

    [`${SmallButtonGroupStart} &&`]: {
      marginLeft: 0,
      marginRight: `${BUTTON_GROUP_GAP}px`,
    },

    [`${SmallButtonGrid} &&`]: {
      flex: '1 0 auto',
      marginLeft: 0,
      minWidth: `calc(50% - ${BUTTON_GROUP_GAP / 2}px)`,
      maxWidth: '100%',
    },

    '&&:hover': {
      background: buttonColors.backgroundHover,
    },
  };
});

interface SmallButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'onClick' | 'color'>,
    MultiButtonCompatibleProps {
  onClick: () => void;
  children: React.ReactNode;
  color?: SmallButtonColor;
}

export function SmallButton(props: SmallButtonProps) {
  const { color, textOffset, ...otherProps } = props;
  return <StyledSmallButton $color={props.color} $textOffset={props.textOffset} {...otherProps} />;
}

export const SmallButtonGroup = styled.div<{ $noMarginTop?: boolean }>((props) => ({
  display: 'flex',
  justifyContent: 'end',
  margin: '0 23px',
  marginTop: props.$noMarginTop ? 0 : '30px',
}));

export const SmallButtonGroupStart = styled(SmallButtonGroup)({
  flex: 1,
  justifyContent: 'start',
  margin: 0,
});

export const SmallButtonGrid = styled.div({
  display: 'flex',
  flexWrap: 'wrap',
  columnGap: `${BUTTON_GROUP_GAP}px`,
  rowGap: `${BUTTON_GROUP_GAP}px`,
});

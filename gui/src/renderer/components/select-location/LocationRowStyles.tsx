import styled from 'styled-components';
import { Styles } from 'styled-components/dist/types';

import { colors } from '../../../config.json';
import * as Cell from '../cell';
import { measurements, normalText } from '../common-styles';
import ImageView from '../ImageView';
import InfoButton from '../InfoButton';

interface ButtonColorProps {
  $backgroundColor: string;
  $backgroundColorHover: string;
}

export const buttonColor = (props: ButtonColorProps) => {
  return {
    backgroundColor: props.$backgroundColor,
    '&&:not(:disabled):hover': {
      backgroundColor: props.$backgroundColorHover,
    },
  };
};

export const StyledLocationRowContainer = styled(Cell.Container)({
  display: 'flex',
  padding: 0,
  background: 'none',
});

export const StyledLocationRowContainerWithMargin = styled(StyledLocationRowContainer)({
  marginBottom: 1,
});

export const StyledLocationRowLabel = styled(Cell.Label)(normalText, {
  flex: 1,
  minWidth: 0,
  fontWeight: 400,
  lineHeight: '24px',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
});

export const StyledLocationRowButton = styled(Cell.Row)<ButtonColorProps & { $level: number }>(
  buttonColor,
  (props) => {
    const paddingLeft = (props.$level + 1) * 16 + 2;

    return {
      display: 'flex',
      flex: 1,
      overflow: 'hidden',
      border: 'none',
      padding: `0 10px 0 ${paddingLeft}px`,
      margin: 0,
    };
  },
);

export const StyledLocationRowIcon = styled.button<ButtonColorProps>(buttonColor, {
  position: 'relative',
  alignSelf: 'stretch',
  paddingLeft: measurements.viewMargin,
  paddingRight: measurements.viewMargin,
  border: 0,

  '&&::before': {
    content: '""',
    position: 'absolute',
    margin: 'auto',
    top: 0,
    left: 0,
    bottom: 0,
    height: '50%',
    width: '1px',
    backgroundColor: colors.darkBlue,
  },
});

interface HoverButtonProps {
  $isLast?: boolean;
}

const hoverButton = (
  props: ButtonColorProps & HoverButtonProps,
): Styles<
  React.DetailedHTMLProps<React.ButtonHTMLAttributes<HTMLButtonElement>, HTMLButtonElement>
> => ({
  flex: 0,
  display: 'none',
  padding: '0 10px',
  paddingRight: props.$isLast ? '17px' : '10px',
  margin: 0,
  border: 0,
  height: measurements.rowMinHeight,
  appearance: 'none',

  '&&:last-child': {
    paddingRight: '25px',
  },

  '&&:not(:disabled):hover': {
    backgroundColor: props.$backgroundColor,
  },
  [`${StyledLocationRowContainer}:hover &&`]: {
    display: 'block',
  },
  [`${StyledLocationRowButton}:hover ~ &&`]: {
    backgroundColor: props.$backgroundColorHover,
  },
});

export const StyledHoverIconButton = styled.button<ButtonColorProps & HoverButtonProps>(
  buttonColor,
  hoverButton,
);

export const StyledHoverIcon = styled(ImageView).attrs({
  width: 18,
  height: 18,
  tintColor: colors.white60,
  tintHoverColor: colors.white,
})({
  [`${StyledHoverIconButton}:hover &&`]: {
    backgroundColor: colors.white,
  },
});

export const StyledHoverInfoButton = styled(InfoButton)<ButtonColorProps & HoverButtonProps>(
  buttonColor,
  hoverButton,
);

export function getButtonColor(selected: boolean, level: number, disabled?: boolean) {
  let backgroundColor = colors.blue60;
  if (selected) {
    backgroundColor = colors.green;
  } else if (level === 1) {
    backgroundColor = colors.blue40;
  } else if (level === 2) {
    backgroundColor = colors.blue20;
  } else if (level === 3) {
    backgroundColor = colors.blue10;
  }

  return {
    $backgroundColor: backgroundColor,
    $backgroundColorHover: selected || disabled ? backgroundColor : colors.blue80,
  };
}

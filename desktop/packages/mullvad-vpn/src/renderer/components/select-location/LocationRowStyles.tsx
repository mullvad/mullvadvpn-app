import styled from 'styled-components';
import { Styles } from 'styled-components/dist/types';

import { Icon } from '../../lib/components';
import { Colors } from '../../lib/foundations';
import * as Cell from '../cell';
import { buttonColor, ButtonColors } from '../cell/styles';
import { measurements, normalText } from '../common-styles';
import InfoButton from '../InfoButton';

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

export const StyledLocationRowButton = styled(Cell.Row)<ButtonColors & { $level: number }>(
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

interface HoverButtonProps {
  $isLast?: boolean;
}

const hoverButton = (
  props: ButtonColors & HoverButtonProps,
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

export const StyledHoverIconButton = styled.button<ButtonColors & HoverButtonProps>(
  buttonColor,
  hoverButton,
);

export const StyledHoverIcon = styled(Icon).attrs({
  color: Colors.white60,
})({
  [`${StyledHoverIconButton}:hover &&`]: {
    backgroundColor: Colors.white,
  },
});

export const StyledHoverInfoButton = styled(InfoButton)<ButtonColors & HoverButtonProps>(
  buttonColor,
  hoverButton,
);

export function getButtonColor(selected: boolean, level: number, disabled?: boolean) {
  let backgroundColor = Colors.blue60;
  if (selected) {
    backgroundColor = Colors.green;
  } else if (level === 1) {
    backgroundColor = Colors.blue40;
  } else if (level === 2) {
    backgroundColor = Colors.blue20;
  } else if (level === 3) {
    backgroundColor = Colors.blue10;
  }

  return {
    $backgroundColor: backgroundColor,
    $backgroundColorHover: selected || disabled ? backgroundColor : Colors.blue80,
  };
}

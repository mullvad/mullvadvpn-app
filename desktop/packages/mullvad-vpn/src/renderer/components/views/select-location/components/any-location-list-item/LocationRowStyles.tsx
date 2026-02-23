import styled from 'styled-components';

import { colors, ColorVariables } from '../../../../../lib/foundations';
import * as Cell from '../../../../cell';
import { buttonColor, ButtonColors } from '../../../../cell/styles';
import { normalText } from '../../../../common-styles';

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

export function getButtonColor(selected: boolean, level: number, disabled?: boolean) {
  let backgroundColor: ColorVariables = colors.blue60;
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

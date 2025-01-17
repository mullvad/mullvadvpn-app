import styled from 'styled-components';

import { colors } from '../../../config.json';
import { measurements } from '../common-styles';
import { buttonColor, ButtonColors } from './styles';

export const SideButton = styled.button<ButtonColors & { $noSeparator?: boolean }>(
  buttonColor,
  (props) => ({
    position: 'relative',
    alignSelf: 'stretch',
    paddingLeft: measurements.horizontalViewMargin,
    paddingRight: measurements.horizontalViewMargin,
    border: 0,

    '&&::before': {
      content: props.$noSeparator ? 'none' : '""',
      position: 'absolute',
      margin: 'auto',
      top: 0,
      left: 0,
      bottom: 0,
      height: '50%',
      width: '1px',
      backgroundColor: colors.darkBlue,
    },
  }),
);

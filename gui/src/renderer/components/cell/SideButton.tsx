import styled from 'styled-components';

import { colors } from '../../../config.json';
import { measurements } from '../common-styles';
import { buttonColor, ButtonColors } from './styles';

export const SideButton = styled.button<ButtonColors>(buttonColor, {
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

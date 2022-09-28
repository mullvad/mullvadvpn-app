import styled from 'styled-components';

import { colors } from '../../../config.json';
import { measurements } from '../common-styles';

export const Row = styled.div({
  display: 'flex',
  alignItems: 'center',
  backgroundColor: colors.blue,
  minHeight: measurements.rowMinHeight,
  paddingLeft: '22px',
  paddingRight: '22px',
  marginBottom: '1px',
});

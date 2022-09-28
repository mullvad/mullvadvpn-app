import styled from 'styled-components';

import { colors } from '../../../config.json';
import { measurements } from '../common-styles';
import { Group } from './Group';

export const Row = styled.div({
  display: 'flex',
  alignItems: 'center',
  backgroundColor: colors.blue,
  minHeight: measurements.rowMinHeight,
  paddingLeft: measurements.viewMargin,
  paddingRight: measurements.viewMargin,
  marginBottom: '1px',
  [`${Group} > &:last-child`]: {
    marginBottom: '0px',
  },
});

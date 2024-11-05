import styled from 'styled-components';

import { measurements } from '../common-styles';

interface IStyledGroupProps {
  $noMarginBottom?: boolean;
}

export const Group = styled.div<IStyledGroupProps>((props) => ({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: props.$noMarginBottom ? '0px' : measurements.rowVerticalMargin,
}));

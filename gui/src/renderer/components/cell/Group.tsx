import styled from 'styled-components';

import { measurements } from '../common-styles';

interface IStyledGroupProps {
  noMarginBottom?: boolean;
}

export const Group = styled.div({}, (props: IStyledGroupProps) => ({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: props.noMarginBottom ? '0px' : measurements.rowVerticalMargin,
}));

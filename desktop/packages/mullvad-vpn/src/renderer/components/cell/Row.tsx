import React from 'react';
import styled from 'styled-components';

import { Colors } from '../../lib/foundations';
import { measurements } from '../common-styles';
import { Group } from './Group';

interface RowProps extends React.HTMLAttributes<HTMLDivElement> {
  includeMarginBottomOnLast?: boolean;
}

export const Row = styled.div.withConfig({
  shouldForwardProp: (prop) => prop !== 'includeMarginBottomOnLast',
})<RowProps>((props) => ({
  display: 'flex',
  alignItems: 'center',
  backgroundColor: Colors.blue,
  minHeight: measurements.rowMinHeight,
  paddingLeft: measurements.horizontalViewMargin,
  paddingRight: measurements.horizontalViewMargin,
  marginBottom: '1px',
  [`${Group} > &&:last-child`]: {
    marginBottom: props.includeMarginBottomOnLast ? '1px' : '0px',
  },
}));

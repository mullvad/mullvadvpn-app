import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
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
  backgroundColor: colors.blue,
  minHeight: measurements.rowMinHeight,
  paddingLeft: measurements.viewMargin,
  paddingRight: measurements.viewMargin,
  marginBottom: '1px',
  [`${Group} > &&:last-child`]: {
    marginBottom: props.includeMarginBottomOnLast ? '1px' : '0px',
  },
}));

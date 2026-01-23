import React from 'react';
import styled from 'styled-components';

import { colors } from '../../foundations';

export type DividerProps = React.ComponentProps<'hr'>;

export const StyledDivider = styled.hr`
  border: none;
  border-top: 1px solid ${colors.whiteAlpha20};
  margin: 0;
  width: 100%;
`;

export function Divider(props: DividerProps) {
  return <StyledDivider {...props} />;
}

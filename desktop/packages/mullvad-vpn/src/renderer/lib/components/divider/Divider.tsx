import React from 'react';
import styled from 'styled-components';

import { colors } from '../../foundations';

export type DividerProps = React.ComponentProps<'hr'> & {
  decorative?: boolean;
};

export const StyledDivider = styled.hr`
  border: none;
  border-top: 1px solid ${colors.whiteAlpha20};
  margin: 0;
  width: 100%;
`;

export function Divider({ decorative = false, ...props }: DividerProps) {
  if (decorative) {
    return <StyledDivider as="div" aria-hidden={true} {...props} />;
  }
  return <StyledDivider aria-orientation="horizontal" {...props} />;
}

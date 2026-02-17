import React from 'react';
import styled from 'styled-components';

import { spacings } from '../../foundations';
import { Divider } from '../divider';
import { SectionTitleIconButton, SectionTitleText, SectionTitleTitle } from './components';

export type SectionTitleProps = React.ComponentProps<'div'>;

export const StyledSectionTitle = styled.div`
  display: grid;
  width: 100%;

  // Default to two columns
  grid-template-columns: auto 1fr;

  // If has three children, set three columns
  &&:has(> :nth-child(3)) {
    grid-template-columns: auto 1fr auto;
  }
  align-items: center;
  gap: ${spacings.small};
`;

function SectionTitle(props: SectionTitleProps) {
  return <StyledSectionTitle {...props} />;
}

const SectionTitleNamespace = Object.assign(SectionTitle, {
  Title: SectionTitleTitle,
  IconButton: SectionTitleIconButton,
  Divider: Divider,
  Text: SectionTitleText,
});

export { SectionTitleNamespace as SectionTitle };

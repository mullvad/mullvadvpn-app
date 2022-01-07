import React from 'react';
import styled from 'styled-components';
import { colors } from '../../../config.json';
import { buttonText } from '../common-styles';

const StyledSection = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

export const SectionTitle = styled.span(buttonText, {
  display: 'flex',
  minHeight: '44px',
  alignItems: 'center',
  backgroundColor: colors.blue,
  padding: '0 16px 0 22px',
  marginBottom: '1px',
});

export const CellSectionContext = React.createContext<boolean>(false);

export function Section(props: React.HTMLAttributes<HTMLDivElement>) {
  const { children, ...otherProps } = props;
  return (
    <StyledSection {...otherProps}>
      <CellSectionContext.Provider value={true}>{children}</CellSectionContext.Provider>
    </StyledSection>
  );
}

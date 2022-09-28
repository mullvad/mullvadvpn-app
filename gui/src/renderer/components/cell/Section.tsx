import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../config.json';
import { buttonText, openSans, sourceSansPro } from '../common-styles';
import { Row } from './Row';

const StyledSection = styled.div({
  display: 'flex',
  flexDirection: 'column',
});

interface SectionTitleProps {
  disabled?: boolean;
  thin?: boolean;
}

export const SectionTitle = styled(Row)(buttonText, (props: SectionTitleProps) => ({
  paddingRight: '16px',
  color: props.disabled ? colors.white20 : colors.white,
  fontWeight: props.thin ? 400 : 600,
  fontSize: props.thin ? '15px' : '18px',
  ...(props.thin ? openSans : sourceSansPro),
}));

export const CellSectionContext = React.createContext<boolean>(false);

export function Section(props: React.HTMLAttributes<HTMLDivElement>) {
  const { children, ...otherProps } = props;
  return (
    <StyledSection {...otherProps}>
      <CellSectionContext.Provider value={true}>{children}</CellSectionContext.Provider>
    </StyledSection>
  );
}

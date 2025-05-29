import styled from 'styled-components';

import { Text } from '../../typography';

export type AccordionTitleProps = {
  children?: React.ReactNode;
};

export const StyledTitleLabel = styled(Text)``;

export function AccordionTitle({ children }: AccordionTitleProps) {
  return (
    <StyledTitleLabel $padding="medium" color="white" variant="titleMedium">
      {children}
    </StyledTitleLabel>
  );
}

import styled from 'styled-components';

import { Text } from '../../typography';
import { useAccordionContext } from '../AccordionContext';

export type AccordionTitleProps = {
  children?: React.ReactNode;
};

export const StyledTitleLabel = styled(Text)``;

export function AccordionTitle({ children }: AccordionTitleProps) {
  const { titleId, disabled } = useAccordionContext();
  return (
    <StyledTitleLabel
      id={titleId}
      $padding="medium"
      color={disabled ? 'whiteAlpha60' : 'white'}
      variant="titleMedium">
      {children}
    </StyledTitleLabel>
  );
}

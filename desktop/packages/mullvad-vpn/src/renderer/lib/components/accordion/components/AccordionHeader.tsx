import styled from 'styled-components';

import { colors } from '../../../foundations';
import { Flex } from '../../flex';
import { StyledAccordionIcon } from './AccordionIcon';

export type AccordionHeaderProps = {
  children?: React.ReactNode;
};

export const StyledAccordionHeader = styled(Flex)`
  background-color: ${colors.blue};
  width: 100%;
  min-height: 48px;
  margin-bottom: 1px;

  && > ${StyledAccordionIcon} {
    margin-left: auto;
  }
`;

export function AccordionHeader({ children }: AccordionHeaderProps) {
  return (
    <StyledAccordionHeader
      $padding={{ horizontal: 'medium', vertical: 'small' }}
      $alignItems="center">
      {children}
    </StyledAccordionHeader>
  );
}

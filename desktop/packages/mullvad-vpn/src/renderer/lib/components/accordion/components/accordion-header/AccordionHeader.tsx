import styled from 'styled-components';

import { ListItem, ListItemProps } from '../../../list-item';
import { useAccordionContext } from '../../AccordionContext';

export type AccordionHeaderProps = ListItemProps;

export const StyledAccordionHeader = styled(ListItem)``;

export function AccordionHeader({ children, ...props }: AccordionHeaderProps) {
  const { disabled } = useAccordionContext();
  return (
    <StyledAccordionHeader disabled={disabled} {...props}>
      {children}
    </StyledAccordionHeader>
  );
}

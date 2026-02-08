import styled from 'styled-components';

import { ListItem, ListItemProps } from '../../../list-item';
import { StyledListItemTrailingAction } from '../../../list-item/components';
import { useAccordionContext } from '../../AccordionContext';
import { StyledAccordionContent } from '../AccordionContent';
import { StyledAccordionHeaderItem } from '../AccordionHeaderItem';

export type AccordionHeaderProps = ListItemProps;

export const StyledAccordionHeaderRoot = styled(ListItem)``;

export const StyledAccordionHeader = styled(StyledAccordionHeaderRoot)`
  ${StyledAccordionHeaderItem}, ${StyledListItemTrailingAction} {
    transition: border-radius 0.15s ease-out;
  }

  &:has(+ ${StyledAccordionContent}) {
    ${StyledAccordionHeaderItem} {
      border-bottom-left-radius: 0;
      border-bottom-right-radius: 0;
    }
    ${StyledListItemTrailingAction} {
      border-bottom-right-radius: 0;
    }
  }
`;

export function AccordionHeader({ children, ...props }: AccordionHeaderProps) {
  const { headerRef, disabled } = useAccordionContext();
  return (
    <StyledAccordionHeader ref={headerRef} disabled={disabled} {...props}>
      {children}
    </StyledAccordionHeader>
  );
}

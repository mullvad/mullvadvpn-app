import styled from 'styled-components';

import { ListItem, ListItemProps } from '../../../list-item';
import { StyledListItemTrailingAction } from '../../../list-item/components/list-item-trailing-actions/components';
import { useAccordionContext } from '../../AccordionContext';
import { StyledAccordionContent } from '../AccordionContent';
import {
  AccordionHeaderItem,
  StyledAccordionHeaderItem,
} from './components/accordion-header-item/AccordionHeaderItem';
import { AccordionHeaderTrigger } from './components/accordion-header-trigger';

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

function AccordionHeader({ children, ...props }: AccordionHeaderProps) {
  const { headerRef, disabled } = useAccordionContext();
  return (
    <StyledAccordionHeader ref={headerRef} disabled={disabled} {...props}>
      {children}
    </StyledAccordionHeader>
  );
}

const AccordionHeaderNamespace = Object.assign(AccordionHeader, {
  AccordionTrigger: AccordionHeaderTrigger,
  Item: AccordionHeaderItem,
  TrailingActions: ListItem.TrailingActions,
  ItemTrigger: ListItem.Trigger,
});

export { AccordionHeaderNamespace as AccordionHeader };

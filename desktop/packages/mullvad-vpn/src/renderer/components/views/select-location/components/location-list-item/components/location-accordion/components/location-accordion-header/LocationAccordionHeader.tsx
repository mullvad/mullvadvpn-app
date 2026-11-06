import styled from 'styled-components';

import { Accordion } from '../../../../../../../../../lib/components/accordion';
import {
  type AccordionHeaderProps,
  StyledAccordionContent,
  StyledAccordionHeader,
} from '../../../../../../../../../lib/components/accordion/components';
import { StyledAccordionHeaderItem } from '../../../../../../../../../lib/components/accordion/components/accordion-header/components';
import {
  StyledListItemTrailingActions,
  StyledListItemTrigger,
} from '../../../../../../../../../lib/components/list-item/components';
import { StyledListItemItemActionGroup } from '../../../../../../../../../lib/components/list-item/components/list-item-item/components';
import { StyledListItemTrailingAction } from '../../../../../../../../../lib/components/list-item/components/list-item-trailing-actions/components';
import {
  LocationAccordionHeaderItem,
  LocationAccordionTrailingActions,
  LocationAccordionTrigger,
} from './components';

export type LocationAccordionHeaderProps = AccordionHeaderProps;

export const LocationAccordionHeaderRoot = styled(Accordion.Header)``;

export const StyledLocationAccordionHeader = styled(LocationAccordionHeaderRoot)`
  // Remove the top border radius of all nested list items
  + ${StyledAccordionContent} {
    ${StyledAccordionHeader} {
      margin-top: 1px;
      ${StyledAccordionHeaderItem}, ${StyledListItemTrailingAction} {
        border-top-left-radius: 0;
        border-top-right-radius: 0;
      }
    }
  }

  // If followed by a list item
  &:has(~ ${LocationAccordionHeaderRoot}) {
    // Remove the bottom border radius of last list item
    + ${StyledAccordionContent} {
      ${StyledAccordionHeader}:last-child {
        ${StyledAccordionHeaderItem}, ${StyledListItemTrailingAction} {
          border-bottom-left-radius: 0;
          border-bottom-right-radius: 0;
        }
      }
    }
  }

  // If the list item has both an item action group and trailing actions
  // and the trailing actions have more than one child, it means it has a
  // child other than the expand/collapse trigger. Then make the first column
  // not take up default 48px, since this will be done by the other trailing action.
  &:has(${StyledListItemItemActionGroup}):has(${StyledListItemTrailingActions} > :nth-child(2))
    ${StyledListItemItemActionGroup} {
    grid-template-columns: auto;
  }

  // If it only has one child, we need to check that it is not the expand/collapse trigger
  &:has(${StyledListItemItemActionGroup}):has(${StyledListItemTrailingActions} > :nth-child(1)):not(
      :has(${StyledListItemTrailingActions} > ${StyledListItemTrigger})
    )
    ${StyledListItemItemActionGroup} {
    grid-template-columns: auto;
  }
`;

function LocationAccordionHeader({ children, ...props }: LocationAccordionHeaderProps) {
  return <StyledLocationAccordionHeader {...props}>{children}</StyledLocationAccordionHeader>;
}

const LocationAccordionHeaderNamespace = Object.assign(LocationAccordionHeader, {
  Item: LocationAccordionHeaderItem,
  TrailingActions: LocationAccordionTrailingActions,
  AccordionTrigger: LocationAccordionTrigger,
  ItemTrigger: Accordion.Header.ItemTrigger,
});

export { LocationAccordionHeaderNamespace as LocationAccordionHeader };

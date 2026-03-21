import styled from 'styled-components';

import { Accordion } from '../../../../../../../../../lib/components/accordion';
import {
  type AccordionHeaderProps,
  StyledAccordionContent,
  StyledAccordionHeader,
} from '../../../../../../../../../lib/components/accordion/components';
import { StyledAccordionHeaderItem } from '../../../../../../../../../lib/components/accordion/components/accordion-header/components';
import { StyledListItemTrailingAction } from '../../../../../../../../../lib/components/list-item/components/list-item-trailing-actions/components';
import { LocationAccordionHeaderItem } from './components';

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
`;

function LocationAccordionHeader({ children, ...props }: LocationAccordionHeaderProps) {
  return <StyledLocationAccordionHeader {...props}>{children}</StyledLocationAccordionHeader>;
}

const LocationAccordionHeaderNamespace = Object.assign(LocationAccordionHeader, {
  Item: LocationAccordionHeaderItem,
  TrailingActions: Accordion.Header.TrailingActions,
  Trigger: Accordion.Header.Trigger,
});

export { LocationAccordionHeaderNamespace as LocationAccordionHeader };

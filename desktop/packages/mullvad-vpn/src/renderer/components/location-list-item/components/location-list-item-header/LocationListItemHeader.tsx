import styled from 'styled-components';

import { Accordion } from '../../../../lib/components/accordion';
import {
  type AccordionHeaderProps,
  StyledAccordionContent,
  StyledAccordionHeader,
  StyledAccordionHeaderItem,
} from '../../../../lib/components/accordion/components';
import { StyledListItemTrailingAction } from '../../../../lib/components/list-item/components';

export type LocationListItemHeaderProps = AccordionHeaderProps;

export const LocationListItemHeaderRoot = styled(Accordion.Header)``;

export const StyledLocationListItemHeader = styled(LocationListItemHeaderRoot)`
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
  &:has(~ ${LocationListItemHeaderRoot}) {
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

export function LocationListItemHeader({ children, ...props }: LocationListItemHeaderProps) {
  return <StyledLocationListItemHeader {...props}>{children}</StyledLocationListItemHeader>;
}

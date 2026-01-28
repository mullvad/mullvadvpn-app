import styled from 'styled-components';

import { ListItem, ListItemProps } from '../../../list-item';
import { StyledListItemTrailingAction } from '../../../list-item/components';
import { useAccordionContext } from '../../AccordionContext';
import { StyledAccordionContent } from '../AccordionContent';
import { StyledAccordionHeaderItem } from '../AccordionHeaderItem';

export type AccordionHeaderProps = ListItemProps;

export const StyledAccordionHeader = styled(ListItem)`
  ${StyledAccordionHeaderItem}, ${StyledListItemTrailingAction} {
    transition: border-radius 0.15s ease-out;
  }
  &:has(+ ${StyledAccordionContent}) {
    ${StyledAccordionHeaderItem} {
      margin-bottom: 1px;
      border-bottom-left-radius: 0;
      border-bottom-right-radius: 0;
    }
    ${StyledListItemTrailingAction} {
      margin-bottom: 1px;
      border-bottom-right-radius: 0;
    }
  }
`;

export function AccordionHeader({ children, ...props }: AccordionHeaderProps) {
  const { disabled } = useAccordionContext();
  return (
    <StyledAccordionHeader disabled={disabled} {...props}>
      {children}
    </StyledAccordionHeader>
  );
}

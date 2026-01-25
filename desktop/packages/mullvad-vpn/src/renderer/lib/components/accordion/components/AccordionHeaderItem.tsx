import styled from 'styled-components';

import { ListItem } from '../../list-item';
import { ListItemItemProps } from '../../list-item/components';
import { useAccordionContext } from '../AccordionContext';

export type AccordionHeaderItemProps = ListItemItemProps;

export const StyledAccordionHeaderItem = styled(ListItem.Item)`
  transition: border-radius 0.15s ease-out;
  &[data-expanded='true'] {
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
  }
`;

export function AccordionHeaderItem({ children, ...props }: AccordionHeaderItemProps) {
  const { expanded } = useAccordionContext();
  return (
    <StyledAccordionHeaderItem data-expanded={expanded} {...props}>
      {children}
    </StyledAccordionHeaderItem>
  );
}

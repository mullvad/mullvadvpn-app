import styled from 'styled-components';

import { ListItem } from '../../list-item';
import { ListItemItemProps } from '../../list-item/components';

export type AccordionHeaderItemProps = ListItemItemProps;

export const StyledAccordionHeaderItem = styled(ListItem.Item)``;

export function AccordionHeaderItem({ children, ...props }: AccordionHeaderItemProps) {
  return <StyledAccordionHeaderItem {...props}>{children}</StyledAccordionHeaderItem>;
}

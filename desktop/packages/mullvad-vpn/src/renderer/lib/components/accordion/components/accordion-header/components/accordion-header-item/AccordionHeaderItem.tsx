import styled from 'styled-components';

import { ListItem } from '../../../../../list-item';
import { ListItemItemProps } from '../../../../../list-item/components';
import { AccordionHeaderItemChevron, AccordionHeaderItemTitle } from './components';

export type AccordionHeaderItemProps = ListItemItemProps;

export const StyledAccordionHeaderItem = styled(ListItem.Item)``;

function AccordionHeaderItem({ children, ...props }: AccordionHeaderItemProps) {
  return <StyledAccordionHeaderItem {...props}>{children}</StyledAccordionHeaderItem>;
}

const AccordionHeaderItemNamespace = Object.assign(AccordionHeaderItem, {
  Title: AccordionHeaderItemTitle,
  Chevron: AccordionHeaderItemChevron,
  ActionGroup: ListItem.Item.ActionGroup,
});

export { AccordionHeaderItemNamespace as AccordionHeaderItem };

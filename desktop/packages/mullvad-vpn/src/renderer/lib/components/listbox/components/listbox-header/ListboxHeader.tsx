import styled from 'styled-components';

import { ListItem, ListItemProps } from '../../../list-item';
import { StyledListItemItem } from '../../../list-item/components';
import { ListboxHeaderItem } from './components';

export type ListboxHeaderProps = ListItemProps;

export const StyledListboxHeader = styled(ListItem)`
  ${StyledListItemItem} {
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
  }
`;

function ListboxHeader({ children, ...props }: ListboxHeaderProps) {
  return <StyledListboxHeader {...props}>{children}</StyledListboxHeader>;
}

export const ListboxHeaderNamespace = Object.assign(ListboxHeader, {
  Item: ListboxHeaderItem,
});

export { ListboxHeaderNamespace as ListboxHeader };

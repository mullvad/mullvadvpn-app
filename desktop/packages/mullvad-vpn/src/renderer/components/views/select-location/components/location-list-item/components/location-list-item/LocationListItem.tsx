import styled from 'styled-components';

import { ListItem, type ListItemProps } from '../../../../../../../lib/components/list-item';
import { StyledListItemTrailingActions } from '../../../../../../../lib/components/list-item/components';
import { StyledListItemItemActionGroup } from '../../../../../../../lib/components/list-item/components/list-item-item/components';
import { LocationListItemItem } from './components';

export type LocationListItemProps = ListItemProps;

export const StyledLocationListItem = styled(ListItem)`
  // If the list item has both an item action group and trailing actions
  // make first column not take up default 48px, since this will be done by
  // the trailing actions.
  &:has(${StyledListItemItemActionGroup}):has(${StyledListItemTrailingActions})
    ${StyledListItemItemActionGroup} {
    grid-template-columns: auto;
  }
`;

function LocationListItem({ children, ...props }: LocationListItemProps) {
  return <StyledLocationListItem {...props}>{children}</StyledLocationListItem>;
}

const LocationListItemNamespace = Object.assign(LocationListItem, {
  Item: LocationListItemItem,
  Footer: ListItem.Footer,
  Trigger: ListItem.Trigger,
  TrailingActions: ListItem.TrailingActions,
});

export { LocationListItemNamespace as LocationListItem };

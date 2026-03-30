import { ListItem, type ListItemProps } from '../../../../../../../lib/components/list-item';
import { LocationListItemItem } from './components';

export type LocationListItemProps = ListItemProps;

function LocationListItem({ children, ...props }: LocationListItemProps) {
  return <ListItem {...props}>{children}</ListItem>;
}

const LocationListItemNamespace = Object.assign(LocationListItem, {
  Item: LocationListItemItem,
  Footer: ListItem.Footer,
  Trigger: ListItem.Trigger,
  TrailingActions: ListItem.TrailingActions,
});

export { LocationListItemNamespace as LocationListItem };

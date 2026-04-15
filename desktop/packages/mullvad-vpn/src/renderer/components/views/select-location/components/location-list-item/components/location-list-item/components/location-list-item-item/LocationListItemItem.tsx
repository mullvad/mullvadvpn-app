import { ListItem } from '../../../../../../../../../lib/components/list-item';
import type { ListItemItemProps } from '../../../../../../../../../lib/components/list-item/components';
import { LocationListItemItemLabel } from './components';

export type LocationListItemItemProps = ListItemItemProps;

function LocationListItemItem({ children, ...props }: LocationListItemItemProps) {
  return <ListItem.Item {...props}>{children}</ListItem.Item>;
}

const LocationListItemItemNamespace = Object.assign(LocationListItemItem, {
  Label: LocationListItemItemLabel,
});

export { LocationListItemItemNamespace as LocationListItemItem };

import { ListItem } from '../../../../../../../../../lib/components/list-item';
import type { ListItemItemProps } from '../../../../../../../../../lib/components/list-item/components';
import { LocationIcon } from '../../../location-icon';
import { LocationListItemItemLabel } from './components';

export type LocationListItemItemProps = ListItemItemProps;

function LocationListItemItem({ children, ...props }: LocationListItemItemProps) {
  return <ListItem.Item {...props}>{children}</ListItem.Item>;
}

const LocationListItemItemNamespace = Object.assign(LocationListItemItem, {
  Label: LocationListItemItemLabel,
  Icon: LocationIcon,
  ActionGroup: ListItem.Item.ActionGroup,
});

export { LocationListItemItemNamespace as LocationListItemItem };

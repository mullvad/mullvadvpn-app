import { ListItem } from '../../../../../list-item';
import type { ListItemItemProps } from '../../../../../list-item/components';
import { ListboxHeaderItemLabel } from './components';

export type ListboxHeaderItemProps = ListItemItemProps;

function ListboxHeaderItem({ children, ...props }: ListboxHeaderItemProps) {
  return <ListItem.Item {...props}>{children}</ListItem.Item>;
}

const ListboxHeaderItemNamespace = Object.assign(ListboxHeaderItem, {
  Label: ListboxHeaderItemLabel,
  Group: ListItem.Item.Group,
  ActionGroup: ListItem.Item.ActionGroup,
  Text: ListItem.Item.Text,
  Icon: ListItem.Item.Icon,
});

export { ListboxHeaderItemNamespace as ListboxHeaderItem };

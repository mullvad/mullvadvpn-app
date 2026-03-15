import { ListItem } from '../../../../lib/components/list-item';
import type { ListItemItemGroupProps } from '../../../../lib/components/list-item/components/list-item-item/components';

export type SettingsListItemGroupProps = ListItemItemGroupProps;

export function SettingsListItemGroup(props: SettingsListItemGroupProps) {
  return <ListItem.Item.Group {...props} />;
}

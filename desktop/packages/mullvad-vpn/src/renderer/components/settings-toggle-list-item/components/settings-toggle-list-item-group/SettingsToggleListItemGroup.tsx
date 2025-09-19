import { ListItem } from '../../../../lib/components/list-item';
import { ListItemGroupProps } from '../../../../lib/components/list-item/components';

export type SettingsToggleListItemGroup = ListItemGroupProps;

export function SettingsToggleListItemGroup(props: SettingsToggleListItemGroup) {
  return <ListItem.Group $gap="medium" {...props} />;
}

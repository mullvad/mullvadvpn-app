import { ListItem } from '../../../../lib/components/list-item';
import { ListItemGroupProps } from '../../../../lib/components/list-item/components';

export type SettingsListItemGroupProps = ListItemGroupProps;

export function SettingsListItemGroup(props: SettingsListItemGroupProps) {
  return <ListItem.Group $gap="medium" {...props} />;
}

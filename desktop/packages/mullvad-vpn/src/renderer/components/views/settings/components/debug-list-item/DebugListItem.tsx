import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type DebugListItemProps = Omit<ListItemProps, 'children'>;

export function DebugListItem(props: DebugListItemProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.debug} {...props}>
      <SettingsNavigationListItem.Label>Developer tools</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.ActionGroup>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.ActionGroup>
    </SettingsNavigationListItem>
  );
}

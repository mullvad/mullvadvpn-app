import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type KillSwitchListItemProps = Omit<ListItemProps, 'children'>;

export function KillSwitchListItem(props: KillSwitchListItemProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.killSwitch} {...props}>
      <SettingsNavigationListItem.Item>
        <SettingsNavigationListItem.Item.Label>
          {messages.gettext('Kill switch')}
        </SettingsNavigationListItem.Item.Label>
        <SettingsNavigationListItem.Item.ActionGroup>
          <SettingsNavigationListItem.Item.Icon icon="chevron-right" />
        </SettingsNavigationListItem.Item.ActionGroup>
      </SettingsNavigationListItem.Item>
    </SettingsNavigationListItem>
  );
}

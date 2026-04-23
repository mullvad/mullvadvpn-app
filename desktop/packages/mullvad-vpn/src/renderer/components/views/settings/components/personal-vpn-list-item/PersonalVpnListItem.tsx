import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { useSettingsCustomVpn } from '../../../../../redux/hooks';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type PersonalVpnListItemProps = Omit<ListItemProps, 'children'>;

export function PersonalVpnListItem(props: PersonalVpnListItemProps) {
  const { customVpnEnabled } = useSettingsCustomVpn();

  return (
    <SettingsNavigationListItem to={RoutePath.personalVpn} {...props}>
      <SettingsNavigationListItem.Item>
        <SettingsNavigationListItem.Item.Label>
          {messages.pgettext('settings-view', 'Personal VPN')}
        </SettingsNavigationListItem.Item.Label>
        <SettingsNavigationListItem.Item.ActionGroup>
          <SettingsNavigationListItem.Item.Text>
            {customVpnEnabled ? messages.gettext('On') : messages.gettext('Off')}
          </SettingsNavigationListItem.Item.Text>
          <SettingsNavigationListItem.Item.Icon icon="chevron-right" />
        </SettingsNavigationListItem.Item.ActionGroup>
      </SettingsNavigationListItem.Item>
    </SettingsNavigationListItem>
  );
}

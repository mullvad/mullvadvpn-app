import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../SettingsNavigationListItem';

export function VpnSettingsListItem() {
  return (
    <SettingsNavigationListItem to={RoutePath.vpnSettings}>
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'VPN settings' view
          messages.pgettext('settings-view', 'VPN settings')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

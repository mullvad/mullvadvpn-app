import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export function IpOverrideSettings() {
  return (
    <SettingsNavigationListItem to={RoutePath.settingsImport}>
      <SettingsNavigationListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Server IP override')}
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

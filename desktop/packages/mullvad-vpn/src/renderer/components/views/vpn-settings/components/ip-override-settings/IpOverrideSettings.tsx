import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { NavigationListItem } from '../../../../NavigationListItem';

export function IpOverrideSettings() {
  return (
    <NavigationListItem to={RoutePath.settingsImport}>
      <NavigationListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Server IP override')}
      </NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

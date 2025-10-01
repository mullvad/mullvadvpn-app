import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export function WireguardSettings() {
  return (
    <SettingsNavigationListItem to={RoutePath.wireguardSettings}>
      <SettingsNavigationListItem.Label>
        {sprintf(
          // TRANSLATORS: %(wireguard)s will be replaced with the string "WireGuard"
          messages.pgettext('vpn-settings-view', '%(wireguard)s settings'),
          { wireguard: strings.wireguard },
        )}
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

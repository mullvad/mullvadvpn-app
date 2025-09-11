import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useTunnelProtocol } from '../../../../../lib/relay-settings-hooks';
import { SettingsNavigationListItem } from '../../../../SettingsNavigationListItem';

export function OpenVpnSettings() {
  const tunnelProtocol = useTunnelProtocol();

  return (
    <SettingsNavigationListItem
      to={RoutePath.openVpnSettings}
      disabled={tunnelProtocol === 'wireguard'}>
      <SettingsNavigationListItem.Label>
        {sprintf(
          // TRANSLATORS: %(openvpn)s will be replaced with the string "OpenVPN"
          messages.pgettext('vpn-settings-view', '%(openvpn)s settings'),
          { openvpn: strings.openvpn },
        )}
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useTunnelProtocol } from '../../../../../lib/relay-settings-hooks';
import { NavigationListItem } from '../../../../NavigationListItem';

export function OpenVpnSettingsButton() {
  const tunnelProtocol = useTunnelProtocol();

  return (
    <NavigationListItem to={RoutePath.openVpnSettings} disabled={tunnelProtocol === 'wireguard'}>
      <NavigationListItem.Label>
        {sprintf(
          // TRANSLATORS: %(openvpn)s will be replaced with the string "OpenVPN"
          messages.pgettext('vpn-settings-view', '%(openvpn)s settings'),
          { openvpn: strings.openvpn },
        )}
      </NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useScrollToListItem } from '../../../../../hooks';
import { useTunnelProtocol } from '../../../../../lib/relay-settings-hooks';
import { NavigationListItem } from '../../../../NavigationListItem';

export function OpenVpnSettings() {
  const tunnelProtocol = useTunnelProtocol();
  const scrollToAnchor = useScrollToListItem();

  return (
    <NavigationListItem
      to={RoutePath.openVpnSettings}
      disabled={tunnelProtocol === 'wireguard'}
      animation={scrollToAnchor?.animation}>
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

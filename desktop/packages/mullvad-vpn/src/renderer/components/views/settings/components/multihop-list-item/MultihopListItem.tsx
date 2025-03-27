import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { RoutePath } from '../../../../../lib/routes';
import { useSelector } from '../../../../../redux/store';
import { NavigationListItem } from '../../../../NavigationListItem';

export function MultihopListItem() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  return (
    <NavigationListItem to={RoutePath.multihopSettings}>
      <ListItem.Label>{messages.pgettext('settings-view', 'Multihop')}</ListItem.Label>
      <ListItem.Group>
        <ListItem.Text>
          {multihop && !unavailable ? messages.gettext('On') : messages.gettext('Off')}
        </ListItem.Text>
        <Icon icon="chevron-right" />
      </ListItem.Group>
    </NavigationListItem>
  );
}

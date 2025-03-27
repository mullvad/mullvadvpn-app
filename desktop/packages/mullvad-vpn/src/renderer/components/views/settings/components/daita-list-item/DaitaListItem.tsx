import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { RoutePath } from '../../../../../lib/routes';
import { useSelector } from '../../../../../redux/store';
import { NavigationListItem } from '../../../../NavigationListItem';

export function DaitaListItem() {
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  return (
    <NavigationListItem to={RoutePath.daitaSettings}>
      <ListItem.Label>{strings.daita}</ListItem.Label>
      <ListItem.Group>
        <ListItem.Text>
          {daita && !unavailable ? messages.gettext('On') : messages.gettext('Off')}
        </ListItem.Text>
        <Icon icon="chevron-right" />
      </ListItem.Group>
    </NavigationListItem>
  );
}

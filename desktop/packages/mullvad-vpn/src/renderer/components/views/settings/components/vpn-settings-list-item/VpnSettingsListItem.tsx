import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { RoutePath } from '../../../../../lib/routes';
import { NavigationListItem } from '../../../../NavigationListItem';

export function VpnSettingsListItem() {
  return (
    <NavigationListItem to={RoutePath.vpnSettings}>
      <ListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'VPN settings' view
          messages.pgettext('settings-view', 'VPN settings')
        }
      </ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

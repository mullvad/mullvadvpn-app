import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { NavigationListItem } from '../../../../NavigationListItem';

export function UserInterfaceSettingsListItem() {
  return (
    <NavigationListItem to={RoutePath.userInterfaceSettings}>
      <ListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'User interface settings' view
          messages.pgettext('settings-view', 'User interface settings')
        }
      </ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

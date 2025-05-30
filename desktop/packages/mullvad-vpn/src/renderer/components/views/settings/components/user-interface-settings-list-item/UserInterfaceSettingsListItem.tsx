import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { NavigationListItem } from '../../../../NavigationListItem';

export function UserInterfaceSettingsListItem() {
  return (
    <NavigationListItem to={RoutePath.userInterfaceSettings}>
      <NavigationListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'User interface settings' view
          messages.pgettext('settings-view', 'User interface settings')
        }
      </NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

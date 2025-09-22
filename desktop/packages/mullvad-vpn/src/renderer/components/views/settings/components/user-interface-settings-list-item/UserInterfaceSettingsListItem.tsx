import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export function UserInterfaceSettingsListItem() {
  return (
    <SettingsNavigationListItem to={RoutePath.userInterfaceSettings}>
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'User interface settings' view
          messages.pgettext('settings-view', 'User interface settings')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

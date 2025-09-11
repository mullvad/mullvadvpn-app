import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../SettingsNavigationListItem';

export function ApiAccessMethodsListItem() {
  return (
    <SettingsNavigationListItem to={RoutePath.apiAccessMethods}>
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'API access methods' view
          messages.pgettext('settings-view', 'API access')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

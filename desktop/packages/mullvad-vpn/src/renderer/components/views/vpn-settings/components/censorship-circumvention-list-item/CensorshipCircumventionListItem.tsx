import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export function CensorshipCircumventionListItem() {
  return (
    <SettingsNavigationListItem to={RoutePath.censorshipCircumvention}>
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Label for list item that navigates to censorship
          // TRANSLATORS: circumvention settings view.
          messages.pgettext('vpn-settings-view', 'Censorship circumvention')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useSelector } from '../../../../../redux/store';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';
import { getObfuscationLabel } from './utils';

export function AntiCensorshipListItem() {
  const { selectedObfuscation } = useSelector((state) => state.settings.obfuscationSettings);
  const obfuscationLabel = getObfuscationLabel(selectedObfuscation);

  return (
    <SettingsNavigationListItem to={RoutePath.antiCensorship}>
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Label for list item that navigates to anti-censorship
          // TRANSLATORS: settings view.
          messages.pgettext('vpn-settings-view', 'Anti-censorship')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Text>{obfuscationLabel}</SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.Group>
    </SettingsNavigationListItem>
  );
}

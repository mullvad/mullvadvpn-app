import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useSelector } from '../../../../../redux/store';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';
import { getObfuscationLabel } from './utils';

export function ObfuscationListItem() {
  const { selectedObfuscation } = useSelector((state) => state.settings.obfuscationSettings);
  const obfuscationLabel = getObfuscationLabel(selectedObfuscation);

  return (
    <SettingsNavigationListItem to={RoutePath.obfuscationSettings}>
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Label for list item that navigates to obfuscation settings view
          messages.pgettext('vpn-settings-view', 'Obfuscation')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Text>{obfuscationLabel}</SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.Group>
    </SettingsNavigationListItem>
  );
}

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useSelector } from '../../../../../redux/store';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';
import { getObfuscationLabel } from './utils';

export function CensorshipCircumventionListItem() {
  const { selectedObfuscation } = useSelector((state) => state.settings.obfuscationSettings);
  const obfuscationLabel = getObfuscationLabel(selectedObfuscation);

  return (
    <SettingsNavigationListItem to={RoutePath.censorshipCircumvention}>
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Label for list item that navigates to censorship
          // TRANSLATORS: circumvention settings view.
          messages.pgettext('vpn-settings-view', 'Censorship circumvention')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Text>{obfuscationLabel}</SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.Group>
    </SettingsNavigationListItem>
  );
}

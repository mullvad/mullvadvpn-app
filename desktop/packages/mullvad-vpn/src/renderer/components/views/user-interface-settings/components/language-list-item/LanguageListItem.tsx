import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useAppContext } from '../../../../../context';
import { Image } from '../../../../../lib/components';
import { useSelector } from '../../../../../redux/store';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export function LanguageListItem() {
  const { getPreferredLocaleDisplayName } = useAppContext();
  const preferredLocale = useSelector((state) => state.settings.guiSettings.preferredLocale);
  const localeDisplayName = getPreferredLocaleDisplayName(preferredLocale);

  return (
    <SettingsNavigationListItem to={RoutePath.selectLanguage}>
      <SettingsNavigationListItem.Group>
        <Image source="icon-language" />
        <SettingsNavigationListItem.Label>
          {
            // TRANSLATORS: Navigation button to the 'Language' settings view
            messages.pgettext('user-interface-settings-view', 'Language')
          }
        </SettingsNavigationListItem.Label>
      </SettingsNavigationListItem.Group>
      <SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Text>{localeDisplayName}</SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.Group>
    </SettingsNavigationListItem>
  );
}

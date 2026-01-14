import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useAppContext } from '../../../../../context';
import { Image } from '../../../../../lib/components';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { useSelector } from '../../../../../redux/store';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type LanguageListItemProps = Omit<ListItemProps, 'children'>;

export function LanguageListItem(props: LanguageListItemProps) {
  const { getPreferredLocaleDisplayName } = useAppContext();
  const preferredLocale = useSelector((state) => state.settings.guiSettings.preferredLocale);
  const localeDisplayName = getPreferredLocaleDisplayName(preferredLocale);

  return (
    <SettingsNavigationListItem to={RoutePath.selectLanguage} {...props}>
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

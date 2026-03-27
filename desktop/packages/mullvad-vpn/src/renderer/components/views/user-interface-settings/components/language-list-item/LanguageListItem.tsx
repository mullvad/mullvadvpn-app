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
      <SettingsNavigationListItem.Item>
        <SettingsNavigationListItem.Item.Group gap="small">
          <Image source="icon-language" />
          <SettingsNavigationListItem.Item.Label>
            {
              // TRANSLATORS: Navigation button to the 'Language' settings view
              messages.pgettext('user-interface-settings-view', 'Language')
            }
          </SettingsNavigationListItem.Item.Label>
        </SettingsNavigationListItem.Item.Group>
        <SettingsNavigationListItem.Item.ActionGroup>
          <SettingsNavigationListItem.Item.Text>
            {localeDisplayName}
          </SettingsNavigationListItem.Item.Text>
          <SettingsNavigationListItem.Item.Icon icon="chevron-right" />
        </SettingsNavigationListItem.Item.ActionGroup>
      </SettingsNavigationListItem.Item>
    </SettingsNavigationListItem>
  );
}

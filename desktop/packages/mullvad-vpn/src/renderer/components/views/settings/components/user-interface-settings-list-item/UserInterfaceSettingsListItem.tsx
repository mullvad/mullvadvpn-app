import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type UserInterfaceSettingsListItemProps = Omit<ListItemProps, 'children'>;

export function UserInterfaceSettingsListItem(props: UserInterfaceSettingsListItemProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.userInterfaceSettings} {...props}>
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

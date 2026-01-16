import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type ApiAccessMethodsListItemProps = Omit<ListItemProps, 'children'>;

export function ApiAccessMethodsListItem(props: ApiAccessMethodsListItemProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.apiAccessMethods} {...props}>
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

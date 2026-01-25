import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type SupportListItemProps = Omit<ListItemProps, 'children'>;

export function SupportListItem(props: SupportListItemProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.support} {...props}>
      <SettingsNavigationListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'Support' view
          messages.pgettext('settings-view', 'Support')
        }
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.ActionGroup>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.ActionGroup>
    </SettingsNavigationListItem>
  );
}

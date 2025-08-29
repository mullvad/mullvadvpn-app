import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { NavigationListItem } from '../../../../NavigationListItem';

export function SupportListItem() {
  return (
    <NavigationListItem to={RoutePath.support}>
      <NavigationListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'Support' view
          messages.pgettext('settings-view', 'Support')
        }
      </NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

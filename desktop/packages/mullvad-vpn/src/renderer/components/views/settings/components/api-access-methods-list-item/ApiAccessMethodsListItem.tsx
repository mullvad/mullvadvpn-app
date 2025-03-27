import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { RoutePath } from '../../../../../lib/routes';
import { NavigationListItem } from '../../../../NavigationListItem';

export function ApiAccessMethodsListItem() {
  return (
    <NavigationListItem to={RoutePath.apiAccessMethods}>
      <ListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'API access methods' view
          messages.pgettext('settings-view', 'API access')
        }
      </ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

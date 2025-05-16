import { messages } from '../../../../../../shared/gettext';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { RoutePath } from '../../../../../lib/routes';
import { NavigationListItem } from '../../../../NavigationListItem';

export function SupportListItem() {
  return (
    <NavigationListItem to={RoutePath.support}>
      <ListItem.Label>
        {
          // TRANSLATORS: Navigation button to the 'Support' view
          messages.pgettext('settings-view', 'Support')
        }
      </ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

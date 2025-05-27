import { RoutePath } from '../../../../../../shared/routes';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { NavigationListItem } from '../../../../NavigationListItem';

export function DebugListItem() {
  return (
    <NavigationListItem to={RoutePath.debug}>
      <ListItem.Label>Developer tools</ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

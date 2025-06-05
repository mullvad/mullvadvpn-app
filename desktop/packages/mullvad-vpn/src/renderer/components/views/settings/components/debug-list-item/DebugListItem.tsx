import { RoutePath } from '../../../../../../shared/routes';
import { NavigationListItem } from '../../../../NavigationListItem';

export function DebugListItem() {
  return (
    <NavigationListItem to={RoutePath.debug}>
      <NavigationListItem.Label>Developer tools</NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

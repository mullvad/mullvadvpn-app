import { strings } from '../../../../../../shared/constants';
import { RoutePath } from '../../../../../../shared/routes';
import { NavigationListItem } from '../../../../NavigationListItem';

export function SplitTunnelingListItem() {
  return (
    <NavigationListItem to={RoutePath.splitTunneling}>
      <NavigationListItem.Label>{strings.splitTunneling}</NavigationListItem.Label>
      <NavigationListItem.Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

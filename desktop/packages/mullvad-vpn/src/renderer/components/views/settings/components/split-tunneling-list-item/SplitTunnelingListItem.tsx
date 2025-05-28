import { strings } from '../../../../../../shared/constants';
import { RoutePath } from '../../../../../../shared/routes';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { NavigationListItem } from '../../../../NavigationListItem';

export function SplitTunnelingListItem() {
  return (
    <NavigationListItem to={RoutePath.splitTunneling}>
      <ListItem.Label>{strings.splitTunneling}</ListItem.Label>
      <Icon icon="chevron-right" />
    </NavigationListItem>
  );
}

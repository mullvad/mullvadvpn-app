import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { NavigationListItem } from '../../../../NavigationListItem';
import { useIsOn } from './hooks';

export function MultihopListItem() {
  const isOn = useIsOn();

  return (
    <NavigationListItem to={RoutePath.multihopSettings}>
      <ListItem.Label>{messages.pgettext('settings-view', 'Multihop')}</ListItem.Label>
      <ListItem.Group>
        <ListItem.Text>{isOn ? messages.gettext('On') : messages.gettext('Off')}</ListItem.Text>
        <Icon icon="chevron-right" />
      </ListItem.Group>
    </NavigationListItem>
  );
}

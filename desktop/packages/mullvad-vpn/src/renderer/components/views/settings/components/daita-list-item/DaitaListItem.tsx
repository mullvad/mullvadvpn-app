import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Icon } from '../../../../../lib/components';
import { ListItem } from '../../../../../lib/components/list-item';
import { NavigationListItem } from '../../../../NavigationListItem';
import { useIsOn } from './hooks';

export function DaitaListItem() {
  const isOn = useIsOn();

  return (
    <NavigationListItem to={RoutePath.daitaSettings}>
      <ListItem.Label>{strings.daita}</ListItem.Label>
      <ListItem.Group>
        <ListItem.Text>{isOn ? messages.gettext('On') : messages.gettext('Off')}</ListItem.Text>
        <Icon icon="chevron-right" />
      </ListItem.Group>
    </NavigationListItem>
  );
}

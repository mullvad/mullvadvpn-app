import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { NavigationListItem } from '../../../../NavigationListItem';
import { useIsOn } from './hooks';

export function DaitaListItem() {
  const isOn = useIsOn();

  return (
    <NavigationListItem to={RoutePath.daitaSettings}>
      <NavigationListItem.Label>{strings.daita}</NavigationListItem.Label>
      <NavigationListItem.Group>
        <NavigationListItem.Text>
          {isOn ? messages.gettext('On') : messages.gettext('Off')}
        </NavigationListItem.Text>
        <NavigationListItem.Icon icon="chevron-right" />
      </NavigationListItem.Group>
    </NavigationListItem>
  );
}

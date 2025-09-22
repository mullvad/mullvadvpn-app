import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { Icon } from '../../../../../lib/components';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';
import { useIsOn } from './hooks';

export function MultihopListItem() {
  const isOn = useIsOn();

  return (
    <SettingsNavigationListItem to={RoutePath.multihopSettings}>
      <SettingsNavigationListItem.Label>
        {messages.pgettext('settings-view', 'Multihop')}
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Text>
          {isOn ? messages.gettext('On') : messages.gettext('Off')}
        </SettingsNavigationListItem.Text>
        <Icon icon="chevron-right" />
      </SettingsNavigationListItem.Group>
    </SettingsNavigationListItem>
  );
}

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../SettingsNavigationListItem';
import { useIsOn } from './hooks';

export function DaitaListItem() {
  const isOn = useIsOn();

  return (
    <SettingsNavigationListItem to={RoutePath.daitaSettings}>
      <SettingsNavigationListItem.Label>{strings.daita}</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Group>
        <SettingsNavigationListItem.Text>
          {isOn ? messages.gettext('On') : messages.gettext('Off')}
        </SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.Group>
    </SettingsNavigationListItem>
  );
}

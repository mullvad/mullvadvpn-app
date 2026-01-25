import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';
import { useIsOn } from './hooks';

export type MultihopListItemProps = Omit<ListItemProps, 'children'>;

export function MultihopListItem(props: MultihopListItemProps) {
  const isOn = useIsOn();

  return (
    <SettingsNavigationListItem to={RoutePath.multihopSettings} {...props}>
      <SettingsNavigationListItem.Label>
        {messages.pgettext('settings-view', 'Multihop')}
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.ActionGroup>
        <SettingsNavigationListItem.Text>
          {isOn ? messages.gettext('On') : messages.gettext('Off')}
        </SettingsNavigationListItem.Text>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.ActionGroup>
    </SettingsNavigationListItem>
  );
}

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
      <SettingsNavigationListItem.Item>
        <SettingsNavigationListItem.Item.Label>
          {messages.pgettext('settings-view', 'Multihop')}
        </SettingsNavigationListItem.Item.Label>
        <SettingsNavigationListItem.Item.ActionGroup>
          <SettingsNavigationListItem.Item.Text>
            {isOn ? messages.gettext('On') : messages.gettext('Off')}
          </SettingsNavigationListItem.Item.Text>
          <SettingsNavigationListItem.Item.Icon icon="chevron-right" />
        </SettingsNavigationListItem.Item.ActionGroup>
      </SettingsNavigationListItem.Item>
    </SettingsNavigationListItem>
  );
}

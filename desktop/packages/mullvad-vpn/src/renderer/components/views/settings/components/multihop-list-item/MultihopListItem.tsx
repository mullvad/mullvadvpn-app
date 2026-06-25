import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';
import { useMessage } from './hooks';

export type MultihopListItemProps = Omit<ListItemProps, 'children'>;

export function MultihopListItem(props: MultihopListItemProps) {
  const message = useMessage();

  return (
    <SettingsNavigationListItem to={RoutePath.multihopSettings} {...props}>
      <SettingsNavigationListItem.Item>
        <SettingsNavigationListItem.Item.Label>
          {messages.pgettext('settings-view', 'Multihop')}
        </SettingsNavigationListItem.Item.Label>
        <SettingsNavigationListItem.Item.ActionGroup>
          <SettingsNavigationListItem.Item.Text>{message}</SettingsNavigationListItem.Item.Text>
          <SettingsNavigationListItem.Item.Icon icon="chevron-right" />
        </SettingsNavigationListItem.Item.ActionGroup>
      </SettingsNavigationListItem.Item>
    </SettingsNavigationListItem>
  );
}

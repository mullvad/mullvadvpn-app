import { strings } from '../../../../../../shared/constants';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type SplitTunnelingListItemProps = Omit<ListItemProps, 'children'>;

export function SplitTunnelingListItem(props: SplitTunnelingListItemProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.splitTunneling} {...props}>
      <SettingsNavigationListItem.Item>
        <SettingsNavigationListItem.Item.Label>
          {strings.splitTunneling}
        </SettingsNavigationListItem.Item.Label>
        <SettingsNavigationListItem.Item.ActionGroup>
          <SettingsNavigationListItem.Item.Icon icon="chevron-right" />
        </SettingsNavigationListItem.Item.ActionGroup>
      </SettingsNavigationListItem.Item>
    </SettingsNavigationListItem>
  );
}

import { strings } from '../../../../../../shared/constants';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type SplitTunnelingIpListItemProps = Omit<ListItemProps, 'children'>;

export function SplitTunnelingIpListItem(props: SplitTunnelingIpListItemProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.splitTunnelingIp} {...props}>
      <SettingsNavigationListItem.Label>{strings.splitTunnelingIp}</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.ActionGroup>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.ActionGroup>
    </SettingsNavigationListItem>
  );
}

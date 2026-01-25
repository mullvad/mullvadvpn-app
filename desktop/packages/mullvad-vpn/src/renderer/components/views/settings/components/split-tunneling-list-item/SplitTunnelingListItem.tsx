import { strings } from '../../../../../../shared/constants';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type SplitTunnelingListItemProps = Omit<ListItemProps, 'children'>;

export function SplitTunnelingListItem(props: SplitTunnelingListItemProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.splitTunneling} {...props}>
      <SettingsNavigationListItem.Label>{strings.splitTunneling}</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.ActionGroup>
        <SettingsNavigationListItem.Icon icon="chevron-right" />
      </SettingsNavigationListItem.ActionGroup>
    </SettingsNavigationListItem>
  );
}

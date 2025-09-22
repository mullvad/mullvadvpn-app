import { strings } from '../../../../../../shared/constants';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export function SplitTunnelingListItem() {
  return (
    <SettingsNavigationListItem to={RoutePath.splitTunneling}>
      <SettingsNavigationListItem.Label>{strings.splitTunneling}</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

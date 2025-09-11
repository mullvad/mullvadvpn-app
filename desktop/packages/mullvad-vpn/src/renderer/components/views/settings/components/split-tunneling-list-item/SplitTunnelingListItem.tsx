import { strings } from '../../../../../../shared/constants';
import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../SettingsNavigationListItem';

export function SplitTunnelingListItem() {
  return (
    <SettingsNavigationListItem to={RoutePath.splitTunneling}>
      <SettingsNavigationListItem.Label>{strings.splitTunneling}</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

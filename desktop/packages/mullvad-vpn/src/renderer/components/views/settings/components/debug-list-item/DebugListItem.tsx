import { RoutePath } from '../../../../../../shared/routes';
import { SettingsNavigationListItem } from '../../../../SettingsNavigationListItem';

export function DebugListItem() {
  return (
    <SettingsNavigationListItem to={RoutePath.debug}>
      <SettingsNavigationListItem.Label>Developer tools</SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

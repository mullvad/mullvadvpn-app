import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { SettingsNavigationListItem } from '../../../../settings-navigation-list-item';

export type IpOverrideSettingsProps = Omit<ListItemProps, 'children'>;

export function IpOverrideSettings(props: IpOverrideSettingsProps) {
  return (
    <SettingsNavigationListItem to={RoutePath.settingsImport} {...props}>
      <SettingsNavigationListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Server IP override')}
      </SettingsNavigationListItem.Label>
      <SettingsNavigationListItem.Icon icon="chevron-right" />
    </SettingsNavigationListItem>
  );
}

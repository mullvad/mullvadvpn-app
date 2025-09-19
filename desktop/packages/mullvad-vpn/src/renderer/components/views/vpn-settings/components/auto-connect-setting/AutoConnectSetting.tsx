import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function AutoConnectSetting() {
  const autoConnect = useSelector((state) => state.settings.guiSettings.autoConnect);
  const { setAutoConnect } = useAppContext();

  const footer = messages.pgettext(
    'vpn-settings-view',
    'Automatically connect to a server when the app launches.',
  );

  return (
    <SettingsToggleListItem
      checked={autoConnect}
      onCheckedChange={setAutoConnect}
      description={footer}>
      <SettingsToggleListItem.Label>
        {messages.pgettext('vpn-settings-view', 'Auto-connect')}
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}

import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function NotificationsSetting() {
  const enableSystemNotifications = useSelector(
    (state) => state.settings.guiSettings.enableSystemNotifications,
  );
  const { setEnableSystemNotifications } = useAppContext();

  return (
    <SettingsToggleListItem
      checked={enableSystemNotifications}
      onCheckedChange={setEnableSystemNotifications}
      description={messages.pgettext(
        'user-interface-settings-view',
        'Enable or disable system notifications. The critical notifications will always be displayed.',
      )}>
      <SettingsToggleListItem.Label>
        {messages.pgettext('user-interface-settings-view', 'Notifications')}
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}

import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function StartMinimizedSetting() {
  const startMinimized = useSelector((state) => state.settings.guiSettings.startMinimized);
  const { setStartMinimized } = useAppContext();

  return (
    <SettingsToggleListItem
      checked={startMinimized}
      onCheckedChange={setStartMinimized}
      description={messages.pgettext(
        'user-interface-settings-view',
        'Show only the tray icon when the app starts.',
      )}>
      <SettingsToggleListItem.Label>
        {messages.pgettext('user-interface-settings-view', 'Start minimized')}
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}

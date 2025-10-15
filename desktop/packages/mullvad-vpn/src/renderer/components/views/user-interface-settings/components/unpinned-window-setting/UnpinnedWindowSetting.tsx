import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function UnpinnedWindowSetting() {
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);
  const { setUnpinnedWindow } = useAppContext();

  return (
    <SettingsToggleListItem
      checked={unpinnedWindow}
      onCheckedChange={setUnpinnedWindow}
      description={messages.pgettext(
        'user-interface-settings-view',
        'Enable to move the app around as a free-standing window.',
      )}>
      <SettingsToggleListItem.Label>
        {messages.pgettext('user-interface-settings-view', 'Unpin app from taskbar')}
      </SettingsToggleListItem.Label>
      <SettingsToggleListItem.Switch />
    </SettingsToggleListItem>
  );
}

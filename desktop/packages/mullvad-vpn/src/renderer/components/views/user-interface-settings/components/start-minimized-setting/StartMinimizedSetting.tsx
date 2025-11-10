import { messages } from '../../../../../../shared/gettext';
import { SettingsListItem } from '../../../../settings-list-item';
import { StartMinimizedSwitch } from './StartMinimizedSwitch';

export function StartMinimizedSetting() {
  return (
    <SettingsListItem>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <StartMinimizedSwitch>
            <StartMinimizedSwitch.Label variant="titleMedium">
              {messages.pgettext('user-interface-settings-view', 'Start minimized')}
            </StartMinimizedSwitch.Label>
            <StartMinimizedSwitch.Trigger>
              <StartMinimizedSwitch.Thumb />
            </StartMinimizedSwitch.Trigger>
          </StartMinimizedSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Text>
          {messages.pgettext(
            'user-interface-settings-view',
            'Show only the tray icon when the app starts.',
          )}
        </SettingsListItem.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}

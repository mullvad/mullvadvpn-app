import { messages } from '../../../../../../shared/gettext';
import { SettingsListItem } from '../../../../settings-list-item';
import { AutoStartSwitch } from './AutoStartSwitch';

export function AutoStartSetting() {
  return (
    <SettingsListItem>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <AutoStartSwitch>
            <AutoStartSwitch.Label variant="titleMedium">
              {messages.pgettext('vpn-settings-view', 'Launch app on start-up')}
            </AutoStartSwitch.Label>
            <AutoStartSwitch.Trigger>
              <AutoStartSwitch.Thumb />
            </AutoStartSwitch.Trigger>
          </AutoStartSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}

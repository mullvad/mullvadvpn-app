import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { useNormalRelaySettings } from '../../../../lib/relay-settings-hooks';
import { MultihopSwitch } from '../multihop-switch/MultihopSwitch';

export function MultihopSetting() {
  const relaySettings = useNormalRelaySettings();
  const unavailable = relaySettings === null;

  return (
    <SettingsListItem anchorId="multihop-setting" disabled={unavailable}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <MultihopSwitch>
            <MultihopSwitch.Label variant="titleMedium">
              {messages.gettext('Enable')}
            </MultihopSwitch.Label>
            <MultihopSwitch.Trigger>
              <MultihopSwitch.Thumb />
            </MultihopSwitch.Trigger>
          </MultihopSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}

import { messages } from '../../../../../../shared/gettext';
import { useNormalRelaySettings } from '../../../../../lib/relay-settings-hooks';
import { SettingsListItem } from '../../../../settings-list-item';
import { MultihopSwitch } from './MultihopSwitch';

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

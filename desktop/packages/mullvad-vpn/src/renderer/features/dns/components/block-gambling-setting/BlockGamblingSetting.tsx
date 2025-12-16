import { messages } from '../../../../../shared/gettext';
import { ListItem } from '../../../../lib/components/list-item';
import { BlockGamblingSwitch } from '../block-gambling-switch/BlockGamblingSwitch';

export function BlockGamblingSetting() {
  return (
    <ListItem level={1}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockGamblingSwitch>
            <BlockGamblingSwitch.Label variant="bodySmall">
              {
                // TRANSLATORS: Label for settings that enables block of gamling related websites.
                messages.pgettext('vpn-settings-view', 'Gambling')
              }
            </BlockGamblingSwitch.Label>
            <BlockGamblingSwitch.Trigger>
              <BlockGamblingSwitch.Thumb />
            </BlockGamblingSwitch.Trigger>
          </BlockGamblingSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}

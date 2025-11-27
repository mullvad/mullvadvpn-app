import { messages } from '../../../../../shared/gettext';
import { FlexRow } from '../../../../lib/components/flex-row';
import { ListItem } from '../../../../lib/components/list-item';
import { BlockGamblingSwitch } from '../block-gambling-switch/BlockGamblingSwitch';

export function BlockGamblingSetting() {
  return (
    <ListItem level={1}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockGamblingSwitch>
            <FlexRow padding={{ left: 'medium' }}>
              <BlockGamblingSwitch.Label variant="bodySmall">
                {
                  // TRANSLATORS: Label for settings that enables block of gamling related websites.
                  messages.pgettext('vpn-settings-view', 'Gambling')
                }
              </BlockGamblingSwitch.Label>
            </FlexRow>
            <BlockGamblingSwitch.Trigger>
              <BlockGamblingSwitch.Thumb />
            </BlockGamblingSwitch.Trigger>
          </BlockGamblingSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}

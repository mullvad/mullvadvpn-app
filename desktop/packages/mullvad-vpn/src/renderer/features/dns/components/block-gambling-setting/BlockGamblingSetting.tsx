import { messages } from '../../../../../shared/gettext';
import { FlexRow } from '../../../../lib/components/flex-row';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { BlockGamblingSwitch } from '../block-gambling-switch/BlockGamblingSwitch';

export type BlockGamblingSettingProps = Omit<ListItemProps, 'children'>;

export function BlockGamblingSetting(props: BlockGamblingSettingProps) {
  return (
    <ListItem level={1} {...props}>
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

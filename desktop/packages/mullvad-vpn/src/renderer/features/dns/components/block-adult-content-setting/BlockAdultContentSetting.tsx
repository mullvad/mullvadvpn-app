import { messages } from '../../../../../shared/gettext';
import { FlexRow } from '../../../../lib/components/flex-row';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { BlockAdultContentSwitch } from '../block-adult-content-switch/BlockAdultContentSwitch';

export type BlockAdultContentSettingProps = Omit<ListItemProps, 'children'>;

export function BlockAdultContentSetting(props: BlockAdultContentSettingProps) {
  return (
    <ListItem level={1} {...props}>
      <ListItem.Item>
        <ListItem.Content>
          <BlockAdultContentSwitch>
            <FlexRow padding={{ left: 'medium' }}>
              <BlockAdultContentSwitch.Label variant="bodySmall">
                {
                  // TRANSLATORS: Label for settings that enables block of adult content.
                  messages.pgettext('vpn-settings-view', 'Adult content')
                }
              </BlockAdultContentSwitch.Label>
            </FlexRow>
            <BlockAdultContentSwitch.Trigger>
              <BlockAdultContentSwitch.Thumb />
            </BlockAdultContentSwitch.Trigger>
          </BlockAdultContentSwitch>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}

import { messages } from '../../../../../shared/gettext';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { BlockAdultContentSwitch } from '../block-adult-content-switch';

export type BlockAdultContentSettingProps = Omit<ListItemProps, 'children'>;

export function BlockAdultContentSetting(props: BlockAdultContentSettingProps) {
  return (
    <ListItem level={1} {...props}>
      <ListItem.Item>
        <BlockAdultContentSwitch>
          <BlockAdultContentSwitch.Label variant="bodySmall">
            {
              // TRANSLATORS: Label for settings that enables block of adult content.
              messages.pgettext('vpn-settings-view', 'Adult content')
            }
          </BlockAdultContentSwitch.Label>
          <ListItem.ActionGroup>
            <BlockAdultContentSwitch.Trigger>
              <BlockAdultContentSwitch.Thumb />
            </BlockAdultContentSwitch.Trigger>
          </ListItem.ActionGroup>
        </BlockAdultContentSwitch>
      </ListItem.Item>
    </ListItem>
  );
}

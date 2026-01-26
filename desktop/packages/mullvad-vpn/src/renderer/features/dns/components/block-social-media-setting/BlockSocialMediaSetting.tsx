import { messages } from '../../../../../shared/gettext';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { BlockSocialMediaSwitch } from '../block-social-media-switch';

export type BlockSocialMediaSettingProps = Omit<ListItemProps, 'children'>;

export function BlockSocialMediaSetting(props: BlockSocialMediaSettingProps) {
  return (
    <ListItem level={1} {...props}>
      <ListItem.Item>
        <BlockSocialMediaSwitch>
          <BlockSocialMediaSwitch.Label variant="bodySmall">
            {
              // TRANSLATORS: Label for settings that enables block of social media.
              messages.pgettext('vpn-settings-view', 'Social media')
            }
          </BlockSocialMediaSwitch.Label>
          <ListItem.ActionGroup>
            <BlockSocialMediaSwitch.Trigger>
              <BlockSocialMediaSwitch.Thumb />
            </BlockSocialMediaSwitch.Trigger>
          </ListItem.ActionGroup>
        </BlockSocialMediaSwitch>
      </ListItem.Item>
    </ListItem>
  );
}

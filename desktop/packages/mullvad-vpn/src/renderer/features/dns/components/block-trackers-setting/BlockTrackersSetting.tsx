import { messages } from '../../../../../shared/gettext';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { BlockTrackersSwitch } from '../block-trackers-switch';

export type BlockTrackersSettingProps = Omit<ListItemProps, 'children'>;

export function BlockTrackersSetting(props: BlockTrackersSettingProps) {
  return (
    <ListItem level={1} {...props}>
      <ListItem.Item>
        <BlockTrackersSwitch>
          <BlockTrackersSwitch.Label variant="bodySmall">
            {
              // TRANSLATORS: Label for settings that enables tracker blocking.
              messages.pgettext('vpn-settings-view', 'Trackers')
            }
          </BlockTrackersSwitch.Label>
          <ListItem.ActionGroup>
            <BlockTrackersSwitch.Thumb />
          </ListItem.ActionGroup>
        </BlockTrackersSwitch>
      </ListItem.Item>
    </ListItem>
  );
}

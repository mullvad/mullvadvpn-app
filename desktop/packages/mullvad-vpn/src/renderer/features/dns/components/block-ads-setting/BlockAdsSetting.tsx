import { messages } from '../../../../../shared/gettext';
import { ListItem, ListItemProps } from '../../../../lib/components/list-item';
import { BlockAdsSwitch } from '../block-ads-switch';

export type BlockAdsSettingProps = Omit<ListItemProps, 'children'>;

export function BlockAdsSetting(props: BlockAdsSettingProps) {
  return (
    <ListItem level={1} {...props}>
      <ListItem.Item>
        <BlockAdsSwitch>
          <BlockAdsSwitch.Label variant="bodySmall">
            {
              // TRANSLATORS: Label for settings that enables ad blocking.
              messages.pgettext('vpn-settings-view', 'Ads')
            }
          </BlockAdsSwitch.Label>
          <ListItem.ActionGroup>
            <BlockAdsSwitch.Trigger>
              <BlockAdsSwitch.Thumb />
            </BlockAdsSwitch.Trigger>
          </ListItem.ActionGroup>
        </BlockAdsSwitch>
      </ListItem.Item>
    </ListItem>
  );
}

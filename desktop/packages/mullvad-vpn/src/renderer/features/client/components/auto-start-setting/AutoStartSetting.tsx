import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { AutoStartSwitch } from '../auto-start-switch';

export type AutoStartSettingProps = Omit<ListItemProps, 'children'>;
export function AutoStartSetting(props: AutoStartSettingProps) {
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <AutoStartSwitch>
            <AutoStartSwitch.Label>
              {messages.pgettext('vpn-settings-view', 'Launch app on start-up')}
            </AutoStartSwitch.Label>
            <AutoStartSwitch.Trigger>
              <AutoStartSwitch.Thumb />
            </AutoStartSwitch.Trigger>
          </AutoStartSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}

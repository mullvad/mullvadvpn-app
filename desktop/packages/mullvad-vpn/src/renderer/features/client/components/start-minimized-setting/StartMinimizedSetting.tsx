import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { StartMinimizedSwitch } from '../start-minimized-switch/StartMinimizedSwitch';

export type StartMinimizedSettingProps = Omit<ListItemProps, 'children'>;

export function StartMinimizedSetting(props: StartMinimizedSettingProps) {
  const description =
    process.platform === 'darwin'
      ? messages.pgettext(
          'user-interface-settings-view',
          'Show only the menu bar icon when the app starts.',
        )
      : messages.pgettext(
          'user-interface-settings-view',
          'Show only the tray icon when the app starts.',
        );
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <StartMinimizedSwitch>
          <StartMinimizedSwitch.Label>
            {messages.pgettext('user-interface-settings-view', 'Start minimized')}
          </StartMinimizedSwitch.Label>
          <SettingsListItem.Item.ActionGroup>
            <StartMinimizedSwitch.Input />
          </SettingsListItem.Item.ActionGroup>
        </StartMinimizedSwitch>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Footer.Text>{description}</SettingsListItem.Footer.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}

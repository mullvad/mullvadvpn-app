import { messages } from '../../../../../shared/gettext';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { StartMinimizedSwitch } from '../start-minimized-switch/StartMinimizedSwitch';

export type StartMinimizedSettingProps = Omit<ListItemProps, 'children'>;

export function StartMinimizedSetting(props: StartMinimizedSettingProps) {
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <StartMinimizedSwitch>
          <StartMinimizedSwitch.Label>
            {messages.pgettext('user-interface-settings-view', 'Start minimized')}
          </StartMinimizedSwitch.Label>
          <SettingsListItem.ActionGroup>
            <StartMinimizedSwitch.Thumb />
          </SettingsListItem.ActionGroup>
        </StartMinimizedSwitch>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.FooterText>
          {messages.pgettext(
            'user-interface-settings-view',
            'Show only the tray icon when the app starts.',
          )}
        </SettingsListItem.FooterText>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}

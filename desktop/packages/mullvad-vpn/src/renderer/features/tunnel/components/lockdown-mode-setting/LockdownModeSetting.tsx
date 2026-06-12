import { messages } from '../../../../../shared/gettext';
import { Info } from '../../../../components/info';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { LockdownModeSwitch } from '../lockdown-mode-switch';

export type LockdownModeSettingProps = Omit<ListItemProps, 'children'>;

export function LockdownModeSetting(props: LockdownModeSettingProps) {
  return (
    <SettingsListItem anchorId="lockdown-mode-setting" {...props}>
      <SettingsListItem.Item>
        <LockdownModeSwitch>
          <LockdownModeSwitch.Label>
            {messages.pgettext('vpn-settings-view', 'Lockdown mode')}
          </LockdownModeSwitch.Label>
          <SettingsListItem.Item.ActionGroup>
            <Info>
              <Info.Button />
              <Info.Dialog>
                <Info.Dialog.Text>
                  {messages.pgettext(
                    'vpn-settings-view',
                    'The difference between the Kill Switch and Lockdown Mode is that the Kill Switch will prevent any leaks from happening during automatic tunnel reconnects, software crashes and similar accidents.',
                  )}
                </Info.Dialog.Text>
                <Info.Dialog.Text>
                  {messages.pgettext(
                    'vpn-settings-view',
                    'With Lockdown Mode enabled, you must be connected to a Mullvad VPN server to be able to reach the internet. Manually disconnecting or quitting the app will block your connection.',
                  )}
                </Info.Dialog.Text>
              </Info.Dialog>
            </Info>
            <LockdownModeSwitch.Input />
          </SettingsListItem.Item.ActionGroup>
        </LockdownModeSwitch>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}

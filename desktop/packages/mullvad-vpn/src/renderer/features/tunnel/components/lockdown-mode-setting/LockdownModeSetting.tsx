import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../shared/gettext';
import { Info } from '../../../../components/info';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { LockdownModeSwitch } from '../lockdown-mode-switch';

export type LockdownModeSettingProps = Omit<ListItemProps, 'children'>;

export function LockdownModeSetting(props: LockdownModeSettingProps) {
  const lockdownModeSettingName = messages.gettext('Lockdown mode');

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
                  {sprintf(
                    // TRANSLATORS: Explanation shown in the info dialog for Lockdown mode.
                    // TRANSLATORS: Available placeholders:
                    // TRANSLATORS: %(lockdownModeSettingName)s - The name of the Lockdown mode setting
                    messages.pgettext(
                      'vpn-settings-view',
                      'With %(lockdownModeSettingName)s enabled, you can only reach the internet if you’re connected through the Mullvad app. Your network traffic will be blocked if you manually disconnect or quit the app.',
                    ),
                    { lockdownModeSettingName },
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

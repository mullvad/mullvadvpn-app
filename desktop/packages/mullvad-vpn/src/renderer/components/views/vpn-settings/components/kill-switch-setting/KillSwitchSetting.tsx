import { messages } from '../../../../../../shared/gettext';
import { ListItemProps } from '../../../../../lib/components/list-item';
import { Switch } from '../../../../../lib/components/switch';
import { Info } from '../../../../info';
import { SettingsListItem } from '../../../../settings-list-item';

export type KillSwitchSettingProps = Omit<ListItemProps, 'children'>;

export function KillSwitchSetting(props: KillSwitchSettingProps) {
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <SettingsListItem.Item.Label>
          {messages.pgettext('vpn-settings-view', 'Kill switch')}
        </SettingsListItem.Item.Label>
        <SettingsListItem.Item.ActionGroup>
          <Info>
            <Info.Button />
            <Info.Dialog>
              <Info.Dialog.Text>
                {messages.pgettext(
                  'vpn-settings-view',
                  'This built-in feature prevents your traffic from leaking outside of the VPN tunnel if your network suddenly stops working or if the tunnel fails, it does this by blocking your traffic until your connection is reestablished.',
                )}
              </Info.Dialog.Text>
              <Info.Dialog.Text>
                {messages.pgettext(
                  'vpn-settings-view',
                  'The difference between the Kill Switch and Lockdown Mode is that the Kill Switch will prevent any leaks from happening during automatic tunnel reconnects, software crashes and similar accidents. With Lockdown Mode enabled, you must be connected to a Mullvad VPN server to be able to reach the internet. Manually disconnecting or quitting the app will block your connection.',
                )}
              </Info.Dialog.Text>
            </Info.Dialog>
          </Info>
          <Switch checked disabled>
            <Switch.Input />
          </Switch>
        </SettingsListItem.Item.ActionGroup>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}

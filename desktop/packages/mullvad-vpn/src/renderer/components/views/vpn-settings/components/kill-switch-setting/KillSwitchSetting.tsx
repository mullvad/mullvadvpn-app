import { messages } from '../../../../../../shared/gettext';
import { useScrollToListItem } from '../../../../../hooks';
import { ListItem } from '../../../../../lib/components/list-item';
import { Switch } from '../../../../../lib/components/switch';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';

export function KillSwitchSetting() {
  const { animation } = useScrollToListItem();

  return (
    <ListItem animation={animation}>
      <ListItem.Item>
        <ListItem.Content>
          <ListItem.Label>{messages.pgettext('vpn-settings-view', 'Kill switch')}</ListItem.Label>
          <ListItem.Group $gap="medium">
            <InfoButton>
              <ModalMessage>
                {messages.pgettext(
                  'vpn-settings-view',
                  'This built-in feature prevents your traffic from leaking outside of the VPN tunnel if your network suddenly stops working or if the tunnel fails, it does this by blocking your traffic until your connection is reestablished.',
                )}
              </ModalMessage>
              <ModalMessage>
                {messages.pgettext(
                  'vpn-settings-view',
                  'The difference between the Kill Switch and Lockdown Mode is that the Kill Switch will prevent any leaks from happening during automatic tunnel reconnects, software crashes and similar accidents. With Lockdown Mode enabled, you must be connected to a Mullvad VPN server to be able to reach the internet. Manually disconnecting or quitting the app will block your connection.',
                )}
              </ModalMessage>
            </InfoButton>
            <Switch checked disabled>
              <Switch.Thumb />
            </Switch>
          </ListItem.Group>
        </ListItem.Content>
      </ListItem.Item>
    </ListItem>
  );
}

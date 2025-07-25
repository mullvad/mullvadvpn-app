import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { Button } from '../../../../../lib/components';
import { spacings } from '../../../../../lib/foundations';
import { useBoolean } from '../../../../../lib/utility-hooks';
import { AriaInput, AriaInputGroup, AriaLabel } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';
import InfoButton from '../../../../InfoButton';
import { ModalAlert, ModalAlertType, ModalMessage } from '../../../../Modal';

const StyledInfoButton = styled(InfoButton)({
  marginRight: spacings.medium,
});

export function KillSwitchSetting() {
  const [killSwitchInfoVisible, showKillSwitchInfo, hideKillSwitchInfo] = useBoolean(false);

  return (
    <>
      <AriaInputGroup>
        <Cell.Container>
          <AriaLabel>
            <Cell.InputLabel>
              {messages.pgettext('vpn-settings-view', 'Kill switch')}
            </Cell.InputLabel>
          </AriaLabel>
          <StyledInfoButton onClick={showKillSwitchInfo} />
          <AriaInput>
            <Cell.Switch isOn disabled />
          </AriaInput>
        </Cell.Container>
      </AriaInputGroup>
      <ModalAlert
        isOpen={killSwitchInfoVisible}
        type={ModalAlertType.info}
        buttons={[
          <Button key="back" onClick={hideKillSwitchInfo}>
            <Button.Text>{messages.gettext('Got it!')}</Button.Text>
          </Button>,
        ]}
        close={hideKillSwitchInfo}>
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
      </ModalAlert>
    </>
  );
}

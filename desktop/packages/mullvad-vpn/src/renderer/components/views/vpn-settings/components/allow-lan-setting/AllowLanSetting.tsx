import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { spacings } from '../../../../../lib/foundations';
import { useSelector } from '../../../../../redux/store';
import { AriaDetails, AriaInput, AriaInputGroup, AriaLabel } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';

const StyledInfoButton = styled(InfoButton)({
  marginRight: spacings.medium,
});

const LanIpRanges = styled.ul({
  listStyle: 'disc outside',
  marginLeft: spacings.large,
});

export function AllowLanSetting() {
  const allowLan = useSelector((state) => state.settings.allowLan);
  const { setAllowLan } = useAppContext();

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>
            {messages.pgettext('vpn-settings-view', 'Local network sharing')}
          </Cell.InputLabel>
        </AriaLabel>
        <AriaDetails>
          <StyledInfoButton>
            <ModalMessage>
              {messages.pgettext(
                'vpn-settings-view',
                'This feature allows access to other devices on the local network, such as for sharing, printing, streaming, etc.',
              )}
            </ModalMessage>
            <ModalMessage>
              {messages.pgettext(
                'vpn-settings-view',
                'It does this by allowing network communication outside the tunnel to local multicast and broadcast ranges as well as to and from these private IP ranges:',
              )}
              <LanIpRanges>
                <li>10.0.0.0/8</li>
                <li>172.16.0.0/12</li>
                <li>192.168.0.0/16</li>
                <li>169.254.0.0/16</li>
                <li>fe80::/10</li>
                <li>fc00::/7</li>
              </LanIpRanges>
            </ModalMessage>
          </StyledInfoButton>
        </AriaDetails>
        <AriaInput>
          <Cell.Switch isOn={allowLan} onChange={setAllowLan} />
        </AriaInput>
      </Cell.Container>
    </AriaInputGroup>
  );
}

import { useCallback } from 'react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
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

export function EnableIpv6Setting() {
  const enableIpv6 = useSelector((state) => state.settings.enableIpv6);
  const { setEnableIpv6: setEnableIpv6Impl } = useAppContext();

  const setEnableIpv6 = useCallback(
    async (enableIpv6: boolean) => {
      try {
        await setEnableIpv6Impl(enableIpv6);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update enable IPv6', error.message);
      }
    },
    [setEnableIpv6Impl],
  );

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>{messages.pgettext('vpn-settings-view', 'Enable IPv6')}</Cell.InputLabel>
        </AriaLabel>
        <AriaDetails>
          <StyledInfoButton>
            <ModalMessage>
              {messages.pgettext(
                'vpn-settings-view',
                'When this feature is enabled, IPv6 can be used alongside IPv4 in the VPN tunnel to communicate with internet services.',
              )}
            </ModalMessage>
            <ModalMessage>
              {messages.pgettext(
                'vpn-settings-view',
                'IPv4 is always enabled and the majority of websites and applications use this protocol. We do not recommend enabling IPv6 unless you know you need it.',
              )}
            </ModalMessage>
          </StyledInfoButton>
        </AriaDetails>
        <AriaInput>
          <Cell.Switch isOn={enableIpv6} onChange={setEnableIpv6} />
        </AriaInput>
      </Cell.Container>
    </AriaInputGroup>
  );
}

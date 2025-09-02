import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../../../../../shared/constants';
import {
  BridgeState,
  RelayProtocol,
  TunnelProtocol,
} from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelector } from '../../../../../redux/store';
import { AriaDescription, AriaInputGroup } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';
import Selector, { SelectorItem } from '../../../../cell/Selector';
import { ModalMessage } from '../../../../Modal';

const StyledSelectorContainer = styled.div({
  flex: 0,
});

export function BridgeModeSetting() {
  const { setBridgeState: setBridgeStateImpl } = useAppContext();
  const relaySettings = useSelector((state) => state.settings.relaySettings);

  const bridgeState = useSelector((state) => state.settings.bridgeState);

  const tunnelProtocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.tunnelProtocol : 'any';
    return protocol === 'any' ? null : protocol;
  }, [relaySettings]);

  const transportProtocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.openvpn.protocol : 'any';
    return protocol === 'any' ? null : protocol;
  }, [relaySettings]);

  const options: SelectorItem<BridgeState>[] = useMemo(
    () => [
      {
        label: messages.gettext('On'),
        value: 'on',
        disabled: tunnelProtocol !== 'openvpn' || transportProtocol === 'udp',
        'data-testid': 'bridge-mode-on',
      },
      {
        label: messages.gettext('Off'),
        value: 'off',
      },
    ],
    [tunnelProtocol, transportProtocol],
  );

  const setBridgeState = useCallback(
    async (bridgeState: BridgeState) => {
      try {
        await setBridgeStateImpl(bridgeState);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to update bridge state: ${error.message}`);
      }
    },
    [setBridgeStateImpl],
  );

  const onSelectBridgeState = useCallback(
    async (newValue: BridgeState) => {
      await setBridgeState(newValue);
    },
    [setBridgeState],
  );

  const footerText = bridgeModeFooterText(bridgeState === 'on', tunnelProtocol, transportProtocol);

  return (
    <>
      <AriaInputGroup>
        <StyledSelectorContainer>
          <Selector
            title={
              // TRANSLATORS: The title for the shadowsocks bridge selector section.
              messages.pgettext('openvpn-settings-view', 'Bridge mode')
            }
            infoTitle={messages.pgettext('openvpn-settings-view', 'Bridge mode')}
            details={
              <>
                <ModalMessage>
                  {sprintf(
                    // TRANSLATORS: This is used as a description for the bridge mode
                    // TRANSLATORS: setting.
                    // TRANSLATORS: Available placeholders:
                    // TRANSLATORS: %(openvpn)s - will be replaced with OpenVPN
                    messages.pgettext(
                      'openvpn-settings-view',
                      'Helps circumvent censorship, by routing your traffic through a bridge server before reaching an %(openvpn)s server. Obfuscation is added to make fingerprinting harder.',
                    ),
                    { openvpn: strings.openvpn },
                  )}
                </ModalMessage>
                <ModalMessage>
                  {messages.gettext('This setting increases latency. Use only if needed.')}
                </ModalMessage>
              </>
            }
            items={options}
            value={bridgeState}
            onSelect={onSelectBridgeState}
            automaticValue={'auto' as const}
          />
        </StyledSelectorContainer>
        {footerText !== undefined && (
          <Cell.CellFooter>
            <AriaDescription>
              <Cell.CellFooterText>{footerText}</Cell.CellFooterText>
            </AriaDescription>
          </Cell.CellFooter>
        )}
      </AriaInputGroup>
    </>
  );
}

function bridgeModeFooterText(
  bridgeModeOn: boolean,
  tunnelProtocol: TunnelProtocol | null,
  transportProtocol: RelayProtocol | null,
): React.ReactNode | void {
  if (bridgeModeOn) {
    // TRANSLATORS: This text is shown beneath the bridge mode setting to instruct users how to
    // TRANSLATORS: configure the feature further.
    return messages.pgettext(
      'openvpn-settings-view',
      'To select a specific bridge server, go to the Select location view.',
    );
  } else if (tunnelProtocol !== 'openvpn') {
    return formatHtml(
      sprintf(
        // TRANSLATORS: This is used to instruct users how to make the bridge mode setting
        // TRANSLATORS: available.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(tunnelProtocol)s - the name of the tunnel protocol setting
        // TRANSLATORS: %(openvpn)s - will be replaced with OpenVPN
        messages.pgettext(
          'openvpn-settings-view',
          'To activate Bridge mode, go back and change <b>%(tunnelProtocol)s</b> to <b>%(openvpn)s</b>.',
        ),
        {
          tunnelProtocol: messages.pgettext('vpn-settings-view', 'Tunnel protocol'),
          openvpn: strings.openvpn,
        },
      ),
    );
  } else if (transportProtocol === 'udp') {
    return formatHtml(
      sprintf(
        // TRANSLATORS: This is used to instruct users how to make the bridge mode setting
        // TRANSLATORS: available.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(transportProtocol)s - the name of the transport protocol setting
        // TRANSLATORS: %(automatic)s - the translation of "Automatic"
        // TRANSLATORS: %(tcp)s - the translation of "TCP"
        messages.pgettext(
          'openvpn-settings-view',
          'To activate Bridge mode, change <b>%(transportProtocol)s</b> to <b>%(automatic)s</b> or <b>%(tcp)s</b>.',
        ),
        {
          transportProtocol: messages.pgettext('openvpn-settings-view', 'Transport protocol'),
          automatic: messages.gettext('Automatic'),
          tcp: messages.gettext('TCP'),
        },
      ),
    );
  }
}

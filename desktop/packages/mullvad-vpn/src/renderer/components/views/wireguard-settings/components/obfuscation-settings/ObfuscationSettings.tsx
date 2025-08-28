import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { Constraint, ObfuscationType } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { AriaInputGroup } from '../../../../AriaGroup';
import Selector, { SelectorItem } from '../../../../cell/Selector';
import { ModalMessage } from '../../../../Modal';

const StyledSelectorContainer = styled.div({
  flex: 0,
});

export function formatPortForSubLabel(port: Constraint<number>): string {
  return port === 'any' ? messages.gettext('Automatic') : `${port.only}`;
}

export function ObfuscationSettings() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  // TRANSLATORS: Text showing currently selected port.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(port)s - Can be either a number between 1 and 65535 or the text "Automatic".
  const subLabelTemplate = messages.pgettext('wireguard-settings-view', 'Port: %(port)s');

  const obfuscationType = obfuscationSettings.selectedObfuscation;
  const obfuscationTypeItems: SelectorItem<ObfuscationType>[] = useMemo(
    () => [
      {
        label: messages.pgettext('wireguard-settings-view', 'Shadowsocks'),
        subLabel: sprintf(subLabelTemplate, {
          port: formatPortForSubLabel(obfuscationSettings.shadowsocksSettings.port),
        }),
        value: ObfuscationType.shadowsocks,
        details: {
          path: RoutePath.shadowsocks,
          ariaLabel: messages.pgettext('accessibility', 'Shadowsocks settings'),
        },
      },
      {
        label: messages.pgettext('wireguard-settings-view', 'UDP-over-TCP'),
        subLabel: sprintf(subLabelTemplate, {
          port: formatPortForSubLabel(obfuscationSettings.udp2tcpSettings.port),
        }),
        value: ObfuscationType.udp2tcp,
        details: {
          path: RoutePath.udpOverTcp,
          ariaLabel: messages.pgettext('accessibility', 'UDP-over-TCP settings'),
        },
      },
      {
        label: messages.pgettext('wireguard-settings-view', 'QUIC'),
        value: ObfuscationType.quic,
      },
      {
        label: messages.gettext('Off'),
        value: ObfuscationType.off,
      },
    ],
    [
      obfuscationSettings.shadowsocksSettings.port,
      obfuscationSettings.udp2tcpSettings.port,
      subLabelTemplate,
    ],
  );

  const selectObfuscationType = useCallback(
    async (value: ObfuscationType) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        selectedObfuscation: value,
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          // TRANSLATORS: The title for the WireGuard obfuscation selector.
          title={messages.pgettext('wireguard-settings-view', 'Obfuscation')}
          details={
            <ModalMessage>
              {
                // TRANSLATORS: Describes what WireGuard obfuscation does, how it works and when
                // TRANSLATORS: it would be useful to enable it.
                messages.pgettext(
                  'wireguard-settings-view',
                  'Obfuscation hides the WireGuard traffic inside another protocol. It can be used to help circumvent censorship and other types of filtering, where a plain WireGuard connection would be blocked.',
                )
              }
            </ModalMessage>
          }
          items={obfuscationTypeItems}
          value={obfuscationType}
          onSelect={selectObfuscationType}
          automaticValue={ObfuscationType.auto}
          automaticTestId="automatic-obfuscation"
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { isInRanges } from '../../../../../../shared/utils';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useSelector } from '../../../../../redux/store';
import { AriaInputGroup } from '../../../../AriaGroup';
import { SelectorItem, SelectorWithCustomItem } from '../../../../cell/Selector';
import { ModalMessage } from '../../../../Modal';

const WIREUGARD_UDP_PORTS = [51820, 53];

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}

const StyledSelectorContainer = styled.div({
  flex: 0,
});

export function PortSelector() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const allowedPortRanges = useSelector((state) => state.settings.wireguardEndpointData.portRanges);

  const wireguardPortItems = useMemo<Array<SelectorItem<number>>>(
    () => WIREUGARD_UDP_PORTS.map(mapPortToSelectorItem),
    [],
  );

  const port = useMemo(() => {
    const port = 'normal' in relaySettings ? relaySettings.normal.wireguard.port : 'any';
    return port === 'any' ? null : port;
  }, [relaySettings]);

  const setWireguardPort = useCallback(
    async (port: number | null) => {
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.port = wrapConstraint(port);
          return settings;
        });
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettingsUpdater],
  );

  const parseValue = useCallback((port: string) => parseInt(port), []);

  const validateValue = useCallback(
    (value: number) => isInRanges(value, allowedPortRanges),
    [allowedPortRanges],
  );

  const portRangesText = allowedPortRanges
    .map(([start, end]) => (start === end ? start : `${start}-${end}`))
    .join(', ');

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <SelectorWithCustomItem
          // TRANSLATORS: The title for the WireGuard port selector.
          title={messages.pgettext('wireguard-settings-view', 'Port')}
          items={wireguardPortItems}
          value={port}
          onSelect={setWireguardPort}
          inputPlaceholder={messages.pgettext('wireguard-settings-view', 'Port')}
          automaticValue={null}
          parseValue={parseValue}
          modifyValue={removeNonNumericCharacters}
          validateValue={validateValue}
          maxLength={5}
          details={
            <>
              <ModalMessage>
                {messages.pgettext(
                  'wireguard-settings-view',
                  'The automatic setting will randomly choose from the valid port ranges shown below.',
                )}
              </ModalMessage>
              <ModalMessage>
                {sprintf(
                  messages.pgettext(
                    'wireguard-settings-view',
                    'The custom port can be any value inside the valid ranges: %(portRanges)s.',
                  ),
                  { portRanges: portRangesText },
                )}
              </ModalMessage>
            </>
          }
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

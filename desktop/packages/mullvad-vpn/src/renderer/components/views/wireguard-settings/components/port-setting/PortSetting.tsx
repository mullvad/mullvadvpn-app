import { useCallback, useMemo } from 'react';
import React from 'react';
import { sprintf } from 'sprintf-js';

import { wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { isInRanges } from '../../../../../../shared/utils';
import { useScrollToListItem } from '../../../../../hooks';
import { Listbox } from '../../../../../lib/components/listbox/Listbox';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useSelector } from '../../../../../redux/store';
import { SelectorItem } from '../../../../cell/Selector';
import { DefaultListboxOption } from '../../../../default-listbox-option';
import InfoButton from '../../../../InfoButton';
import { InputListboxOption } from '../../../../input-listbox-option';
import { ModalMessage } from '../../../../Modal';

const WIREUGARD_UDP_PORTS = [51820, 53];

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}
export function PortSetting() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const allowedPortRanges = useSelector((state) => state.settings.wireguardEndpointData.portRanges);

  const id = 'port-setting';
  const ref = React.useRef<HTMLDivElement>(null);
  const scrollToAnchor = useScrollToListItem(ref, id);

  const wireguardPortItems = useMemo<Array<SelectorItem<number>>>(
    () => WIREUGARD_UDP_PORTS.map(mapPortToSelectorItem),
    [],
  );

  const selectedOption = useMemo(() => {
    const port = 'normal' in relaySettings ? relaySettings.normal.wireguard.port : 'any';
    if (port === 'any')
      return {
        port: 'any',
        value: null,
      };
    if (port && !WIREUGARD_UDP_PORTS.includes(port))
      return {
        port,
        value: 'custom',
      };
    return {
      port,
      value: port,
    };
  }, [relaySettings]);

  const setWireguardPort = useCallback(
    async (port: number | string | null) => {
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.port = wrapConstraint(
            typeof port === 'string' ? parseInt(port) : port,
          );
          return settings;
        });
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettingsUpdater],
  );

  const validateValue = useCallback(
    (value: number) => isInRanges(value, allowedPortRanges),
    [allowedPortRanges],
  );

  const validateStringValue = useCallback(
    (value: string) => {
      const numericValue = parseInt(value, 10);
      if (Number.isNaN(numericValue)) return false;
      return validateValue(numericValue);
    },
    [validateValue],
  );

  const portRangesText = allowedPortRanges
    .map(([start, end]) => (start === end ? start : `${start}-${end}`))
    .join(', ');

  return (
    <Listbox
      value={selectedOption.value}
      onValueChange={setWireguardPort}
      animation={scrollToAnchor?.animation}>
      <Listbox.Item ref={ref}>
        <Listbox.Content>
          <Listbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard port selector.
              messages.pgettext('wireguard-settings-view', 'Port')
            }
          </Listbox.Label>
          <InfoButton>
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
          </InfoButton>
        </Listbox.Content>
      </Listbox.Item>
      <Listbox.Options>
        <DefaultListboxOption value={null}>{messages.gettext('Automatic')}</DefaultListboxOption>
        {wireguardPortItems.map((item) => (
          <DefaultListboxOption key={item.value} value={item.value}>
            {item.label}
          </DefaultListboxOption>
        ))}
        <InputListboxOption value="custom">
          <InputListboxOption.Label>{messages.gettext('Custom')}</InputListboxOption.Label>
          <InputListboxOption.Input
            initialValue={
              selectedOption.value === 'custom' ? selectedOption.port?.toString() : undefined
            }
            placeholder={messages.pgettext('wireguard-settings-view', 'Port')}
            maxLength={5}
            type="text"
            inputMode="numeric"
            validate={validateStringValue}
            format={removeNonNumericCharacters}
          />
        </InputListboxOption>
      </Listbox.Options>
    </Listbox>
  );
}

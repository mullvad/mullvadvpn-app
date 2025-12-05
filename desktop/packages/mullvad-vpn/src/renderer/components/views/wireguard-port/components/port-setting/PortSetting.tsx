import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';

import { liftConstraint, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { isInRanges } from '../../../../../../shared/utils';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SelectorItem } from '../../../../cell/Selector';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListbox } from '../../../../settings-listbox';

const WIREGUARD_UDP_PORTS = [51820, 53];

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}
export function PortSetting() {
  const { setObfuscationSettings } = useAppContext();

  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);
  const allowedPortRanges = useSelector((state) => state.settings.wireguardEndpointData.portRanges);

  const wireguardPortItems = useMemo<Array<SelectorItem<number>>>(
    () => WIREGUARD_UDP_PORTS.map(mapPortToSelectorItem),
    [],
  );

  const selectedOption = useMemo(() => {
    const port = liftConstraint(obfuscationSettings.wireGuardPortSettings.port);

    if (port === 'any') {
      return {
        port: 'any',
        value: null,
      };
    }

    if (port && !WIREGUARD_UDP_PORTS.includes(port)) {
      return {
        port,
        value: 'custom',
      };
    }

    return {
      port,
      value: port,
    };
  }, [obfuscationSettings]);

  const setWireguardPort = useCallback(
    async (port: number | string | null) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        wireGuardPortSettings: {
          ...obfuscationSettings.wireGuardPortSettings,
          port: wrapConstraint(typeof port === 'string' ? parseInt(port) : port),
        },
      });
    },
    [setObfuscationSettings, obfuscationSettings],
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
    <SettingsListbox
      anchorId="port-setting"
      value={selectedOption.value}
      onValueChange={setWireguardPort}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard port selector.
              messages.pgettext('wireguard-settings-view', 'Port')
            }
          </SettingsListbox.Label>
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
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <SettingsListbox.BaseOption value={null}>
          {messages.gettext('Automatic')}
        </SettingsListbox.BaseOption>
        {wireguardPortItems.map((item) => (
          <SettingsListbox.BaseOption key={item.value} value={item.value}>
            {item.label}
          </SettingsListbox.BaseOption>
        ))}
        <SettingsListbox.InputOption
          defaultValue={
            selectedOption.value === 'custom' ? selectedOption.port?.toString() : undefined
          }
          value="custom"
          validate={validateStringValue}
          format={removeNonNumericCharacters}>
          <SettingsListbox.InputOption.Label>
            {messages.gettext('Custom')}
          </SettingsListbox.InputOption.Label>
          <SettingsListbox.InputOption.Input
            placeholder={messages.pgettext('wireguard-settings-view', 'Port')}
            maxLength={5}
            type="text"
            inputMode="numeric"
          />
        </SettingsListbox.InputOption>
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}

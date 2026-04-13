import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';

import { liftConstraint, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { useAppContext } from '../../../../../context';
import {
  formatPortRanges,
  validatePortString,
} from '../../../../../features/anti-censorship/utils';
import { useSelector } from '../../../../../redux/store';
import { SelectorItem } from '../../../../cell/Selector';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListbox } from '../../../../settings-listbox';

const LWO_UDP_PORTS = [51820, 53];

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}
export function LwoPortSetting() {
  const { setObfuscationSettings } = useAppContext();

  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);
  const allowedPortRanges = useSelector((state) => state.settings.wireguardEndpointData.portRanges);

  const lwoPortItems = useMemo<Array<SelectorItem<number>>>(
    () => LWO_UDP_PORTS.map(mapPortToSelectorItem),
    [],
  );

  const selectedOption = useMemo(() => {
    const port = liftConstraint(obfuscationSettings.lwoSettings.port);

    if (port === 'any') {
      return {
        port: 'any',
        value: null,
      };
    }

    if (port && !LWO_UDP_PORTS.includes(port)) {
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

  const setLwoPort = useCallback(
    async (port: number | string | null) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        lwoSettings: {
          ...obfuscationSettings.lwoSettings,
          port: wrapConstraint(typeof port === 'string' ? parseInt(port) : port),
        },
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  const validateStringValue = useCallback(
    (value: string) => validatePortString(value, allowedPortRanges),
    [allowedPortRanges],
  );

  const portRangesText = formatPortRanges(allowedPortRanges);

  return (
    <SettingsListbox
      anchorId="port-setting"
      value={selectedOption.value}
      onValueChange={setLwoPort}>
      <SettingsListbox.Header>
        <SettingsListbox.Header.Item>
          <SettingsListbox.Header.Item.Label>
            {
              // TRANSLATORS: The title for the LWO port selector.
              messages.pgettext('lwo-settings-view', 'Port')
            }
          </SettingsListbox.Header.Item.Label>
          <SettingsListbox.Header.Item.ActionGroup>
            <InfoButton>
              <>
                <ModalMessage>
                  {messages.pgettext(
                    'lwo-settings-view',
                    'The automatic setting will randomly choose from the valid port ranges shown below.',
                  )}
                </ModalMessage>
                <ModalMessage>
                  {sprintf(
                    messages.pgettext(
                      'lwo-settings-view',
                      'The custom port can be any value inside the valid ranges: %(portRanges)s.',
                    ),
                    { portRanges: portRangesText },
                  )}
                </ModalMessage>
              </>
            </InfoButton>
          </SettingsListbox.Header.Item.ActionGroup>
        </SettingsListbox.Header.Item>
      </SettingsListbox.Header>
      <SettingsListbox.Options>
        <SettingsListbox.Options.BaseOption value={null}>
          {messages.gettext('Automatic')}
        </SettingsListbox.Options.BaseOption>
        {lwoPortItems.map((item) => (
          <SettingsListbox.Options.BaseOption key={item.value} value={item.value}>
            {item.label}
          </SettingsListbox.Options.BaseOption>
        ))}
        <SettingsListbox.Options.InputOption
          defaultValue={
            selectedOption.value === 'custom' ? selectedOption.port?.toString() : undefined
          }
          value="custom"
          validate={validateStringValue}
          format={removeNonNumericCharacters}>
          <SettingsListbox.Options.InputOption.Label>
            {messages.gettext('Custom')}
          </SettingsListbox.Options.InputOption.Label>
          <SettingsListbox.Header.Item.ActionGroup>
            <SettingsListbox.Options.InputOption.Input
              placeholder={messages.pgettext('lwo-settings-view', 'Port')}
              maxLength={5}
              type="text"
              inputMode="numeric"
            />
          </SettingsListbox.Header.Item.ActionGroup>
        </SettingsListbox.Options.InputOption>
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}

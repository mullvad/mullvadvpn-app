import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { useAppContext } from '../../../../../context';
import { Listbox } from '../../../../../lib/components/listbox/Listbox';
import { useSelector } from '../../../../../redux/store';
import { DefaultListboxOption } from '../../../../default-listbox-option';
import { InputListboxOption } from '../../../../input-listbox-option';

const ALLOWED_RANGE = [1, 65535];

export function ShadowsocksPortSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);
  const descriptionId = React.useId();

  const selectedOption = React.useMemo(() => {
    const port = obfuscationSettings.shadowsocksSettings.port;
    if (port === 'any')
      return {
        port: 'any',
        value: null,
      };
    return {
      port: port.only,
      value: 'custom',
    };
  }, [obfuscationSettings]);

  const setShadowsocksPort = useCallback(
    async (port: string | null) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        shadowsocksSettings: {
          ...obfuscationSettings.shadowsocksSettings,
          port: wrapConstraint(typeof port === 'string' ? parseInt(port) : port),
        },
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  const validateValue = useCallback((value: string) => {
    const port = parseInt(value);
    return port >= ALLOWED_RANGE[0] && port <= ALLOWED_RANGE[1];
  }, []);

  return (
    <Listbox value={selectedOption.value} onValueChange={setShadowsocksPort}>
      <Listbox.Item>
        <Listbox.Content>
          <Listbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard port selector.
              messages.pgettext('wireguard-settings-view', 'Port')
            }
          </Listbox.Label>
        </Listbox.Content>
      </Listbox.Item>
      <Listbox.Options>
        <DefaultListboxOption value={null}>{messages.gettext('Automatic')}</DefaultListboxOption>
        <InputListboxOption value="custom">
          <InputListboxOption.Label>{messages.gettext('Custom')}</InputListboxOption.Label>
          <InputListboxOption.Input
            aria-describedby={descriptionId}
            type="text"
            placeholder={messages.pgettext('wireguard-settings-view', 'Port')}
            initialValue={
              selectedOption.value === 'custom' ? selectedOption.port?.toString() : undefined
            }
            validate={validateValue}
            format={removeNonNumericCharacters}
            maxLength={`${ALLOWED_RANGE[1]}`.length}
          />
        </InputListboxOption>
      </Listbox.Options>
      <Listbox.Footer>
        <Listbox.Text id={descriptionId}>
          {sprintf(
            // TRANSLATORS: Text describing the valid port range for a port selector.
            messages.pgettext('wireguard-settings-view', 'Valid range: %(min)s - %(max)s'),
            { min: ALLOWED_RANGE[0], max: ALLOWED_RANGE[1] },
          )}
        </Listbox.Text>
      </Listbox.Footer>
    </Listbox>
  );
}

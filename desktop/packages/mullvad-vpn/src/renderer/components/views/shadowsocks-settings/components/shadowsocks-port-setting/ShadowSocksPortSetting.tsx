import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { liftConstraint, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { removeNonNumericCharacters } from '../../../../../../shared/string-helpers';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsListItem } from '../../../../settings-list-item';
import { SettingsListbox } from '../../../../settings-listbox';

const ALLOWED_RANGE = [1, 65535];

export function ShadowsocksPortSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);
  const descriptionId = React.useId();

  const selectedOption = React.useMemo(() => {
    const port = liftConstraint(obfuscationSettings.shadowsocksSettings.port);
    const value = port !== 'any' ? 'custom' : null;

    return {
      port,
      value,
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
    <SettingsListbox value={selectedOption.value} onValueChange={setShadowsocksPort}>
      <SettingsListbox.Header>
        <SettingsListbox.Header.Item>
          <SettingsListbox.Header.Item.Label>
            {
              // TRANSLATORS: The title for the WireGuard port selector.
              messages.pgettext('wireguard-settings-view', 'Port')
            }
          </SettingsListbox.Header.Item.Label>
        </SettingsListbox.Header.Item>
      </SettingsListbox.Header>
      <SettingsListbox.Options>
        <SettingsListbox.Options.BaseOption value={null}>
          {messages.gettext('Automatic')}
        </SettingsListbox.Options.BaseOption>
        <SettingsListbox.Options.InputOption
          value="custom"
          defaultValue={
            selectedOption.value === 'custom' ? selectedOption.port?.toString() : undefined
          }
          validate={validateValue}
          format={removeNonNumericCharacters}>
          <SettingsListbox.Options.InputOption.Label>
            {messages.gettext('Custom')}
          </SettingsListbox.Options.InputOption.Label>
          <SettingsListItem.Item.ActionGroup>
            <SettingsListbox.Options.InputOption.Input
              aria-describedby={descriptionId}
              type="text"
              placeholder={messages.pgettext('wireguard-settings-view', 'Port')}
              maxLength={`${ALLOWED_RANGE[1]}`.length}
            />
          </SettingsListItem.Item.ActionGroup>
        </SettingsListbox.Options.InputOption>
      </SettingsListbox.Options>
      <SettingsListbox.Footer>
        <SettingsListbox.Footer.Text id={descriptionId}>
          {sprintf(
            // TRANSLATORS: Text describing the valid port range for a port selector.
            messages.pgettext('wireguard-settings-view', 'Valid range: %(min)s - %(max)s'),
            { min: ALLOWED_RANGE[0], max: ALLOWED_RANGE[1] },
          )}
        </SettingsListbox.Footer.Text>
      </SettingsListbox.Footer>
    </SettingsListbox>
  );
}

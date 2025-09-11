import React, { useCallback, useMemo } from 'react';

import { RelayProtocol, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelector } from '../../../../../redux/store';
import { SettingsListbox } from '../../../../settings-listbox';

export function TransportProtocolSetting() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const bridgeState = useSelector((state) => state.settings.bridgeState);

  const descriptionId = React.useId();

  const protocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.openvpn.protocol : 'any';
    return protocol === 'any' ? null : protocol;
  }, [relaySettings]);

  const onSelect = useCallback(
    async (protocol: RelayProtocol | null) => {
      await relaySettingsUpdater((settings) => {
        settings.openvpnConstraints.protocol = wrapConstraint(protocol);
        settings.openvpnConstraints.port = wrapConstraint<number>(undefined);
        return settings;
      });
    },
    [relaySettingsUpdater],
  );

  return (
    <SettingsListbox value={protocol} onValueChange={onSelect}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {messages.pgettext('openvpn-settings-view', 'Transport protocol')}
          </SettingsListbox.Label>
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <SettingsListbox.BaseOption value={null}>
          {messages.gettext('Automatic')}
        </SettingsListbox.BaseOption>
        <SettingsListbox.BaseOption value={'tcp'}>
          {messages.gettext('TCP')}
        </SettingsListbox.BaseOption>
        <SettingsListbox.BaseOption
          value={'udp'}
          disabled={bridgeState === 'on'}
          aria-describedby={bridgeState === 'on' ? descriptionId : undefined}>
          {messages.gettext('UDP')}
        </SettingsListbox.BaseOption>
      </SettingsListbox.Options>
      {bridgeState === 'on' && (
        <SettingsListbox.Footer>
          <SettingsListbox.Text id={descriptionId}>
            {formatHtml(
              // TRANSLATORS: This is used to instruct users how to make UDP mode
              // TRANSLATORS: available.
              messages.pgettext(
                'openvpn-settings-view',
                'To activate UDP, change <b>Bridge mode</b> to <b>Automatic</b> or <b>Off</b>.',
              ),
            )}
          </SettingsListbox.Text>
        </SettingsListbox.Footer>
      )}
    </SettingsListbox>
  );
}

import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useSelector } from '../../../../../redux/store';
import { SelectorItem } from '../../../../cell/Selector';
import { SettingsListbox } from '../../../../settings-listbox';

const UDP_PORTS = [1194, 1195, 1196, 1197, 1300, 1301, 1302];
const TCP_PORTS = [80, 443];

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}

export function OpenVpnPortSetting() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);

  const protocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.openvpn.protocol : 'any';
    return protocol === 'any' ? null : protocol;
  }, [relaySettings]);

  const port = useMemo(() => {
    const port = 'normal' in relaySettings ? relaySettings.normal.openvpn.port : 'any';
    return port === 'any' ? null : port;
  }, [relaySettings]);

  const onSelect = useCallback(
    async (port: number | null) => {
      await relaySettingsUpdater((settings) => {
        settings.openvpnConstraints.port = wrapConstraint(port);
        return settings;
      });
    },
    [relaySettingsUpdater],
  );

  const portItems = {
    udp: UDP_PORTS.map(mapPortToSelectorItem),
    tcp: TCP_PORTS.map(mapPortToSelectorItem),
  };

  if (protocol === null) {
    return null;
  }

  return (
    <SettingsListbox value={port} onValueChange={onSelect}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {sprintf(
              // TRANSLATORS: The title for the port selector section.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(portType)s - a selected protocol (either TCP or UDP)
              messages.pgettext('openvpn-settings-view', '%(portType)s port'),
              {
                portType: protocol.toUpperCase(),
              },
            )}
          </SettingsListbox.Label>
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <SettingsListbox.BaseOption value={null}>
          {messages.gettext('Automatic')}
        </SettingsListbox.BaseOption>
        {portItems[protocol].map((item) => (
          <SettingsListbox.BaseOption key={item.value} value={item.value}>
            {item.label}
          </SettingsListbox.BaseOption>
        ))}
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}

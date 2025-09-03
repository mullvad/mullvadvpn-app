import React, { useCallback, useMemo } from 'react';

import { RelayProtocol, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useScrollToListItem } from '../../../../../hooks';
import { Listbox } from '../../../../../lib/components/listbox/Listbox';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelector } from '../../../../../redux/store';
import { DefaultListboxOption } from '../../../../default-listbox-option';

export function TransportProtocolSetting() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const bridgeState = useSelector((state) => state.settings.bridgeState);
  const scrollToListItem = useScrollToListItem();

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
    <Listbox animation={scrollToListItem?.animation} value={protocol} onValueChange={onSelect}>
      <Listbox.Item>
        <Listbox.Content>
          <Listbox.Label>
            {messages.pgettext('openvpn-settings-view', 'Transport protocol')}
          </Listbox.Label>
        </Listbox.Content>
      </Listbox.Item>
      <Listbox.Options>
        <DefaultListboxOption value={null}>{messages.gettext('Automatic')}</DefaultListboxOption>
        <DefaultListboxOption value={'tcp'}>{messages.gettext('TCP')}</DefaultListboxOption>
        <DefaultListboxOption
          value={'udp'}
          disabled={bridgeState === 'on'}
          aria-describedby={bridgeState === 'on' ? descriptionId : undefined}>
          {messages.gettext('UDP')}
        </DefaultListboxOption>
      </Listbox.Options>
      {bridgeState === 'on' && (
        <Listbox.Footer>
          <Listbox.Text id={descriptionId}>
            {formatHtml(
              // TRANSLATORS: This is used to instruct users how to make UDP mode
              // TRANSLATORS: available.
              messages.pgettext(
                'openvpn-settings-view',
                'To activate UDP, change <b>Bridge mode</b> to <b>Automatic</b> or <b>Off</b>.',
              ),
            )}
          </Listbox.Text>
        </Listbox.Footer>
      )}
    </Listbox>
  );
}

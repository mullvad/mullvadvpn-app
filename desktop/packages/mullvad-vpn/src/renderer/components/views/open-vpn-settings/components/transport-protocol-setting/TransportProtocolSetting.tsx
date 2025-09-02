import { useCallback, useMemo } from 'react';

import { RelayProtocol, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { formatHtml } from '../../../../../lib/html-formatter';
import { useSelector } from '../../../../../redux/store';
import { AriaDescription, AriaInputGroup } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';
import Selector, { SelectorItem } from '../../../../cell/Selector';
import { StyledSelectorContainer } from '../../OpenVpnSettingsView';

export function TransportProtocolSetting() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const bridgeState = useSelector((state) => state.settings.bridgeState);

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

  const items: SelectorItem<RelayProtocol>[] = useMemo(
    () => [
      {
        label: messages.gettext('TCP'),
        value: 'tcp',
      },
      {
        label: messages.gettext('UDP'),
        value: 'udp',
        disabled: bridgeState === 'on',
      },
    ],
    [bridgeState],
  );

  return (
    <StyledSelectorContainer>
      <AriaInputGroup>
        <Selector
          title={messages.pgettext('openvpn-settings-view', 'Transport protocol')}
          items={items}
          value={protocol}
          onSelect={onSelect}
          automaticValue={null}
        />
        {bridgeState === 'on' && (
          <Cell.CellFooter>
            <AriaDescription>
              <Cell.CellFooterText>
                {formatHtml(
                  // TRANSLATORS: This is used to instruct users how to make UDP mode
                  // TRANSLATORS: available.
                  messages.pgettext(
                    'openvpn-settings-view',
                    'To activate UDP, change <b>Bridge mode</b> to <b>Automatic</b> or <b>Off</b>.',
                  ),
                )}
              </Cell.CellFooterText>
            </AriaDescription>
          </Cell.CellFooter>
        )}
      </AriaInputGroup>
    </StyledSelectorContainer>
  );
}

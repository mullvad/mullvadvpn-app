import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../../../../../shared/constants';
import { IpVersion, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useSelector } from '../../../../../redux/store';
import { AriaDescription, AriaInputGroup } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';
import Selector, { SelectorItem } from '../../../../cell/Selector';

const StyledSelectorContainer = styled.div({
  flex: 0,
});

export function IpVersionSetting() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const ipVersion = useMemo(() => {
    const ipVersion = 'normal' in relaySettings ? relaySettings.normal.wireguard.ipVersion : 'any';
    return ipVersion === 'any' ? null : ipVersion;
  }, [relaySettings]);

  const ipVersionItems: SelectorItem<IpVersion>[] = useMemo(
    () => [
      {
        label: messages.gettext('IPv4'),
        value: 'ipv4',
      },
      {
        label: messages.gettext('IPv6'),
        value: 'ipv6',
      },
    ],
    [],
  );

  const setIpVersion = useCallback(
    async (ipVersion: IpVersion | null) => {
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.ipVersion = wrapConstraint(ipVersion);
          return settings;
        });
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettingsUpdater],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          // TRANSLATORS: The title for the WireGuard IP version selector.
          title={messages.pgettext('wireguard-settings-view', 'IP version')}
          items={ipVersionItems}
          value={ipVersion}
          onSelect={setIpVersion}
          automaticValue={null}
        />
      </StyledSelectorContainer>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {sprintf(
              // TRANSLATORS: The hint displayed below the WireGuard IP version selector.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
              messages.pgettext(
                'wireguard-settings-view',
                'This allows access to %(wireguard)s for devices that only support IPv6.',
              ),
              { wireguard: strings.wireguard },
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';

import { strings, urls } from '../../../../../../shared/constants';
import { TunnelProtocol } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useTunnelProtocol } from '../../../../../lib/relay-settings-hooks';
import { useSelector } from '../../../../../redux/store';
import { AriaDescription, AriaInputGroup } from '../../../../AriaGroup';
import * as Cell from '../../../../cell';
import Selector, { SelectorItem } from '../../../../cell/Selector';
import { ExternalLink } from '../../../../ExternalLink';

export function TunnelProtocolSetting() {
  const tunnelProtocol = useTunnelProtocol();

  const relaySettingsUpdater = useRelaySettingsUpdater();

  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);
  const openVpnDisabled = daita || multihop || quantumResistant;

  const featuresToDisableForOpenVpn = [];
  if (daita) {
    featuresToDisableForOpenVpn.push(strings.daita);
  }
  if (multihop) {
    featuresToDisableForOpenVpn.push(messages.pgettext('wireguard-settings-view', 'Multihop'));
  }
  if (quantumResistant) {
    featuresToDisableForOpenVpn.push(
      messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel'),
    );
  }

  const setTunnelProtocol = useCallback(
    async (tunnelProtocol: TunnelProtocol) => {
      try {
        await relaySettingsUpdater((settings) => ({
          ...settings,
          tunnelProtocol,
        }));
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update tunnel protocol constraints', error.message);
      }
    },
    [relaySettingsUpdater],
  );

  const tunnelProtocolItems: Array<SelectorItem<TunnelProtocol>> = useMemo(
    () => [
      {
        label: strings.wireguard,
        value: 'wireguard',
      },
      {
        label: strings.openvpn,
        value: 'openvpn',
        disabled: openVpnDisabled,
      },
    ],
    [openVpnDisabled],
  );

  return (
    <AriaInputGroup>
      <Selector
        title={messages.pgettext('vpn-settings-view', 'Tunnel protocol')}
        items={tunnelProtocolItems}
        value={tunnelProtocol}
        onSelect={setTunnelProtocol}
      />
      {openVpnDisabled && (
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {sprintf(
                messages.pgettext(
                  'vpn-settings-view',
                  'To select %(openvpn)s, please disable these settings: %(featureList)s.',
                ),
                { openvpn: strings.openvpn, featureList: featuresToDisableForOpenVpn.join(', ') },
              )}
            </Cell.CellFooterText>
          </AriaDescription>
        </Cell.CellFooter>
      )}
      {tunnelProtocol === 'openvpn' && (
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {sprintf(
                // TRANSLATORS: Footer text for tunnel protocol selector when OpenVPN is selected.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(openvpn)s - Will be replaced with OpenVPN
                messages.pgettext(
                  'vpn-settings-view',
                  'Attention: We are removing support for %(openVpn)s.',
                ),
                { openVpn: strings.openvpn },
              )}{' '}
            </Cell.CellFooterText>
          </AriaDescription>
          <ExternalLink variant="labelTiny" to={urls.removingOpenVpnBlog}>
            <ExternalLink.Text>
              {sprintf(
                // TRANSLATORS: Link in tunnel protocol selector footer to blog post
                // TRANSLATORS: about OpenVPN support ending.
                messages.pgettext('vpn-settings-view', 'Read more'),
              )}
            </ExternalLink.Text>
            <ExternalLink.Icon icon="external" size="small" />
          </ExternalLink>
        </Cell.CellFooter>
      )}
    </AriaInputGroup>
  );
}

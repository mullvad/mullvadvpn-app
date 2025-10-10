import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useSelector } from '../../../../../redux/store';
import { SettingsToggleListItem } from '../../../../settings-toggle-list-item';

export function MultihopSetting() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;
  const unavailable = !('normal' in relaySettings);

  const setMultihop = useCallback(
    async (enabled: boolean) => {
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.useMultihop = enabled;
          return settings;
        });
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update WireGuard multihop settings', error.message);
      }
    },
    [relaySettingsUpdater],
  );

  return (
    <>
      <SettingsToggleListItem
        anchorId="multihop-setting"
        disabled={unavailable}
        checked={multihop && !unavailable}
        onCheckedChange={setMultihop}
        description={unavailable ? featureUnavailableMessage() : undefined}>
        <SettingsToggleListItem.Label>{messages.gettext('Enable')}</SettingsToggleListItem.Label>
        <SettingsToggleListItem.Switch />
      </SettingsToggleListItem>
    </>
  );
}

function featureUnavailableMessage() {
  const tunnelProtocol = messages.pgettext('vpn-settings-view', 'Tunnel protocol');
  const multihop = messages.pgettext('wireguard-settings-view', 'Multihop');

  return sprintf(
    messages.pgettext(
      // TRANSLATORS: Informs the user that the feature is only available when WireGuard
      // TRANSLATORS: is selected.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(wireguard)s - will be replaced with WireGuard
      // TRANSLATORS: %(tunnelProtocol)s - the name of the tunnel protocol setting
      // TRANSLATORS: %(setting)s - the name of the setting
      'wireguard-settings-view',
      'Switch to “%(wireguard)s” in Settings > %(tunnelProtocol)s to make %(setting)s available.',
    ),
    { wireguard: strings.wireguard, tunnelProtocol, setting: multihop },
  );
}

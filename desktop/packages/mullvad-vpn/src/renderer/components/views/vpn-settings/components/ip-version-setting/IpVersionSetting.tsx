import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { IpVersion, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useSelector } from '../../../../../redux/store';
import { SettingsListbox } from '../../../../settings-listbox';

export function IpVersionSetting() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const ipVersion = useMemo(() => {
    const ipVersion = 'normal' in relaySettings ? relaySettings.normal.wireguard.ipVersion : 'any';
    return ipVersion === 'any' ? null : ipVersion;
  }, [relaySettings]);

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
    <SettingsListbox value={ipVersion} onValueChange={setIpVersion}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard IP version selector.
              messages.pgettext('wireguard-settings-view', 'IP version')
            }
          </SettingsListbox.Label>
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <SettingsListbox.BaseOption value={null}>
          {messages.gettext('Automatic')}
        </SettingsListbox.BaseOption>
        <SettingsListbox.BaseOption value={'ipv4'}>
          {messages.gettext('IPv4')}
        </SettingsListbox.BaseOption>
        <SettingsListbox.BaseOption value={'ipv6'}>
          {messages.gettext('IPv6')}
        </SettingsListbox.BaseOption>
      </SettingsListbox.Options>
      <SettingsListbox.Footer>
        <SettingsListbox.Text>
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
        </SettingsListbox.Text>
      </SettingsListbox.Footer>
    </SettingsListbox>
  );
}

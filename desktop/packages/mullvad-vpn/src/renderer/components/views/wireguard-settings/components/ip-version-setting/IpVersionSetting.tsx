import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../../../shared/constants';
import { IpVersion, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useScrollToListItem } from '../../../../../hooks';
import { Listbox } from '../../../../../lib/components/listbox/Listbox';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useSelector } from '../../../../../redux/store';
import { DefaultListboxOption } from '../../../../default-listbox-option';

export function IpVersionSetting() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const ipVersion = useMemo(() => {
    const ipVersion = 'normal' in relaySettings ? relaySettings.normal.wireguard.ipVersion : 'any';
    return ipVersion === 'any' ? null : ipVersion;
  }, [relaySettings]);

  const scrollToAnchor = useScrollToListItem();

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
    <Listbox value={ipVersion} onValueChange={setIpVersion} animation={scrollToAnchor?.animation}>
      <Listbox.Item>
        <Listbox.Content>
          <Listbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard IP version selector.
              messages.pgettext('wireguard-settings-view', 'IP version')
            }
          </Listbox.Label>
        </Listbox.Content>
      </Listbox.Item>
      <Listbox.Options>
        <DefaultListboxOption value={null}>{messages.gettext('Automatic')}</DefaultListboxOption>
        <DefaultListboxOption value={'ipv4'}>{messages.gettext('IPv4')}</DefaultListboxOption>
        <DefaultListboxOption value={'ipv6'}>{messages.gettext('IPv6')}</DefaultListboxOption>
      </Listbox.Options>
      <Listbox.Footer>
        <Listbox.Text>
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
        </Listbox.Text>
      </Listbox.Footer>
    </Listbox>
  );
}

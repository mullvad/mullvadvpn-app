import { useCallback, useMemo } from 'react';

import { IpVersion, wrapConstraint } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import log from '../../../../../../shared/logging';
import { useRelaySettingsUpdater } from '../../../../../lib/constraint-updater';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
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
              // TRANSLATORS: Title for device IP version setting.
              messages.pgettext('wireguard-settings-view', 'Device IP version')
            }
          </SettingsListbox.Label>
          <SettingsListbox.Group $gap="medium">
            <InfoButton>
              <ModalMessage>
                {
                  // TRANSLATORS: A description for the setting Device IP version,
                  // TRANSLATORS: explaining how the user can configure the setting.
                  messages.pgettext(
                    'vpn-settings-view',
                    'This feature allows you to choose whether to use only IPv4, only IPv6, or allow the app to automatically decide the best option when connecting to a server.',
                  )
                }
              </ModalMessage>
              <ModalMessage>
                {
                  // TRANSLATORS: A complimentary description for the setting Device IP version,
                  // TRANSLATORS: explaining why the user might want to configure the setting.
                  messages.pgettext(
                    'vpn-settings-view',
                    'It can be useful when you are aware of problems caused by a certain IP version.',
                  )
                }
              </ModalMessage>
            </InfoButton>
          </SettingsListbox.Group>
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
    </SettingsListbox>
  );
}

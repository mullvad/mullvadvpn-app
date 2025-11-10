import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListItem } from '../../../../settings-list-item';
import { EnableIpv6Switch } from './EnableIpv6Switch';

export function EnableIpv6Setting() {
  const descriptionId = React.useId();
  return (
    <SettingsListItem>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <EnableIpv6Switch>
            <EnableIpv6Switch.Label variant="titleMedium">
              {
                // TRANSLATORS: Title of in-tunnel IPv6 setting.
                messages.pgettext('vpn-settings-view', 'In-tunnel IPv6')
              }
            </EnableIpv6Switch.Label>
            <SettingsListItem.Group>
              <InfoButton>
                <ModalMessage>
                  {messages.pgettext(
                    'vpn-settings-view',
                    'When this feature is enabled, IPv6 can be used alongside IPv4 in the VPN tunnel to communicate with internet services.',
                  )}
                </ModalMessage>
                <ModalMessage>
                  {messages.pgettext(
                    'vpn-settings-view',
                    'IPv4 is always enabled and the majority of websites and applications use this protocol. We do not recommend enabling IPv6 unless you know you need it.',
                  )}
                </ModalMessage>
              </InfoButton>

              <EnableIpv6Switch.Trigger aria-describedby={descriptionId}>
                <EnableIpv6Switch.Thumb />
              </EnableIpv6Switch.Trigger>
            </SettingsListItem.Group>
          </EnableIpv6Switch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Text id={descriptionId}>
          {
            // TRANSLATORS: Description of in-tunnel IPv6 setting.
            messages.pgettext(
              'vpn-settings-view',
              'Enable to allow IPv6 traffic through the tunnel.',
            )
          }
        </SettingsListItem.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}

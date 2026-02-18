import React from 'react';

import { messages } from '../../../../../shared/gettext';
import InfoButton from '../../../../components/InfoButton';
import { ModalMessage } from '../../../../components/Modal';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { EnableIpv6Switch } from '../enable-ipv6-switch/EnableIpv6Switch';

export type EnableIpv6SettingProps = Omit<ListItemProps, 'children'>;

export function EnableIpv6Setting(props: EnableIpv6SettingProps) {
  const descriptionId = React.useId();
  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Item>
        <EnableIpv6Switch descriptionId={descriptionId}>
          <EnableIpv6Switch.Label>
            {
              // TRANSLATORS: Title of in-tunnel IPv6 setting.
              messages.pgettext('vpn-settings-view', 'In-tunnel IPv6')
            }
          </EnableIpv6Switch.Label>
          <SettingsListItem.ActionGroup>
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

            <EnableIpv6Switch.Input />
          </SettingsListItem.ActionGroup>
        </EnableIpv6Switch>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.FooterText id={descriptionId}>
          {
            // TRANSLATORS: Description of in-tunnel IPv6 setting.
            messages.pgettext(
              'vpn-settings-view',
              'Enable to allow IPv6 traffic through the tunnel.',
            )
          }
        </SettingsListItem.FooterText>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}

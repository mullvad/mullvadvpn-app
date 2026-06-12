import React from 'react';

import { messages } from '../../../../../shared/gettext';
import { Info } from '../../../../components/info';
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
          <SettingsListItem.Item.ActionGroup>
            <Info>
              <Info.Button />
              <Info.Dialog>
                <Info.Dialog.Text>
                  {messages.pgettext(
                    'vpn-settings-view',
                    'When this feature is enabled, IPv6 can be used alongside IPv4 in the VPN tunnel to communicate with internet services.',
                  )}
                </Info.Dialog.Text>
                <Info.Dialog.Text>
                  {messages.pgettext(
                    'vpn-settings-view',
                    'IPv4 is always enabled and the majority of websites and applications use this protocol. We do not recommend enabling IPv6 unless you know you need it.',
                  )}
                </Info.Dialog.Text>
              </Info.Dialog>
            </Info>

            <EnableIpv6Switch.Input />
          </SettingsListItem.Item.ActionGroup>
        </EnableIpv6Switch>
      </SettingsListItem.Item>
      <SettingsListItem.Footer>
        <SettingsListItem.Footer.Text id={descriptionId}>
          {
            // TRANSLATORS: Description of in-tunnel IPv6 setting.
            messages.pgettext(
              'vpn-settings-view',
              'Enable to allow IPv6 traffic through the tunnel.',
            )
          }
        </SettingsListItem.Footer.Text>
      </SettingsListItem.Footer>
    </SettingsListItem>
  );
}

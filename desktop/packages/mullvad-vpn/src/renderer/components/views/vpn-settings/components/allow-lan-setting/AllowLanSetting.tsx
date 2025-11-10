import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { spacings } from '../../../../../lib/foundations';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListItem } from '../../../../settings-list-item';
import { AllowLanSwitch } from './AllowLanSwitch';

const LanIpRanges = styled.ul({
  listStyle: 'disc outside',
  marginLeft: spacings.large,
});

export function AllowLanSetting() {
  return (
    <SettingsListItem anchorId="allow-lan-setting">
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <AllowLanSwitch>
            <AllowLanSwitch.Label variant="titleMedium">
              {messages.pgettext('vpn-settings-view', 'Local network sharing')}
            </AllowLanSwitch.Label>
            <SettingsListItem.Group>
              <InfoButton>
                <ModalMessage>
                  {messages.pgettext(
                    'vpn-settings-view',
                    'This feature allows access to other devices on the local network, such as for sharing, printing, streaming, etc.',
                  )}
                </ModalMessage>
                <ModalMessage>
                  {messages.pgettext(
                    'vpn-settings-view',
                    'It does this by allowing network communication outside the tunnel to local multicast and broadcast ranges as well as to and from these private IP ranges:',
                  )}
                  <LanIpRanges>
                    <li>10.0.0.0/8</li>
                    <li>172.16.0.0/12</li>
                    <li>192.168.0.0/16</li>
                    <li>169.254.0.0/16</li>
                    <li>fe80::/10</li>
                    <li>fc00::/7</li>
                  </LanIpRanges>
                </ModalMessage>
              </InfoButton>

              <AllowLanSwitch.Trigger>
                <AllowLanSwitch.Thumb />
              </AllowLanSwitch.Trigger>
            </SettingsListItem.Group>
          </AllowLanSwitch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}

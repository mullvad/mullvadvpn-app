import styled from 'styled-components';

import { messages } from '../../../../../shared/gettext';
import InfoButton from '../../../../components/InfoButton';
import { ModalMessage } from '../../../../components/Modal';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { ListItemProps } from '../../../../lib/components/list-item';
import { spacings } from '../../../../lib/foundations';
import { AllowLanSwitch } from '../allow-lan-switch/AllowLanSwitch';

export type AllowLanSettingProps = Omit<ListItemProps, 'children'>;

const LanIpRanges = styled.ul({
  listStyle: 'disc outside',
  marginLeft: spacings.large,
});

export function AllowLanSetting(props: AllowLanSettingProps) {
  return (
    <SettingsListItem anchorId="allow-lan-setting" {...props}>
      <SettingsListItem.Item>
        <AllowLanSwitch>
          <AllowLanSwitch.Label>
            {messages.pgettext('vpn-settings-view', 'Local network sharing')}
          </AllowLanSwitch.Label>
          <SettingsListItem.ActionGroup>
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
          </SettingsListItem.ActionGroup>
        </AllowLanSwitch>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}

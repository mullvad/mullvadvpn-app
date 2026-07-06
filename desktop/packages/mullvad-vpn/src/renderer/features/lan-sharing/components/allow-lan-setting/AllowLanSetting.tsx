import { useCallback } from 'react';
import styled from 'styled-components';

import { urls } from '../../../../../shared/constants';
import { messages } from '../../../../../shared/gettext';
import CustomScrollbars from '../../../../components/CustomScrollbars';
import { Info } from '../../../../components/info';
import { SettingsListItem } from '../../../../components/settings-list-item';
import { useAppContext } from '../../../../context';
import { Flex } from '../../../../lib/components';
import { ListItemProps } from '../../../../lib/components/list-item';
import { spacings } from '../../../../lib/foundations';
import { AllowLanSwitch } from '../allow-lan-switch/AllowLanSwitch';

export type AllowLanSettingProps = Omit<ListItemProps, 'children'>;

const LanIpRanges = styled.ul({
  listStyle: 'disc outside',
  marginLeft: spacings.large,
});

const StyledCustomScrollbars = styled(CustomScrollbars)({
  paddingRight: '12px',
  marginRight: '-16px',
  marginLeft: '-4px',
});

const StyledFlex = styled(Flex)`
  padding-left: 4px;
  padding-right: 4px;
  padding-bottom: 4px;
`;

export function AllowLanSetting(props: AllowLanSettingProps) {
  const { openUrl } = useAppContext();

  const openGuide = useCallback(() => openUrl(urls.lanShare), [openUrl]);

  return (
    <SettingsListItem anchorId="allow-lan-setting" {...props}>
      <SettingsListItem.Item>
        <AllowLanSwitch>
          <AllowLanSwitch.Label>
            {messages.pgettext('vpn-settings-view', 'Local network sharing')}
          </AllowLanSwitch.Label>
          <SettingsListItem.Item.ActionGroup>
            <Info>
              <Info.Button />
              <Info.Dialog>
                <StyledCustomScrollbars>
                  <StyledFlex flexDirection="column" gap="medium">
                    <Info.Dialog.Text>
                      {messages.pgettext(
                        'vpn-settings-view',
                        'This feature allows access to other devices on the local network, such as for sharing, printing, streaming, etc.',
                      )}
                    </Info.Dialog.Text>
                    <Info.Dialog.Text>
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
                    </Info.Dialog.Text>
                    <Info.Dialog.Text>
                      {messages.pgettext(
                        'vpn-settings-view',
                        'If you can’t connect you can try using the IP address instead of the host name. If you want to connect to a subnet or a private network address range, you can follow our guide to add a static route.',
                      )}
                    </Info.Dialog.Text>
                    <Info.Dialog.Button onClick={openGuide}>
                      <Info.Dialog.Button.Text>{messages.gettext('Guide')}</Info.Dialog.Button.Text>
                      <Info.Dialog.Button.Icon icon="external" />
                    </Info.Dialog.Button>
                  </StyledFlex>
                </StyledCustomScrollbars>
              </Info.Dialog>
            </Info>
            <AllowLanSwitch.Input />
          </SettingsListItem.Item.ActionGroup>
        </AllowLanSwitch>
      </SettingsListItem.Item>
    </SettingsListItem>
  );
}

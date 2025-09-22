import { useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import * as Cell from '../../cell';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import {
  BridgeModeSetting,
  MssFixSetting,
  OpenVpnPortSetting,
  TransportProtocolSetting,
} from './components';

export enum BridgeModeAvailability {
  available,
  blockedDueToTunnelProtocol,
  blockedDueToTransportProtocol,
}

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

export function OpenVpnSettingsView() {
  const { pop } = useHistory();

  const relaySettings = useSelector((state) => state.settings.relaySettings);

  const protocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.openvpn.protocol : undefined;
    return protocol === 'any' ? undefined : protocol;
  }, [relaySettings]);

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={sprintf(
                // TRANSLATORS: Title label in navigation bar
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(openvpn)s - Will be replaced with "OpenVPN"
                messages.pgettext('openvpn-settings-nav', '%(openvpn)s settings'),
                { openvpn: strings.openvpn },
              )}
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {sprintf(
                    // TRANSLATORS: %(openvpn)s will be replaced with "OpenVPN"
                    messages.pgettext('openvpn-settings-view', '%(openvpn)s settings'),
                    {
                      openvpn: strings.openvpn,
                    },
                  )}
                </HeaderTitle>
              </SettingsHeader>

              <Cell.Group>
                <TransportProtocolSetting />
              </Cell.Group>

              {protocol ? (
                <Cell.Group>
                  <OpenVpnPortSetting />
                </Cell.Group>
              ) : undefined}

              <Cell.Group>
                <BridgeModeSetting />
              </Cell.Group>

              <Cell.Group>
                <MssFixSetting />
              </Cell.Group>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

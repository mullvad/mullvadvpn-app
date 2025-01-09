import { useCallback, useMemo } from 'react';
import styled from 'styled-components';

import { liftConstraint, LiftedConstraint, wrapConstraint } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import { AriaInputGroup } from './AriaGroup';
import * as Cell from './cell';
import Selector, { SelectorItem } from './cell/Selector';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalMessage } from './Modal';
import { NavigationContainer } from './NavigationContainer';
import { NavigationScrollbars } from './NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const UDP2TCP_PORTS = [80, 5001];

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

const StyledSelectorContainer = styled.div({
  flex: 0,
});

export default function UdpOverTcp() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('wireguard-settings-nav', 'UDP-over-TCP')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'UDP-over-TCP')}
                </HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <Cell.Group>
                  <Udp2tcpPortSetting />
                </Cell.Group>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function Udp2tcpPortSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  const port = liftConstraint(obfuscationSettings.udp2tcpSettings.port);
  const portItems: SelectorItem<number>[] = useMemo(
    () => UDP2TCP_PORTS.map(mapPortToSelectorItem),
    [],
  );

  const selectPort = useCallback(
    async (port: LiftedConstraint<number>) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        udp2tcpSettings: {
          ...obfuscationSettings.udp2tcpSettings,
          port: wrapConstraint(port),
        },
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          // TRANSLATORS: The title for the UDP-over-TCP port selector.
          title={messages.pgettext('wireguard-settings-view', 'UDP-over-TCP port')}
          details={
            <ModalMessage>
              {messages.pgettext(
                'wireguard-settings-view',
                'Which TCP port the UDP-over-TCP obfuscation protocol should connect to on the VPN server.',
              )}
            </ModalMessage>
          }
          items={portItems}
          value={port}
          onSelect={selectPort}
          thinTitle
          automaticValue={'any' as const}
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

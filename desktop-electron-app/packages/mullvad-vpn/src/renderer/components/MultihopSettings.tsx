import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../config.json';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { useRelaySettingsUpdater } from '../lib/constraint-updater';
import { useHistory } from '../lib/history';
import { useSelector } from '../redux/store';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import { StyledIllustration } from './DaitaSettings';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

export default function MultihopSettings() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>
                  {messages.pgettext('wireguard-settings-view', 'Multihop')}
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'Multihop')}
                </HeaderTitle>
                <HeaderSubTitle>
                  <StyledIllustration src="../../assets/images/multihop-illustration.svg" />
                  {messages.pgettext(
                    'wireguard-settings-view',
                    'Multihop routes your traffic into one WireGuard server and out another, making it harder to trace. This results in increased latency but increases anonymity online.',
                  )}
                </HeaderSubTitle>
              </SettingsHeader>

              <StyledContent>
                <Cell.Group>
                  <MultihopSetting />
                </Cell.Group>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function MultihopSetting() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const relaySettingsUpdater = useRelaySettingsUpdater();

  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;
  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  const setMultihop = useCallback(
    async (enabled: boolean) => {
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.useMultihop = enabled;
          return settings;
        });
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update WireGuard multihop settings', error.message);
      }
    },
    [relaySettingsUpdater],
  );

  return (
    <>
      <AriaInputGroup>
        <Cell.Container disabled={unavailable}>
          <AriaLabel>
            <Cell.InputLabel>{messages.gettext('Enable')}</Cell.InputLabel>
          </AriaLabel>
          <AriaInput>
            <Cell.Switch isOn={multihop && !unavailable} onChange={setMultihop} />
          </AriaInput>
        </Cell.Container>
        {unavailable ? (
          <Cell.CellFooter>
            <AriaDescription>
              <Cell.CellFooterText>{featureUnavailableMessage()}</Cell.CellFooterText>
            </AriaDescription>
          </Cell.CellFooter>
        ) : null}
      </AriaInputGroup>
    </>
  );
}

function featureUnavailableMessage() {
  const automatic = messages.gettext('Automatic');
  const tunnelProtocol = messages.pgettext('vpn-settings-view', 'Tunnel protocol');
  const multihop = messages.pgettext('wireguard-settings-view', 'Multihop');

  return sprintf(
    messages.pgettext(
      'wireguard-settings-view',
      'Switch to “%(wireguard)s” or “%(automatic)s” in Settings > %(tunnelProtocol)s to make %(setting)s available.',
    ),
    { wireguard: strings.wireguard, automatic, tunnelProtocol, setting: multihop },
  );
}

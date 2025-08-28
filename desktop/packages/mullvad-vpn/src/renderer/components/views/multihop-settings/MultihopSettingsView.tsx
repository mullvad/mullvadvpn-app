import { useCallback, useRef } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import log from '../../../../shared/logging';
import { useScrollToListItem } from '../../../hooks';
import { Flex } from '../../../lib/components';
import { useRelaySettingsUpdater } from '../../../lib/constraint-updater';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import { AriaDescription } from '../../AriaGroup';
import * as Cell from '../../cell';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from '../../SettingsHeader';
import { ToggleListItem } from '../../toggle-list-item';

const PATH_PREFIX = process.env.NODE_ENV === 'development' ? '../' : '';

const StyledIllustration = styled.img({
  width: '100%',
  padding: '8px 0 8px',
});

export function MultihopSettingsView() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title={messages.pgettext('wireguard-settings-view', 'Multihop')} />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'Multihop')}
                </HeaderTitle>
                <HeaderSubTitle>
                  <StyledIllustration
                    src={`${PATH_PREFIX}assets/images/multihop-illustration.svg`}
                  />
                  {messages.pgettext(
                    'wireguard-settings-view',
                    'Multihop routes your traffic into one WireGuard server and out another, making it harder to trace. This results in increased latency but increases anonymity online.',
                  )}
                </HeaderSubTitle>
              </SettingsHeader>

              <Flex $flexDirection="column" $flex={1}>
                <Cell.Group>
                  <MultihopSetting />
                </Cell.Group>
              </Flex>
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

  const id = 'multihop-setting';
  const ref = useRef<HTMLDivElement>(null);
  const scrollTo = useScrollToListItem(ref, id);

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
      <ToggleListItem
        ref={ref}
        animation={scrollTo?.animation}
        disabled={unavailable}
        checked={multihop && !unavailable}
        onCheckedChange={setMultihop}>
        <ToggleListItem.Label>{messages.gettext('Enable')}</ToggleListItem.Label>
        <ToggleListItem.Switch />
      </ToggleListItem>
      {unavailable ? (
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>{featureUnavailableMessage()}</Cell.CellFooterText>
          </AriaDescription>
        </Cell.CellFooter>
      ) : null}
    </>
  );
}

function featureUnavailableMessage() {
  const tunnelProtocol = messages.pgettext('vpn-settings-view', 'Tunnel protocol');
  const multihop = messages.pgettext('wireguard-settings-view', 'Multihop');

  return sprintf(
    messages.pgettext(
      // TRANSLATORS: Informs the user that the feature is only available when WireGuard
      // TRANSLATORS: is selected.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(wireguard)s - will be replaced with WireGuard
      // TRANSLATORS: %(tunnelProtocol)s - the name of the tunnel protocol setting
      // TRANSLATORS: %(setting)s - the name of the setting
      'wireguard-settings-view',
      'Switch to “%(wireguard)s” in Settings > %(tunnelProtocol)s to make %(setting)s available.',
    ),
    { wireguard: strings.wireguard, tunnelProtocol, setting: multihop },
  );
}

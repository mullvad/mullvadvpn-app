import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../config.json';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { useHistory } from '../lib/history';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import InfoButton from './InfoButton';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import PageSlider from './PageSlider';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import { SmallButton, SmallButtonColor } from './SmallButton';

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

const StyledHeaderSubTitle = styled(HeaderSubTitle)({
  display: 'inline-block',

  '&&:not(:last-child)': {
    paddingBottom: '18px',
  },
});

const EnableFooter = styled(Cell.CellFooter)({
  paddingBottom: '16px',
});

const StyledIllustration = styled.img({
  width: '100%',
  padding: '8px 0 8px',
});

export default function DaitaSettings() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>{strings.daita}</TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{strings.daita}</HeaderTitle>
                <PageSlider
                  content={[
                    <React.Fragment key="without-daita">
                      <StyledIllustration src="../../assets/images/daita-off-illustration.svg" />
                      <StyledHeaderSubTitle>
                        {sprintf(
                          messages.pgettext(
                            'wireguard-settings-view',
                            '%(daita)s (%(daitaFull)s) hides patterns in your encrypted VPN traffic.',
                          ),
                          { daita: strings.daita, daitaFull: strings.daitaFull },
                        )}
                      </StyledHeaderSubTitle>
                      <StyledHeaderSubTitle>
                        {messages.pgettext(
                          'wireguard-settings-view',
                          'If anyone is monitoring your connection, this makes it significantly harder for them to identify what websites you are visiting.',
                        )}
                      </StyledHeaderSubTitle>
                    </React.Fragment>,
                    <React.Fragment key="with-daita">
                      <StyledIllustration src="../../assets/images/daita-on-illustration.svg" />
                      <StyledHeaderSubTitle>
                        {messages.pgettext(
                          'wireguard-settings-view',
                          'It does this by carefully adding network noise and making all network packets the same size.',
                        )}
                      </StyledHeaderSubTitle>
                      <StyledHeaderSubTitle>
                        {messages.pgettext(
                          'wireguard-settings-view',
                          'Can only be used with WireGuard. Since this increases your total network traffic, be cautious if you have a limited data plan. It can also negatively impact your network speed.',
                        )}
                      </StyledHeaderSubTitle>
                    </React.Fragment>,
                  ]}
                />
              </SettingsHeader>

              <StyledContent>
                <Cell.Group>
                  <DaitaToggle />
                </Cell.Group>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function DaitaToggle() {
  const { setEnableDaita, setDaitaSmartRouting } = useAppContext();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const smartRouting = useSelector(
    (state) => state.settings.wireguard.daita?.smartRouting ?? false,
  );

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  const setDaita = useCallback((value: boolean) => {
    void setEnableDaita(value);
  }, []);

  const setSmartRouting = useCallback((value: boolean) => {
    if (value) {
      void setDaitaSmartRouting(value);
    } else {
      showConfirmationDialog();
    }
  }, []);

  const confirmDisableSmartRouting = useCallback(() => {
    void setDaitaSmartRouting(false);
    hideConfirmationDialog();
  }, []);

  return (
    <>
      <AriaInputGroup>
        <Cell.Container disabled={unavailable}>
          <AriaLabel>
            <Cell.InputLabel>{messages.gettext('Enable')}</Cell.InputLabel>
          </AriaLabel>
          <AriaInput>
            <Cell.Switch isOn={daita && !unavailable} onChange={setDaita} />
          </AriaInput>
        </Cell.Container>
        {unavailable ? (
          <EnableFooter>
            <AriaDescription>
              <Cell.CellFooterText>{featureUnavailableMessage()}</Cell.CellFooterText>
            </AriaDescription>
          </EnableFooter>
        ) : null}
      </AriaInputGroup>
      <AriaInputGroup>
        <Cell.Container disabled={!daita || unavailable}>
          <AriaLabel>
            <Cell.InputLabel>{messages.gettext('Smart routing')}</Cell.InputLabel>
          </AriaLabel>
          <InfoButton>
            <SmartRoutingModalMessage />
          </InfoButton>
          <AriaInput>
            <Cell.Switch isOn={smartRouting && !unavailable} onChange={setSmartRouting} />
          </AriaInput>
        </Cell.Container>
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {sprintf(
                messages.pgettext(
                  'vpn-settings-view',
                  'Makes it possible to use %(daita)s with any server and is automatically enabled.',
                ),
                { daita: strings.daita },
              )}
            </Cell.CellFooterText>
          </AriaDescription>
        </Cell.CellFooter>
      </AriaInputGroup>
      <ModalAlert
        isOpen={confirmationDialogVisible}
        type={ModalAlertType.caution}
        gridButtons={[
          <SmallButton
            key="confirm"
            onClick={confirmDisableSmartRouting}
            color={SmallButtonColor.blue}>
            {messages.gettext('Disable anyway')}
          </SmallButton>,
          <SmallButton key="cancel" onClick={hideConfirmationDialog} color={SmallButtonColor.blue}>
            {messages.pgettext('wireguard-settings-view', 'Use Smart routing')}
          </SmallButton>,
        ]}
        close={hideConfirmationDialog}>
        <ModalMessage>
          {sprintf(
            // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
            messages.pgettext(
              'wireguard-settings-view',
              'Not all our servers are %(daita)s-enabled. In order to use the internet, you might have to select a new location after disabling, or you can continue using %(daita)s with Smart routing.',
            ),
            { daita: strings.daita },
          )}
        </ModalMessage>
      </ModalAlert>
    </>
  );
}

export function SmartRoutingModalMessage() {
  return (
    <ModalMessage>
      {sprintf(
        messages.pgettext(
          'wireguard-settings-view',
          'Not all our servers are %(daita)s-enabled. Smart routing allows %(daita)s to be used at any location. It does this by using multihop in the background to route your traffic via the closest %(daita)s-enabled server first.',
        ),
        {
          daita: strings.daita,
        },
      )}
    </ModalMessage>
  );
}

function featureUnavailableMessage() {
  const automatic = messages.gettext('Automatic');
  const tunnelProtocol = messages.pgettext('vpn-settings-view', 'Tunnel protocol');

  return sprintf(
    messages.pgettext(
      'wireguard-settings-view',
      'Switch to “%(wireguard)s” or “%(automatic)s” in Settings > %(tunnelProtocol)s to make %(setting)s available.',
    ),
    { wireguard: strings.wireguard, automatic, tunnelProtocol, setting: strings.daita },
  );
}

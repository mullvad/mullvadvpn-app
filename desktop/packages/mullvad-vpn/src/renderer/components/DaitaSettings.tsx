import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../shared/constants';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { Flex } from '../lib/components';
import { Spacings } from '../lib/foundations';
import { useHistory } from '../lib/history';
import { useBoolean } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import InfoButton from './InfoButton';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';
import { NavigationContainer } from './NavigationContainer';
import { NavigationScrollbars } from './NavigationScrollbars';
import PageSlider from './PageSlider';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import { SmallButton, SmallButtonColor } from './SmallButton';

const StyledHeaderSubTitle = styled(HeaderSubTitle)({
  display: 'inline-block',
});

export const StyledIllustration = styled.img({
  width: '100%',
  padding: '8px 0 8px',
});

const StyledInfoButton = styled(InfoButton)({
  marginRight: Spacings.spacing5,
});

const PATH_PREFIX = process.env.NODE_ENV === 'development' ? '../' : '';

export default function DaitaSettings() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title={strings.daita} />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{strings.daita}</HeaderTitle>
                <PageSlider
                  content={[
                    <React.Fragment key="without-daita">
                      <StyledIllustration
                        src={`${PATH_PREFIX}assets/images/daita-off-illustration.svg`}
                      />
                      <Flex $flexDirection="column" $gap={Spacings.spacing5}>
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
                            'By using sophisticated AI it’s possible to analyze the traffic of data packets going in and out of your device (even if the traffic is encrypted).',
                          )}
                        </StyledHeaderSubTitle>
                        <StyledHeaderSubTitle>
                          {sprintf(
                            messages.pgettext(
                              'wireguard-settings-view',
                              'If an observer monitors these data packets, %(daita)s makes it significantly harder for them to identify which websites you are visiting or with whom you are communicating.',
                            ),
                            { daita: strings.daita },
                          )}
                        </StyledHeaderSubTitle>
                      </Flex>
                    </React.Fragment>,
                    <React.Fragment key="with-daita">
                      <StyledIllustration
                        src={`${PATH_PREFIX}assets/images/daita-on-illustration.svg`}
                      />
                      <Flex $flexDirection="column" $gap={Spacings.spacing5}>
                        <StyledHeaderSubTitle>
                          {sprintf(
                            messages.pgettext(
                              'wireguard-settings-view',
                              '%(daita)s does this by carefully adding network noise and making all network packets the same size.',
                            ),
                            { daita: strings.daita },
                          )}
                        </StyledHeaderSubTitle>
                        <StyledHeaderSubTitle>
                          {sprintf(
                            messages.pgettext(
                              'wireguard-settings-view',
                              'Not all our servers are %(daita)s-enabled. Therefore, we use multihop automatically to enable %(daita)s with any server.',
                            ),
                            { daita: strings.daita },
                          )}
                        </StyledHeaderSubTitle>
                        <StyledHeaderSubTitle>
                          {sprintf(
                            messages.pgettext(
                              'wireguard-settings-view',
                              'Attention: Be cautious if you have a limited data plan as this feature will increase your network traffic. This feature can only be used with %(wireguard)s.',
                            ),
                            { wireguard: strings.wireguard },
                          )}
                        </StyledHeaderSubTitle>
                      </Flex>
                    </React.Fragment>,
                  ]}
                />
              </SettingsHeader>
              <SettingsContainer>
                <Cell.Group>
                  <DaitaToggle />
                </Cell.Group>
              </SettingsContainer>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function DaitaToggle() {
  const { setEnableDaita, setDaitaDirectOnly } = useAppContext();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  const setDaita = useCallback(
    (value: boolean) => {
      void setEnableDaita(value);
    },
    [setEnableDaita],
  );

  const setDirectOnly = useCallback(
    (value: boolean) => {
      if (value) {
        showConfirmationDialog();
      } else {
        void setDaitaDirectOnly(value);
      }
    },
    [setDaitaDirectOnly, showConfirmationDialog],
  );

  const confirmEnableDirectOnly = useCallback(() => {
    void setDaitaDirectOnly(true);
    hideConfirmationDialog();
  }, [hideConfirmationDialog, setDaitaDirectOnly]);

  const directOnlyString = messages.gettext('Direct only');

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
      </AriaInputGroup>
      <AriaInputGroup>
        <Cell.Container disabled={!daita || unavailable}>
          <AriaLabel>
            <Cell.InputLabel>{directOnlyString}</Cell.InputLabel>
          </AriaLabel>
          <StyledInfoButton>
            <DirectOnlyModalMessage />
          </StyledInfoButton>
          <AriaInput>
            <Cell.Switch isOn={directOnly && !unavailable} onChange={setDirectOnly} />
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
      <ModalAlert
        isOpen={confirmationDialogVisible}
        type={ModalAlertType.caution}
        gridButtons={[
          <SmallButton
            key="confirm"
            onClick={confirmEnableDirectOnly}
            color={SmallButtonColor.blue}>
            {
              // TRANSLATORS: A toggle that refers to the setting "Direct only".
              messages.gettext('Enable direct only')
            }
          </SmallButton>,
          <SmallButton key="cancel" onClick={hideConfirmationDialog} color={SmallButtonColor.blue}>
            {messages.pgettext('wireguard-settings-view', 'Cancel')}
          </SmallButton>,
        ]}
        close={hideConfirmationDialog}>
        <ModalMessage>
          {sprintf(
            // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
            messages.pgettext(
              'wireguard-settings-view',
              'Not all our servers are %(daita)s-enabled. In order to use the internet, you might have to select a new location after enabling.',
            ),
            { daita: strings.daita },
          )}
        </ModalMessage>
      </ModalAlert>
    </>
  );
}

function DirectOnlyModalMessage() {
  const directOnlyString = messages.gettext('Direct only');

  return (
    <ModalMessage>
      {sprintf(
        messages.pgettext(
          'wireguard-settings-view',
          'By enabling “%(directOnly)s” you will have to manually select a server that is %(daita)s-enabled. This can cause you to end up in a blocked state until you have selected a compatible server in the “Select location” view.',
        ),
        {
          daita: strings.daita,
          directOnly: directOnlyString,
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

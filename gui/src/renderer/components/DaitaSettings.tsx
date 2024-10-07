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

export const StyledIllustration = styled.img({
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
                        {sprintf(
                          messages.pgettext(
                            'wireguard-settings-view',
                            'Not all our servers are %(daita)s-enabled. We use multihop automatically to use %(daita)s with any server.',
                          ),
                          { daita: strings.daita, daitaFull: strings.daitaFull },
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
  const { setEnableDaita, setDaitaDirectOnly } = useAppContext();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

  const unavailable =
    'normal' in relaySettings ? relaySettings.normal.tunnelProtocol === 'openvpn' : true;

  const setDaita = useCallback((value: boolean) => {
    void setEnableDaita(value);
  }, []);

  const setDirectOnly = useCallback((value: boolean) => {
    if (value) {
      showConfirmationDialog();
    } else {
      void setDaitaDirectOnly(value);
    }
  }, []);

  const confirmEnableDirectOnly = useCallback(() => {
    void setDaitaDirectOnly(true);
    hideConfirmationDialog();
  }, []);

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
            <Cell.InputLabel>{directOnlyString}</Cell.InputLabel>
          </AriaLabel>
          <InfoButton>
            <DirectOnlyModalMessage />
          </InfoButton>
          <AriaInput>
            <Cell.Switch isOn={directOnly && !unavailable} onChange={setDirectOnly} />
          </AriaInput>
        </Cell.Container>
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {sprintf(
                messages.pgettext(
                  'vpn-settings-view',
                  'Manually choose which %(daita)s-enabled server to use.',
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
            onClick={confirmEnableDirectOnly}
            color={SmallButtonColor.blue}>
            {sprintf(messages.gettext('Enable "%(directOnly)s"'), { directOnly: directOnlyString })}
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

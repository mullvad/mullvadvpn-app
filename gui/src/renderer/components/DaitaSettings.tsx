import { useCallback } from 'react';
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
  NavigationInfoButton,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';
import { SmallButton, SmallButtonColor } from './SmallButton';

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
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

                <NavigationInfoButton>
                  <ModalMessage>
                    {sprintf(
                      messages.pgettext(
                        'wireguard-settings-view',
                        '%(daita)s (%(daitaFull)s) hides patterns in your encrypted VPN traffic. If anyone is monitoring your connection, this makes it significantly harder for them to identify what websites you are visiting. It does this by carefully adding network noise and making all network packets the same size.',
                      ),
                      { daita: strings.daita, daitaFull: strings.daitaFull },
                    )}
                  </ModalMessage>
                </NavigationInfoButton>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{strings.daita}</HeaderTitle>
                <HeaderSubTitle>
                  {messages.pgettext(
                    'wireguard-settings-view',
                    'Hides patterns in your encrypted VPN traffic. Since this increases your total network traffic, be cautious if you have a limited data plan. It can also negatively impact your network speed and battery usage.',
                  )}
                </HeaderSubTitle>
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
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const smartRouting = useSelector(
    (state) => state.settings.wireguard.daita?.smartRouting ?? false,
  );

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

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
        <Cell.Container>
          <AriaLabel>
            <Cell.InputLabel>{messages.gettext('Enable')}</Cell.InputLabel>
          </AriaLabel>
          <AriaInput>
            <Cell.Switch isOn={daita} onChange={setDaita} />
          </AriaInput>
        </Cell.Container>
      </AriaInputGroup>
      <AriaInputGroup>
        <Cell.Container disabled={!daita}>
          <AriaLabel>
            <Cell.InputLabel>{messages.gettext('Smart routing')}</Cell.InputLabel>
          </AriaLabel>
          <InfoButton>
            <SmartRoutingModalMessage />
          </InfoButton>
          <AriaInput>
            <Cell.Switch isOn={smartRouting} onChange={setSmartRouting} />
          </AriaInput>
        </Cell.Container>
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {sprintf(
                messages.pgettext(
                  'vpn-settings-view',
                  'Is automatically enabled with %(daita)s, makes it possible to use %(daita)s with any server by using multihop. This might increase latency.',
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

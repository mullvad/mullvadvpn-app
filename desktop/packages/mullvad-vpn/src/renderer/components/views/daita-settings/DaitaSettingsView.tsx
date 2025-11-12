import React, { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import {
  Button,
  Flex,
  Icon,
  Image,
  LabelTiny,
  LabelTinySemiBold,
  Text,
} from '../../../lib/components';
import { Dialog } from '../../../lib/components/dialog';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import * as Cell from '../../cell';
import InfoButton from '../../InfoButton';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import PageSlider from '../../PageSlider';
import { SettingsToggleListItem } from '../../settings-toggle-list-item';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { useShowDaitaMultihopInfo } from './hooks';

const StyledLabelTinySemiBold = styled(LabelTinySemiBold).attrs({ color: 'whiteAlpha60' })`
  display: inline-block;
`;

const StyledLabelTiny = styled(LabelTiny).attrs({ color: 'whiteAlpha60' })`
  display: inline-block;
`;

const StyledIllustration = styled(Image)({
  width: '100%',
  padding: '8px 0 8px',
});

export function DaitaSettingsView() {
  const { pop } = useHistory();
  const showDaitaMultihopInfo = useShowDaitaMultihopInfo();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title={strings.daita} />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{strings.daita}</HeaderTitle>
                {showDaitaMultihopInfo && (
                  <Flex $gap="small" $alignItems="center">
                    <Icon icon="info-circle" color="whiteOnBlue60" size="small" />
                    <Text variant="labelTinySemiBold" color="whiteAlpha60">
                      {messages.pgettext(
                        'wireguard-settings-view',
                        'Multihop is being used to enable DAITA for your selected location',
                      )}
                    </Text>
                  </Flex>
                )}
                <PageSlider
                  content={[
                    <React.Fragment key="without-daita">
                      <StyledIllustration source="daita-off-illustration" />
                      <Flex $flexDirection="column" $gap="medium">
                        <StyledLabelTinySemiBold>
                          {messages.pgettext(
                            // TRANSLATORS: Information to the user that with this setting enabled
                            // TRANSLATORS: their network and device's battery life will be
                            // TRANSLATORS: affected negatively.
                            'wireguard-settings-view',
                            'Attention: This increases network traffic and will also negatively affect speed, latency, and battery usage. Use with caution on limited plans.',
                          )}
                        </StyledLabelTinySemiBold>
                        <StyledLabelTiny>
                          {sprintf(
                            messages.pgettext(
                              // TRANSLATORS: Information to the user what the DAITA setting does.
                              // TRANSLATORS: Available placeholders:
                              // TRANSLATORS: %(daita)s - Will be replaced with DAITA
                              // TRANSLATORS: %(daitaFull)s - Will be replaced with Defence against AI-guided Traffic Analysis
                              'wireguard-settings-view',
                              '%(daita)s (%(daitaFull)s) hides patterns in your encrypted VPN traffic.',
                            ),
                            { daita: strings.daita, daitaFull: strings.daitaFull },
                          )}
                        </StyledLabelTiny>
                        <StyledLabelTiny>
                          {messages.pgettext(
                            // TRANSLATORS: Information to the user on the background why the DAITA setting exists.
                            'wireguard-settings-view',
                            'By using sophisticated AI it’s possible to analyze the traffic of data packets going in and out of your device (even if the traffic is encrypted).',
                          )}
                        </StyledLabelTiny>
                      </Flex>
                    </React.Fragment>,
                    <React.Fragment key="with-daita">
                      <StyledIllustration source="daita-on-illustration" />
                      <Flex $flexDirection="column" $gap="medium">
                        <StyledLabelTiny>
                          {sprintf(
                            messages.pgettext(
                              // TRANSLATORS: Information to the user on the background why the DAITA setting exists.
                              'wireguard-settings-view',
                              'If an observer monitors these data packets, %(daita)s makes it significantly harder for them to identify which websites you are visiting or with whom you are communicating.',
                            ),
                            { daita: strings.daita },
                          )}
                        </StyledLabelTiny>
                        <StyledLabelTiny>
                          {sprintf(
                            messages.pgettext(
                              // TRANSLATORS: Information to the user what the DAITA setting does.
                              // TRANSLATORS: Available placeholders:
                              // TRANSLATORS: %(daita)s - Will be replaced with DAITA
                              'wireguard-settings-view',
                              '%(daita)s does this by carefully adding network noise and making all network packets the same size.',
                            ),
                            { daita: strings.daita },
                          )}
                        </StyledLabelTiny>
                        <StyledLabelTiny>
                          {sprintf(
                            messages.pgettext(
                              // TRANSLATORS: Information to the user that DAITA is not available
                              // TRANSLATORS: on all servers, however in the background the multihop
                              // TRANSLATORS: feature is used automatically which enables the use
                              // TRANSLATORS: of DAITA with any server.
                              // TRANSLATORS: Available placeholders:
                              // TRANSLATORS: %(daita)s - Will be replaced with DAITA
                              'wireguard-settings-view',
                              'Not all our servers are %(daita)s-enabled. Therefore, we use multihop automatically to enable %(daita)s with any server.',
                            ),
                            { daita: strings.daita },
                          )}
                        </StyledLabelTiny>
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

  const [confirmDialogVisible, setConfirmDialogVisible] = React.useState(false);

  const unavailable = !('normal' in relaySettings);

  const setDaita = useCallback(
    (value: boolean) => {
      void setEnableDaita(value);
    },
    [setEnableDaita],
  );

  const hideConfirmationDialog = useCallback(() => {
    setConfirmDialogVisible(false);
  }, [setConfirmDialogVisible]);

  const setDirectOnly = useCallback(
    (value: boolean) => {
      if (value) {
        setConfirmDialogVisible(true);
      } else {
        void setDaitaDirectOnly(value);
      }
    },
    [setDaitaDirectOnly, setConfirmDialogVisible],
  );

  const confirmEnableDirectOnly = useCallback(() => {
    void setDaitaDirectOnly(true);
    hideConfirmationDialog();
  }, [hideConfirmationDialog, setDaitaDirectOnly]);

  const directOnlyString = messages.gettext('Direct only');

  return (
    <>
      <SettingsToggleListItem
        anchorId="daita-enable-setting"
        disabled={unavailable}
        checked={daita && !unavailable}
        onCheckedChange={setDaita}
        description={unavailable ? featureUnavailableMessage() : undefined}>
        <SettingsToggleListItem.Label>{messages.gettext('Enable')}</SettingsToggleListItem.Label>
        <SettingsToggleListItem.Switch />
      </SettingsToggleListItem>
      <SettingsToggleListItem
        disabled={!daita || unavailable}
        checked={directOnly && !unavailable}
        onCheckedChange={setDirectOnly}>
        <SettingsToggleListItem.Label>{directOnlyString}</SettingsToggleListItem.Label>
        <SettingsToggleListItem.Group>
          <InfoButton>
            <DirectOnlyModalMessage />
          </InfoButton>
          <SettingsToggleListItem.Switch />
        </SettingsToggleListItem.Group>
      </SettingsToggleListItem>
      <Dialog open={confirmDialogVisible} onOpenChange={setConfirmDialogVisible}>
        <Dialog.Container>
          <Dialog.Icon icon="info-circle" />
          <Dialog.Text>
            {sprintf(
              // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
              messages.pgettext(
                'wireguard-settings-view',
                'Not all our servers are %(daita)s-enabled. In order to use the internet, you might have to select a new location after enabling.',
              ),
              { daita: strings.daita },
            )}
          </Dialog.Text>
          <Dialog.ButtonGroup>
            <Button key="confirm" onClick={confirmEnableDirectOnly}>
              <Button.Text>
                {
                  // TRANSLATORS: A toggle that refers to the setting "Direct only".
                  messages.gettext('Enable direct only')
                }
              </Button.Text>
            </Button>
            <Dialog.Button key="cancel" onClick={hideConfirmationDialog}>
              <Button.Text>{messages.pgettext('wireguard-settings-view', 'Cancel')}</Button.Text>
            </Dialog.Button>
          </Dialog.ButtonGroup>
        </Dialog.Container>
      </Dialog>
    </>
  );
}

function DirectOnlyModalMessage() {
  const directOnlyString = messages.gettext('Direct only');

  return (
    <Dialog.Text>
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
    </Dialog.Text>
  );
}

function featureUnavailableMessage() {
  const tunnelProtocol = messages.pgettext('vpn-settings-view', 'Tunnel protocol');

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
    { wireguard: strings.wireguard, tunnelProtocol, setting: strings.daita },
  );
}

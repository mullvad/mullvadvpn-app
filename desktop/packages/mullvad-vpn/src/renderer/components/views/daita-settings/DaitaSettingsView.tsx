import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { useAppContext } from '../../../context';
import { Button, Flex, Icon, Text } from '../../../lib/components';
import { Carousel } from '../../../lib/components/carousel';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { useBoolean } from '../../../lib/utility-hooks';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../..';
import InfoButton from '../../InfoButton';
import { BackAction } from '../../KeyboardNavigation';
import { ModalAlert, ModalAlertType, ModalMessage } from '../../Modal';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { SettingsToggleListItem } from '../../settings-toggle-list-item';
import { HeaderTitle } from '../../SettingsHeader';
import { useShowDaitaMultihopInfo } from './hooks';

export function DaitaSettingsView() {
  const { pop } = useHistory();
  const showDaitaMultihopInfo = useShowDaitaMultihopInfo();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader title={strings.daita} />

          <NavigationScrollbars>
            <Flex $flexDirection="column" $gap="medium">
              <View.Container>
                <FlexColumn $gap="medium">
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

                  <Carousel
                    aria-label={
                      // TRANSLATORS: Accessibility label for a carousel that explains
                      // TRANSLATORS: the DAITA feature to the user.
                      messages.pgettext('accessibility', 'DAITA information carousel')
                    }>
                    <Carousel.Slides>
                      <Carousel.Slide key="without-daita">
                        <Carousel.Image
                          source="daita-off-illustration"
                          alt={
                            // TRANSLATORS: Alt text for an illustration showing VPN traffic without DAITA.
                            messages.pgettext(
                              'accessibility',
                              'Illustration showing VPN traffic without DAITA',
                            )
                          }
                        />
                        <Carousel.TextGroup>
                          <Carousel.Text variant="labelTinySemiBold">
                            {messages.pgettext(
                              // TRANSLATORS: Information to the user that with this setting enabled
                              // TRANSLATORS: their network and device's battery life will be
                              // TRANSLATORS: affected negatively.
                              'wireguard-settings-view',
                              'Attention: This increases network traffic and will also negatively affect speed, latency, and battery usage. Use with caution on limited plans.',
                            )}
                          </Carousel.Text>
                          <Carousel.Text>
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
                          </Carousel.Text>
                          <Carousel.Text>
                            {messages.pgettext(
                              // TRANSLATORS: Information to the user on the background why the DAITA setting exists.
                              'wireguard-settings-view',
                              'By using sophisticated AI it’s possible to analyze the traffic of data packets going in and out of your device (even if the traffic is encrypted).',
                            )}
                          </Carousel.Text>
                        </Carousel.TextGroup>
                      </Carousel.Slide>
                      <Carousel.Slide key="with-daita">
                        <Carousel.Image
                          source="daita-on-illustration"
                          alt={
                            // TRANSLATORS: Alt text for an illustration showing VPN traffic with DAITA.
                            messages.pgettext(
                              'accessibility',
                              'Illustration showing VPN traffic with DAITA enabled',
                            )
                          }
                        />
                        <Carousel.TextGroup>
                          <Carousel.Text>
                            {sprintf(
                              messages.pgettext(
                                // TRANSLATORS: Information to the user on the background why the DAITA setting exists.
                                'wireguard-settings-view',
                                'If an observer monitors these data packets, %(daita)s makes it significantly harder for them to identify which websites you are visiting or with whom you are communicating.',
                              ),
                              { daita: strings.daita },
                            )}
                          </Carousel.Text>
                          <Carousel.Text>
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
                          </Carousel.Text>
                          <Carousel.Text>
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
                          </Carousel.Text>
                        </Carousel.TextGroup>
                      </Carousel.Slide>
                    </Carousel.Slides>
                    <Carousel.Controls>
                      <Carousel.Indicators
                        ariaLabelTemplate={messages.pgettext(
                          // TRANSLATORS: Accessibility label for carousel indicators that
                          // TRANSLATORS: allow navigation between slides.
                          // TRANSLATORS: Available placeholders:
                          // TRANSLATORS: %(index)s - Will be replaced with the slide index number.
                          'accessibility',
                          'Go to slide %(index)s',
                        )}
                      />
                      <Carousel.ControlGroup>
                        <Carousel.PrevButton
                          aria-label={
                            // TRANSLATORS: Accessibility label for a button that
                            // TRANSLATORS: navigates to the previous slide in a carousel.
                            messages.pgettext('accessibility', 'Previous slide')
                          }
                        />
                        <Carousel.NextButton
                          aria-label={
                            // TRANSLATORS: Accessibility label for a button that
                            // TRANSLATORS: navigates to the next slide in a carousel.
                            messages.pgettext('accessibility', 'Next slide')
                          }
                        />
                      </Carousel.ControlGroup>
                    </Carousel.Controls>
                  </Carousel>
                </FlexColumn>
              </View.Container>
              <FlexColumn $padding={{ bottom: 'medium' }}>
                <DaitaToggle />
              </FlexColumn>
            </Flex>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

function DaitaToggle() {
  const { setEnableDaita, setDaitaDirectOnly } = useAppContext();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const daita = useSelector((state) => state.settings.wireguard.daita?.enabled ?? false);
  const directOnly = useSelector((state) => state.settings.wireguard.daita?.directOnly ?? false);

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

  const unavailable = !('normal' in relaySettings);

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
      <ModalAlert
        isOpen={confirmationDialogVisible}
        type={ModalAlertType.caution}
        gridButtons={[
          <Button key="cancel" onClick={hideConfirmationDialog}>
            <Button.Text>{messages.pgettext('wireguard-settings-view', 'Cancel')}</Button.Text>
          </Button>,
          <Button key="confirm" onClick={confirmEnableDirectOnly}>
            <Button.Text>
              {
                // TRANSLATORS: A toggle that refers to the setting "Direct only".
                messages.gettext('Enable direct only')
              }
            </Button.Text>
          </Button>,
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

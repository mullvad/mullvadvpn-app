import { sprintf } from 'sprintf-js';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { DaitaDirectOnlySetting, DaitaSetting } from '../../../features/daita/components';
import { Flex, Icon, Text } from '../../../lib/components';
import { Carousel } from '../../../lib/components/carousel';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
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
                      // TRANSLATORS: Accessibility label for a carousel that explains the DAITA feature to the user.
                      messages.pgettext('accessibility', 'DAITA information carousel')
                    }>
                    <Carousel.Slides>
                      <Carousel.Slide>
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
                              // TRANSLATORS: Information to the user that with this setting enabled their network and device's battery life will be
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
                      <Carousel.Slide>
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
                                // TRANSLATORS: Information to the user that DAITA is not available on all servers, however in the background the multihop
                                // TRANSLATORS: feature is used automatically which enables the use of DAITA with any server.
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
                      <Carousel.Indicators />
                      <Carousel.ControlGroup>
                        <Carousel.PrevButton />
                        <Carousel.NextButton />
                      </Carousel.ControlGroup>
                    </Carousel.Controls>
                  </Carousel>
                </FlexColumn>
              </View.Container>
              <FlexColumn $padding={{ bottom: 'medium' }}>
                <DaitaSetting />
                <DaitaDirectOnlySetting />
              </FlexColumn>
            </Flex>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

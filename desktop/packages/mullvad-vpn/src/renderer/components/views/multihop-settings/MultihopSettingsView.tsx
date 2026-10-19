import { messages } from '../../../../shared/gettext';
import { MultihopSetting } from '../../../features/multihop/components';
import { Carousel } from '../../../lib/components/carousel';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
import { AlwaysSlide, MultihopSlide, NeverSlide, WhenNeededSlide } from './components';

export function MultihopSettingsView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader title={messages.pgettext('wireguard-settings-view', 'Multihop')} />

          <NavigationScrollbars>
            <View.Content>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'Multihop')}
                </HeaderTitle>
                <FlexColumn gap="large">
                  <Carousel
                    aria-label={
                      // TRANSLATORS: Accessibility label for a carousel that explains the multihop feature to the user.
                      messages.pgettext('accessibility', 'Multihop information carousel')
                    }>
                    <Carousel.Slides>
                      <MultihopSlide />
                      <WhenNeededSlide />
                      <AlwaysSlide />
                      <NeverSlide />
                    </Carousel.Slides>
                    <Carousel.Controls>
                      <Carousel.Controls.Indicators />
                      <Carousel.Controls.ControlGroup>
                        <Carousel.Controls.PrevButton />
                        <Carousel.Controls.NextButton />
                      </Carousel.Controls.ControlGroup>
                    </Carousel.Controls>
                  </Carousel>

                  <MultihopSetting />
                </FlexColumn>
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

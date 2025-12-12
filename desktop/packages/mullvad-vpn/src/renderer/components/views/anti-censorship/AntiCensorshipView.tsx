import { messages } from '../../../../shared/gettext';
import { Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { MethodSetting } from './components';

export function AntiCensorshipView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={messages.pgettext('anti-censorship-view', 'Anti-censorship')}
          />

          <NavigationScrollbars>
            <View.Content>
              <SettingsHeader>
                <HeaderTitle>
                  {
                    // TRANSLATORS: Page title for anti censorship settings view
                    messages.pgettext('anti-censorship-view', 'Anti-censorship')
                  }
                </HeaderTitle>
              </SettingsHeader>
              <FlexColumn gap="medium">
                <View.Container indent="medium" flexDirection="column" gap="medium">
                  <Text variant="labelTiny" color="whiteAlpha60">
                    {
                      // TRANSLATORS: First paragraph of description text in anti-censorship view
                      messages.pgettext(
                        'anti-censorship-view',
                        'These methods may be useful in situations where you are blocked from reaching Mullvad. When "Automatic" is selected, the app will attempt all methods until one works.',
                      )
                    }
                  </Text>
                  <Text variant="labelTinySemiBold" color="whiteAlpha60">
                    {
                      // TRANSLATORS: Second paragraph of description text in anti-censorship view
                      messages.pgettext(
                        'anti-censorship-view',
                        'Please note that these methods do not improve performance, and may increase system utilization and battery consumption.',
                      )
                    }
                  </Text>
                </View.Container>

                <MethodSetting />
              </FlexColumn>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

import { messages } from '../../../../shared/gettext';
import { Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { Container } from '../../../lib/components/view/components';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { MethodSetting } from './components';

export function CensorshipCircumventionView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={messages.pgettext('censorship-circumvention-view', 'Censorship circumvention')}
          />

          <NavigationScrollbars>
            <SettingsHeader>
              <HeaderTitle>
                {
                  // TRANSLATORS: Page title for censorship circumvention settings view
                  messages.pgettext('censorship-circumvention-view', 'Censorship circumvention')
                }
              </HeaderTitle>
            </SettingsHeader>
            <FlexColumn $gap="medium">
              <Container size="4" $flexDirection="column" $gap="medium">
                <Text variant="labelTinySemiBold" color="whiteAlpha60">
                  {
                    // TRANSLATORS: First paragraph of description text in censorship circumvention settings view
                    messages.pgettext(
                      'censorship-circumvention-view',
                      'Obfuscation methods makes your encrypted VPN traffic look like something else. This can be used to help circumvent censorship and other types of filtering, where a plain connection would be blocked.',
                    )
                  }
                </Text>
                <Text variant="labelTinySemiBold" color="whiteAlpha60">
                  {
                    // TRANSLATORS: Second paragraph of description text in censorship circumvention settings view
                    messages.pgettext(
                      'censorship-circumvention-view',
                      'When “Automatic” is selected, the app tries all methods when connecting until it finds one that works.',
                    )
                  }
                </Text>
              </Container>

              <MethodSetting />
            </FlexColumn>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

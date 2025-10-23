import { messages } from '../../../../shared/gettext';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { SettingsContent, SettingsGroup } from '../../Layout';
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

            <SettingsContent>
              <SettingsGroup>
                <MethodSetting />
              </SettingsGroup>
            </SettingsContent>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

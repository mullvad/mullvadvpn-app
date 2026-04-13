import { strings } from '../../../../shared/constants';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
import { LwoPortSetting } from './components';

export function LwoSettingsView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <View.Content>
            <AppNavigationHeader title={strings.lwo} />

            <NavigationScrollbars>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>{strings.lwo}</HeaderTitle>
                <LwoPortSetting />
              </View.Container>
            </NavigationScrollbars>
          </View.Content>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

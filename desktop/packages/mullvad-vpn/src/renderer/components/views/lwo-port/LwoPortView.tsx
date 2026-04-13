import { sprintf } from 'sprintf-js';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';
import { PortSetting } from './components';

export function LwoPortView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <View.Content>
            <AppNavigationHeader
              title={sprintf(
                // TRANSLATORS: Navigation header for LWO port view
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(lwo)s - will be replaced with LWO
                messages.pgettext('lwo-settings-view', '%(lwo)s port'),
                { lwo: strings.lwo },
              )}
            />

            <NavigationScrollbars>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="medium">
                <HeaderTitle>
                  {sprintf(
                    // TRANSLATORS: Page title for LWO port view
                    // TRANSLATORS: Available placeholders:
                    // TRANSLATORS: %(lwo)s - will be replaced with LWO
                    messages.pgettext('lwo-settings-view', '%(lwo)s port'),
                    { lwo: strings.lwo },
                  )}
                </HeaderTitle>
                <PortSetting />
              </View.Container>
            </NavigationScrollbars>
          </View.Content>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

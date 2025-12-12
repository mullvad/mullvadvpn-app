import { messages } from '../../../../shared/gettext';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { FaqButton, ProblemReportButton } from './components';

export function SupportView() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <View backgroundColor="darkBlue">
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title label in navigation bar
              messages.pgettext('support-view', 'Support')
            }
          />

          <NavigationScrollbars>
            <SettingsHeader>
              <HeaderTitle>{messages.pgettext('support-view', 'Support')}</HeaderTitle>
            </SettingsHeader>

            <FlexColumn>
              <ProblemReportButton />
              <FaqButton />
            </FlexColumn>
          </NavigationScrollbars>
        </NavigationContainer>
      </View>
    </BackAction>
  );
}

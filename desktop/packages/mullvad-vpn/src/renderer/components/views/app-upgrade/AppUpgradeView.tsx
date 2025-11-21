import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../app-navigation-header';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { Footer, UpgradeDetails } from './components';
import { NoUpgradeAvailable } from './components/no-upgrade-available';
import { useHasUpgrade } from './hooks';

const StyledFooter = styled.div`
  // TODO: Use color from Colors
  background-color: rgba(21, 39, 58, 1);
  position: sticky;
  bottom: 0;
  width: 100%;
`;

export const AppUpgradeView = () => {
  const { pop } = useHistory();
  const hasUpgrade = useHasUpgrade();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader
            title={
              // TRANSLATORS: Title in navigation bar
              messages.pgettext('app-upgrade-view', 'Update available')
            }
          />
          <NavigationScrollbars>
            <View.Content data-testid="view-conent">
              {hasUpgrade ? <UpgradeDetails /> : <NoUpgradeAvailable />}
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
        <StyledFooter>
          <Footer />
        </StyledFooter>
      </BackAction>
    </View>
  );
};

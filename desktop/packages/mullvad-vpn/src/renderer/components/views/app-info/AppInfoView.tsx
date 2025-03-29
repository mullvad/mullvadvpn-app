import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { Flex } from '../../../lib/components';
import { Animate } from '../../../lib/components/animate';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { ChangelogListItem, UpdateAvailableListItem, VersionListItem } from './components';
import { BetaListItem } from './components/beta-list-item';
import { useShowUpdateAvailable } from './hooks';

const StyledUpdateAvailableListItem = styled(UpdateAvailableListItem)`
  margin-bottom: 16px;
`;

export function AppInfoView() {
  const { pop } = useHistory();
  const showUpdateAvailable = useShowUpdateAvailable();
  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title of the app info view.
                messages.pgettext('app-info-view', 'App info')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('app-info-view', 'App info')}</HeaderTitle>
              </SettingsHeader>
              <Animate
                present={showUpdateAvailable}
                animations={[{ type: 'fade' }, { type: 'wipe', direction: 'vertical' }]}>
                <StyledUpdateAvailableListItem />
              </Animate>
              <Flex $flexDirection="column" $gap="medium">
                <Flex $flexDirection="column">
                  <VersionListItem />
                  <ChangelogListItem />
                </Flex>
                <BetaListItem />
              </Flex>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

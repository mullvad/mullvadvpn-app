import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { Flex } from '../../../lib/components';
import { ListItemFooter } from '../../../lib/components/list-item/components';
import { useHistory } from '../../../lib/history';
import { useSelector } from '../../../redux/store';
import { AppNavigationHeader } from '../../';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer, SettingsStack } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { ChangelogListItem, UpdateAvailableListItem, VersionListItem } from './components';
import { BetaListItem } from './components/BetaListItem';

const StyledFlex = styled(Flex)`
  gap: 1px;
  &&:has(${ListItemFooter}) {
    gap: 8px;
  }
`;

export const AppInfoView = () => {
  const { pop } = useHistory();
  const suggestedUpgrade = useSelector((state) => state.version.suggestedUpgrade);
  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title={messages.pgettext('app-info-view', 'App info')} />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>{messages.pgettext('app-info-view', 'App info')}</HeaderTitle>
              </SettingsHeader>
              <SettingsContainer>
                <SettingsStack>
                  {suggestedUpgrade && <UpdateAvailableListItem />}
                  <StyledFlex $flexDirection="column">
                    <VersionListItem />
                    <ChangelogListItem />
                  </StyledFlex>
                  <BetaListItem />
                </SettingsStack>
              </SettingsContainer>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
};

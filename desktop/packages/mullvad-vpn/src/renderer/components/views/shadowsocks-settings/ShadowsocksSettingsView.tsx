import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import * as Cell from '../../cell';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { ShadowsocksPortSetting } from './components';

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

export function ShadowsocksSettingsView() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('wireguard-settings-nav', 'Shadowsocks')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'Shadowsocks')}
                </HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <Cell.Group>
                  <ShadowsocksPortSetting />
                </Cell.Group>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

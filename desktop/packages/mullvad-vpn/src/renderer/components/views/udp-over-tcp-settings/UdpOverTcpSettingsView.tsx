import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import * as Cell from '../../cell';
import { SelectorItem } from '../../cell/Selector';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { UdpOverTcpPortSetting } from './components';

export const UDP2TCP_PORTS = [80, 5001];

export function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}

const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

export function UdpOverTcpSettingsView() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar
                messages.pgettext('wireguard-settings-nav', 'UDP-over-TCP')
              }
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'UDP-over-TCP')}
                </HeaderTitle>
              </SettingsHeader>

              <StyledContent>
                <Cell.Group>
                  <UdpOverTcpPortSetting />
                </Cell.Group>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

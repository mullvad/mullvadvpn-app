import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { Flex } from '../../../lib/components';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import * as Cell from '../../cell';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from '../../SettingsHeader';
import { MultihopSetting } from './components';

const PATH_PREFIX = process.env.NODE_ENV === 'development' ? '../' : '';

const StyledIllustration = styled.img({
  width: '100%',
  padding: '8px 0 8px',
});

export function MultihopSettingsView() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader title={messages.pgettext('wireguard-settings-view', 'Multihop')} />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'Multihop')}
                </HeaderTitle>
                <HeaderSubTitle>
                  <StyledIllustration
                    src={`${PATH_PREFIX}assets/images/multihop-illustration.svg`}
                  />
                  {messages.pgettext(
                    'wireguard-settings-view',
                    'Multihop routes your traffic into one WireGuard server and out another, making it harder to trace. This results in increased latency but increases anonymity online.',
                  )}
                </HeaderSubTitle>
              </SettingsHeader>

              <Flex $flexDirection="column" $flex={1}>
                <Cell.Group>
                  <MultihopSetting />
                </Cell.Group>
              </Flex>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

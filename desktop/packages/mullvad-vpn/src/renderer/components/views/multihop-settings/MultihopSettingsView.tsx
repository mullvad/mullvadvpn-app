import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { Flex, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import * as Cell from '../../cell';
import { BackAction } from '../../KeyboardNavigation';
import { Layout, SettingsContainer } from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import { MultihopSetting } from './components';

const StyledIllustration = styled.img({
  width: '100%',
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
                <FlexColumn $gap="medium">
                  <HeaderTitle>
                    {messages.pgettext('wireguard-settings-view', 'Multihop')}
                  </HeaderTitle>
                  <FlexColumn $gap="small">
                    <StyledIllustration src="assets/images/multihop-illustration.svg" />
                    <Text variant="labelTiny" color="whiteAlpha60">
                      {messages.pgettext(
                        'wireguard-settings-view',
                        'Multihop routes your traffic into one WireGuard server and out another, making it harder to trace. This results in increased latency but increases anonymity online.',
                      )}
                    </Text>
                  </FlexColumn>
                </FlexColumn>
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

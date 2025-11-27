import styled from 'styled-components';

import { messages } from '../../../../shared/gettext';
import { MultihopSetting } from '../../../features/multihop/components';
import { Image, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';

const StyledIllustration = styled(Image)({
  width: '100%',
});

export function MultihopSettingsView() {
  const { pop } = useHistory();

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader title={messages.pgettext('wireguard-settings-view', 'Multihop')} />

          <NavigationScrollbars>
            <View.Content>
              <SettingsHeader>
                <FlexColumn gap="medium">
                  <HeaderTitle>
                    {messages.pgettext('wireguard-settings-view', 'Multihop')}
                  </HeaderTitle>
                  <FlexColumn gap="small">
                    <StyledIllustration source="multihop-illustration" />
                    <Text variant="labelTiny" color="whiteAlpha60">
                      {messages.pgettext(
                        'wireguard-settings-view',
                        'Multihop routes your traffic into one WireGuard server and out another, making it harder to trace. This results in increased latency but increases anonymity online.',
                      )}
                    </Text>
                  </FlexColumn>
                </FlexColumn>
              </SettingsHeader>

              <MultihopSetting />
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

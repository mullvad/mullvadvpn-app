import { sprintf } from 'sprintf-js';

import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import { Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { formatHtml } from '../../../lib/html-formatter';
import { AppNavigationHeader } from '../..';
import { InternalLink } from '../../internal-link';
import { BackAction } from '../../keyboard-navigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { HeaderTitle } from '../../SettingsHeader';

export function KillSwitchView() {
  const { pop } = useHistory();

  const killSwitchSettingName = messages.gettext('Kill switch');
  const lockdownModeSettingName = messages.gettext('Lockdown mode');

  return (
    <View backgroundColor="darkBlue">
      <BackAction action={pop}>
        <NavigationContainer>
          <AppNavigationHeader title={killSwitchSettingName} />

          <NavigationScrollbars>
            <View.Content>
              <View.Container horizontalMargin="medium" flexDirection="column" gap="small">
                <HeaderTitle>{killSwitchSettingName}</HeaderTitle>
                <FlexColumn gap="medium">
                  <Text variant="labelTiny" color="whiteAlpha60">
                    {sprintf(
                      // TRANSLATORS: First paragraph of description text in kill switch view.
                      // TRANSLATORS: Available placeholders:
                      // TRANSLATORS: %(killSwitchSettingName)s - The name of the Kill switch setting
                      messages.pgettext(
                        'kill-switch-view',
                        'The %(killSwitchSettingName)s is active while you’re connected, it automatically blocks all network traffic if your VPN tunnel, network, or internet connection stops working. Once your connection is restored, the app will unblock your traffic.',
                      ),
                      { killSwitchSettingName },
                    )}
                  </Text>
                  <Text variant="labelTiny" color="whiteAlpha60">
                    {sprintf(
                      // TRANSLATORS: Second paragraph of description text in kill switch view.
                      // TRANSLATORS: Available placeholders:
                      // TRANSLATORS: %(killSwitchSettingName)s - The name of the Kill switch setting
                      messages.pgettext(
                        'kill-switch-view',
                        'The %(killSwitchSettingName)s is a core part of the app’s leak prevention and can’t be disabled.',
                      ),
                      { killSwitchSettingName },
                    )}
                  </Text>
                  <Text variant="labelTinySemiBold" color="whiteAlpha60">
                    {formatHtml(
                      sprintf(
                        // TRANSLATORS: Third paragraph of description text in kill switch view.
                        // TRANSLATORS: Available placeholders:
                        // TRANSLATORS: %(killSwitchSettingName)s - The name of the Kill switch setting
                        // TRANSLATORS: %(lockdownModeSettingName)s - The name of the Lockdown mode setting
                        messages.pgettext(
                          'kill-switch-view',
                          'The %(killSwitchSettingName)s doesn’t protect you if you manually disconnect or quit the app. If that’s something you want, you can enable <a>%(lockdownModeSettingName)s</a>.',
                        ),
                        { killSwitchSettingName, lockdownModeSettingName },
                      ),
                      {
                        a: (value) => (
                          <InternalLink
                            variant="labelTinySemiBold"
                            to={RoutePath.vpnSettings}
                            locationState={{
                              options: [
                                {
                                  type: 'scroll-to-anchor',
                                  id: 'lockdown-mode-setting',
                                },
                              ],
                            }}>
                            <InternalLink.Text>{value}</InternalLink.Text>
                          </InternalLink>
                        ),
                      },
                    )}
                  </Text>
                </FlexColumn>
              </View.Container>
            </View.Content>
          </NavigationScrollbars>
        </NavigationContainer>
      </BackAction>
    </View>
  );
}

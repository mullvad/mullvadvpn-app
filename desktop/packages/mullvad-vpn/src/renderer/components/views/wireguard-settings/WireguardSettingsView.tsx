import { sprintf } from 'sprintf-js';

import { strings } from '../../../../shared/constants';
import { messages } from '../../../../shared/gettext';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../..';
import { BackAction } from '../../KeyboardNavigation';
import {
  Layout,
  SettingsContainer,
  SettingsContent,
  SettingsGroup,
  SettingsStack,
} from '../../Layout';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from '../../SettingsHeader';
import {
  IpVersionSetting,
  MtuSetting,
  ObfuscationSettings,
  PortSelector,
  QuantumResistantSetting,
} from './components';

export function WireguardSettingsView() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <AppNavigationHeader
              title={sprintf(
                // TRANSLATORS: Title label in navigation bar
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
                messages.pgettext('wireguard-settings-nav', '%(wireguard)s settings'),
                { wireguard: strings.wireguard },
              )}
            />

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {sprintf(
                    // TRANSLATORS: Available placeholders:
                    // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
                    messages.pgettext('wireguard-settings-view', '%(wireguard)s settings'),
                    { wireguard: strings.wireguard },
                  )}
                </HeaderTitle>
              </SettingsHeader>
              <SettingsContent>
                <SettingsStack>
                  <SettingsGroup>
                    <PortSelector />
                  </SettingsGroup>

                  <SettingsGroup>
                    <ObfuscationSettings />
                  </SettingsGroup>

                  <SettingsGroup>
                    <QuantumResistantSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <IpVersionSetting />
                  </SettingsGroup>

                  <SettingsGroup>
                    <MtuSetting />
                  </SettingsGroup>
                </SettingsStack>
              </SettingsContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

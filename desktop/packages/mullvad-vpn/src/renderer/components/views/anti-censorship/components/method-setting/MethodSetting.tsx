import { useCallback } from 'react';

import { ObfuscationType } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsListbox } from '../../../../settings-listbox';
import {
  AutomaticOption,
  LwoOption,
  NoneOption,
  QuicOption,
  ShadowsocksOption,
  UdpOverTcpOption,
  WireguardPortOption,
} from './components';

export function MethodSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  const { selectedObfuscation } = obfuscationSettings;

  const handleSelectObfuscation = useCallback(
    async (value: ObfuscationType) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        selectedObfuscation: value,
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  return (
    <SettingsListbox
      anchorId="obfuscation-setting"
      onValueChange={handleSelectObfuscation}
      value={selectedObfuscation}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {
              // TRANSLATORS: The title for the obfuscation method selector.
              messages.pgettext('anti-censorship-view', 'Method')
            }
          </SettingsListbox.Label>
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <AutomaticOption />
        <WireguardPortOption />
        <LwoOption />
        <QuicOption />
        <ShadowsocksOption />
        <UdpOverTcpOption />
        <NoneOption />
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}

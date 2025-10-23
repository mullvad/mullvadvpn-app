import { useCallback } from 'react';

import { Constraint, ObfuscationType } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SettingsListbox } from '../../../../settings-listbox';
import {
  AutomaticOption,
  LwoOption,
  OffOption,
  QuicOption,
  ShadowsocksOption,
  UdpOverTcpOption,
  WireguardPortOption,
} from './components';

export function formatPortForSubLabel(port: Constraint<number>): string {
  return port === 'any' ? messages.gettext('Automatic') : `${port.only}`;
}

export function MethodSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  const obfuscationType = obfuscationSettings.selectedObfuscation;

  const selectObfuscationType = useCallback(
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
      onValueChange={selectObfuscationType}
      value={obfuscationType}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {
              // TRANSLATORS: The title for the obfuscation method selector.
              messages.pgettext('censorship-circumvention-view', 'Method')
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
        <OffOption />
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}

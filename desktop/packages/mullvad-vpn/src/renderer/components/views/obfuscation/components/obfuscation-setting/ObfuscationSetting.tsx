import { useCallback } from 'react';

import { ObfuscationType } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListbox } from '../../../../settings-listbox';
import {
  AutomaticOption,
  LwoOption,
  PortOption,
  QuicOption,
  ShadowsocksOption,
  UdpOverTcpOption,
} from './components';

export function ObfuscationSetting() {
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
              // TRANSLATORS: The title for the WireGuard obfuscation selector.
              messages.pgettext('wireguard-settings-view', 'Obfuscation')
            }
          </SettingsListbox.Label>
          <InfoButton>
            <ModalMessage>
              {
                // TRANSLATORS: Describes what WireGuard obfuscation does, how it works and when
                // TRANSLATORS: it would be useful to enable it.
                messages.pgettext(
                  'wireguard-settings-view',
                  'Obfuscation hides the WireGuard traffic inside another protocol. It can be used to help circumvent censorship and other types of filtering, where a plain WireGuard connection would be blocked.',
                )
              }
            </ModalMessage>
          </InfoButton>
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <AutomaticOption />
        <PortOption />
        <LwoOption />
        <QuicOption />
        <ShadowsocksOption />
        <UdpOverTcpOption />
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}

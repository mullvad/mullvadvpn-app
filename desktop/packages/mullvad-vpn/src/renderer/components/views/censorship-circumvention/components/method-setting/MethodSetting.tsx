import { useCallback } from 'react';

import { Constraint, ObfuscationType } from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListbox } from '../../../../settings-listbox';
import {
  AutomaticOption,
  LwoOption,
  OffOption,
  QuicOption,
  ShadowsocksOption,
  UdpOverTcpOption,
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
              messages.pgettext('wireguard-settings-view', 'Method')
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
        <LwoOption />
        <QuicOption />
        <ShadowsocksOption />
        <UdpOverTcpOption />
        <OffOption />
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}

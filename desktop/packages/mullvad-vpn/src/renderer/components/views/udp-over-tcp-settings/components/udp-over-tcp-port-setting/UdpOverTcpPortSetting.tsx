import { useCallback, useMemo } from 'react';

import {
  liftConstraint,
  LiftedConstraint,
  wrapConstraint,
} from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { SelectorItem } from '../../../../cell/Selector';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';
import { SettingsListbox } from '../../../../settings-listbox';

const UDP2TCP_PORTS = [80, 443, 5001];

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}

export function UdpOverTcpPortSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  const port = liftConstraint(obfuscationSettings.udp2tcpSettings.port);
  const portItems: SelectorItem<number>[] = useMemo(
    () => UDP2TCP_PORTS.map(mapPortToSelectorItem),
    [],
  );

  const selectPort = useCallback(
    async (port: LiftedConstraint<number>) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        udp2tcpSettings: {
          ...obfuscationSettings.udp2tcpSettings,
          port: wrapConstraint(port),
        },
      });
    },
    [setObfuscationSettings, obfuscationSettings],
  );

  return (
    <SettingsListbox value={port} onValueChange={selectPort}>
      <SettingsListbox.Item>
        <SettingsListbox.Content>
          <SettingsListbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard port selector.
              messages.pgettext('wireguard-settings-view', 'Port')
            }
          </SettingsListbox.Label>
          <InfoButton>
            <ModalMessage>
              {messages.pgettext(
                'wireguard-settings-view',
                'Which TCP port the UDP-over-TCP obfuscation protocol should connect to on the VPN server.',
              )}
            </ModalMessage>
          </InfoButton>
        </SettingsListbox.Content>
      </SettingsListbox.Item>
      <SettingsListbox.Options>
        <SettingsListbox.BaseOption value={'any'}>
          {messages.gettext('Automatic')}
        </SettingsListbox.BaseOption>
        {portItems.map((item) => {
          return (
            <SettingsListbox.BaseOption key={item.value} value={item.value}>
              {item.label}
            </SettingsListbox.BaseOption>
          );
        })}
      </SettingsListbox.Options>
    </SettingsListbox>
  );
}

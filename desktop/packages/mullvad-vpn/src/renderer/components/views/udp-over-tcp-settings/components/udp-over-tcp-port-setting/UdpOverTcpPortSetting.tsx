import { useCallback, useMemo } from 'react';

import {
  liftConstraint,
  LiftedConstraint,
  wrapConstraint,
} from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Listbox } from '../../../../../lib/components/listbox/Listbox';
import { useSelector } from '../../../../../redux/store';
import { SelectorItem } from '../../../../cell/Selector';
import { DefaultListboxOption } from '../../../../default-listbox-option';
import InfoButton from '../../../../InfoButton';
import { ModalMessage } from '../../../../Modal';

const UDP2TCP_PORTS = [80, 5001];

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
    <Listbox value={port} onValueChange={selectPort}>
      <Listbox.Item>
        <Listbox.Content>
          <Listbox.Label>
            {
              // TRANSLATORS: The title for the WireGuard port selector.
              messages.pgettext('wireguard-settings-view', 'Port')
            }
          </Listbox.Label>
          <InfoButton>
            <ModalMessage>
              {messages.pgettext(
                'wireguard-settings-view',
                'Which TCP port the UDP-over-TCP obfuscation protocol should connect to on the VPN server.',
              )}
            </ModalMessage>
          </InfoButton>
        </Listbox.Content>
      </Listbox.Item>
      <Listbox.Options>
        <DefaultListboxOption value={'any'}>{messages.gettext('Automatic')}</DefaultListboxOption>
        {portItems.map((item) => {
          return (
            <DefaultListboxOption key={item.value} value={item.value}>
              {item.label}
            </DefaultListboxOption>
          );
        })}
      </Listbox.Options>
    </Listbox>
  );
}

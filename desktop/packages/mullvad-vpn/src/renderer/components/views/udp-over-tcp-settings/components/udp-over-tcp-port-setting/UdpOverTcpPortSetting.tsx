import { useCallback, useMemo } from 'react';

import {
  liftConstraint,
  LiftedConstraint,
  wrapConstraint,
} from '../../../../../../shared/daemon-rpc-types';
import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { AriaInputGroup } from '../../../../AriaGroup';
import Selector, { SelectorItem } from '../../../../cell/Selector';
import { ModalMessage } from '../../../../Modal';
import {
  mapPortToSelectorItem,
  StyledSelectorContainer,
  UDP2TCP_PORTS,
} from '../../UdpOverTcpSettingsView';

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
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          // TRANSLATORS: The title for the UDP-over-TCP port selector.
          title={messages.pgettext('wireguard-settings-view', 'UDP-over-TCP port')}
          details={
            <ModalMessage>
              {messages.pgettext(
                'wireguard-settings-view',
                'Which TCP port the UDP-over-TCP obfuscation protocol should connect to on the VPN server.',
              )}
            </ModalMessage>
          }
          items={portItems}
          value={port}
          onSelect={selectPort}
          thinTitle
          automaticValue={'any' as const}
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

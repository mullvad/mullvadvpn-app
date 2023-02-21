import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../config.json';
import {
  IpVersion,
  liftConstraint,
  LiftedConstraint,
  ObfuscationType,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { removeNonNumericCharacters } from '../../shared/string-helpers';
import { useAppContext } from '../context';
import { createWireguardRelayUpdater } from '../lib/constraint-updater';
import { useHistory } from '../lib/history';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import Selector, { SelectorItem, SelectorWithCustomItem } from './cell/Selector';
import { InfoIcon } from './InfoButton';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const MIN_WIREGUARD_MTU_VALUE = 1280;
const MAX_WIREGUARD_MTU_VALUE = 1420;
const WIREUGARD_UDP_PORTS = [51820, 53];
const UDP2TCP_PORTS = [80, 443, 5001];

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}

export const StyledContent = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  marginBottom: '2px',
});

export const StyledCellIcon = styled(Cell.UntintedIcon)({
  marginRight: '8px',
});

export const StyledInfoIcon = styled(InfoIcon)({
  marginRight: '16px',
});

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

export default function WireguardSettings() {
  const { pop } = useHistory();

  return (
    <BackAction action={pop}>
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <TitleBarItem>
                  {sprintf(
                    // TRANSLATORS: Title label in navigation bar
                    // TRANSLATORS: Available placeholders:
                    // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
                    messages.pgettext('wireguard-settings-nav', '%(wireguard)s settings'),
                    { wireguard: strings.wireguard },
                  )}
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

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

              <StyledContent>
                <Cell.Group>
                  <PortSelector />
                </Cell.Group>

                <Cell.Group>
                  <ObfuscationSettings />
                  <Udp2tcpPortSetting />
                </Cell.Group>

                <Cell.Group>
                  <QuantumResistantSetting />
                </Cell.Group>

                <Cell.Group>
                  <MultihopSetting />
                </Cell.Group>

                <Cell.Group>
                  <IpVersionSetting />
                </Cell.Group>

                <Cell.Group>
                  <MtuSetting />
                </Cell.Group>
              </StyledContent>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function PortSelector() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const { updateRelaySettings } = useAppContext();
  const allowedPortRanges = useSelector((state) => state.settings.wireguardEndpointData.portRanges);

  const wireguardPortItems = useMemo<Array<SelectorItem<number>>>(
    () => WIREUGARD_UDP_PORTS.map(mapPortToSelectorItem),
    [],
  );

  const port = useMemo(() => {
    const port = 'normal' in relaySettings ? relaySettings.normal.wireguard.port : 'any';
    return port === 'any' ? null : port;
  }, [relaySettings]);

  const setWireguardPort = useCallback(
    async (port: number | null) => {
      const relayUpdate = createWireguardRelayUpdater(relaySettings)
        .tunnel.wireguard((wireguard) => {
          if (port !== null) {
            wireguard.port.exact(port);
          } else {
            wireguard.port.any();
          }
        })
        .build();
      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettings],
  );

  const parseValue = useCallback((port: string) => parseInt(port), []);

  const validateValue = useCallback(
    (value: number) => allowedPortRanges.some(([start, end]) => value >= start && value <= end),
    [allowedPortRanges],
  );

  const portRangesText = allowedPortRanges
    .map(([start, end]) => (start === end ? start : `${start}-${end}`))
    .join(', ');

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <SelectorWithCustomItem
          // TRANSLATORS: The title for the WireGuard port selector.
          title={messages.pgettext('wireguard-settings-view', 'Port')}
          items={wireguardPortItems}
          value={port}
          onSelect={setWireguardPort}
          inputPlaceholder={messages.pgettext('wireguard-settings-view', 'Port')}
          automaticValue={null}
          parseValue={parseValue}
          modifyValue={removeNonNumericCharacters}
          validateValue={validateValue}
          maxLength={5}
          details={
            <>
              <ModalMessage>
                {messages.pgettext(
                  'wireguard-settings-view',
                  'The automatic setting will randomly choose from the valid port ranges shown below.',
                )}
              </ModalMessage>
              <ModalMessage>
                {sprintf(
                  messages.pgettext(
                    'wireguard-settings-view',
                    'The custom port can be any value inside the valid ranges: %(portRanges)s.',
                  ),
                  { portRanges: portRangesText },
                )}
              </ModalMessage>
            </>
          }
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

function ObfuscationSettings() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  const obfuscationType = obfuscationSettings.selectedObfuscation;
  const obfuscationTypeItems: SelectorItem<ObfuscationType>[] = useMemo(
    () => [
      {
        label: messages.pgettext('wireguard-settings-view', 'On (UDP-over-TCP)'),
        value: ObfuscationType.udp2tcp,
      },
      {
        label: messages.gettext('Off'),
        value: ObfuscationType.off,
      },
    ],
    [],
  );

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
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          // TRANSLATORS: The title for the WireGuard obfuscation selector.
          title={messages.pgettext('wireguard-settings-view', 'Obfuscation')}
          details={
            <ModalMessage>
              {messages.pgettext(
                'wireguard-settings-view',
                'Obfuscation hides the WireGuard traffic inside another protocol. It can be used to help circumvent censorship and other types of filtering, where a plain WireGuard connect would be blocked.',
              )}
            </ModalMessage>
          }
          items={obfuscationTypeItems}
          value={obfuscationType}
          onSelect={selectObfuscationType}
          automaticValue={ObfuscationType.auto}
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

function Udp2tcpPortSetting() {
  const { setObfuscationSettings } = useAppContext();
  const obfuscationSettings = useSelector((state) => state.settings.obfuscationSettings);

  const port = liftConstraint(obfuscationSettings.udp2tcpSettings.port);
  const portItems: SelectorItem<number>[] = useMemo(
    () => UDP2TCP_PORTS.map(mapPortToSelectorItem),
    [],
  );

  const expandableProps = useMemo(() => ({ expandable: true, id: 'udp2tcp-port' }), []);

  const selectPort = useCallback(
    async (port: LiftedConstraint<number>) => {
      await setObfuscationSettings({
        ...obfuscationSettings,
        udp2tcpSettings: {
          ...obfuscationSettings.udp2tcpSettings,
          port: port === 'any' ? 'any' : { only: port },
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
          disabled={obfuscationSettings.selectedObfuscation === ObfuscationType.off}
          expandable={expandableProps}
          thinTitle
          automaticValue={'any' as const}
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

function MultihopSetting() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const { updateRelaySettings } = useAppContext();

  const multihop = 'normal' in relaySettings ? relaySettings.normal.wireguard.useMultihop : false;

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

  const setMultihopImpl = useCallback(
    async (enabled: boolean) => {
      const relayUpdate = createWireguardRelayUpdater(relaySettings)
        .tunnel.wireguard((wireguard) => wireguard.useMultihop(enabled))
        .build();
      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update WireGuard multihop settings', error.message);
      }
    },
    [relaySettings, updateRelaySettings],
  );

  const setMultihop = useCallback(
    async (newValue: boolean) => {
      if (newValue) {
        showConfirmationDialog();
      } else {
        await setMultihopImpl(false);
      }
    },
    [setMultihopImpl],
  );

  const confirmMultihop = useCallback(async () => {
    await setMultihopImpl(true);
    hideConfirmationDialog();
  }, [setMultihopImpl]);

  return (
    <>
      <AriaInputGroup>
        <Cell.Container>
          <AriaLabel>
            <Cell.InputLabel>
              {
                // TRANSLATORS: The label next to the multihop settings toggle.
                messages.pgettext('vpn-settings-view', 'Enable multihop')
              }
            </Cell.InputLabel>
          </AriaLabel>
          <AriaInput>
            <Cell.Switch isOn={multihop} onChange={setMultihop} />
          </AriaInput>
        </Cell.Container>
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {sprintf(
                // TRANSLATORS: Description for multihop settings toggle.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
                messages.pgettext(
                  'vpn-settings-view',
                  'Increases anonymity by routing your traffic into one %(wireguard)s server and out another, making it harder to trace.',
                ),
                { wireguard: strings.wireguard },
              )}
            </Cell.CellFooterText>
          </AriaDescription>
        </Cell.CellFooter>
      </AriaInputGroup>
      <ModalAlert
        isOpen={confirmationDialogVisible}
        type={ModalAlertType.info}
        message={
          // TRANSLATORS: Warning text in a dialog that is displayed after a setting is toggled.
          messages.gettext('This setting increases latency. Use only if needed.')
        }
        buttons={[
          <AppButton.RedButton key="confirm" onClick={confirmMultihop}>
            {messages.gettext('Enable anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="back" onClick={hideConfirmationDialog}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ]}
        close={hideConfirmationDialog}
      />
    </>
  );
}

function IpVersionSetting() {
  const { updateRelaySettings } = useAppContext();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const ipVersion = useMemo(() => {
    const ipVersion = 'normal' in relaySettings ? relaySettings.normal.wireguard.ipVersion : 'any';
    return ipVersion === 'any' ? null : ipVersion;
  }, [relaySettings]);

  const ipVersionItems: SelectorItem<IpVersion>[] = useMemo(
    () => [
      {
        label: messages.gettext('IPv4'),
        value: 'ipv4',
      },
      {
        label: messages.gettext('IPv6'),
        value: 'ipv6',
      },
    ],
    [],
  );

  const setIpVersion = useCallback(
    async (ipVersion: IpVersion | null) => {
      const relayUpdate = createWireguardRelayUpdater(relaySettings)
        .tunnel.wireguard((wireguard) => {
          if (ipVersion !== null) {
            wireguard.ipVersion.exact(ipVersion);
          } else {
            wireguard.ipVersion.any();
          }
        })
        .build();
      try {
        await updateRelaySettings(relayUpdate);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettings, updateRelaySettings],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          // TRANSLATORS: The title for the WireGuard IP version selector.
          title={messages.pgettext('wireguard-settings-view', 'IP version')}
          items={ipVersionItems}
          value={ipVersion}
          onSelect={setIpVersion}
          automaticValue={null}
        />
      </StyledSelectorContainer>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {sprintf(
              // TRANSLATORS: The hint displayed below the WireGuard IP version selector.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
              messages.pgettext(
                'wireguard-settings-view',
                'This allows access to %(wireguard)s for devices that only support IPv6.',
              ),
              { wireguard: strings.wireguard },
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

function mtuIsValid(mtu: string): boolean {
  const parsedMtu = mtu ? parseInt(mtu) : undefined;
  return (
    parsedMtu === undefined ||
    (parsedMtu >= MIN_WIREGUARD_MTU_VALUE && parsedMtu <= MAX_WIREGUARD_MTU_VALUE)
  );
}

function MtuSetting() {
  const { setWireguardMtu: setWireguardMtuImpl } = useAppContext();
  const mtu = useSelector((state) => state.settings.wireguard.mtu);

  const setMtu = useCallback(
    async (mtu?: number) => {
      try {
        await setWireguardMtuImpl(mtu);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update mtu value', error.message);
      }
    },
    [setWireguardMtuImpl],
  );

  const onSubmit = useCallback(
    async (value: string) => {
      const parsedValue = value === '' ? undefined : parseInt(value, 10);
      if (mtuIsValid(value)) {
        await setMtu(parsedValue);
      }
    },
    [setMtu],
  );

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>{messages.pgettext('wireguard-settings-view', 'MTU')}</Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.AutoSizingTextInput
            initialValue={mtu ? mtu.toString() : ''}
            inputMode={'numeric'}
            maxLength={4}
            placeholder={messages.gettext('Default')}
            onSubmitValue={onSubmit}
            validateValue={mtuIsValid}
            submitOnBlur={true}
            modifyValue={removeNonNumericCharacters}
          />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {sprintf(
              // TRANSLATORS: The hint displayed below the WireGuard MTU input field.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(wireguard)s - Will be replaced with the string "WireGuard"
              // TRANSLATORS: %(max)d - the maximum possible wireguard mtu value
              // TRANSLATORS: %(min)d - the minimum possible wireguard mtu value
              messages.pgettext(
                'wireguard-settings-view',
                'Set %(wireguard)s MTU value. Valid range: %(min)d - %(max)d.',
              ),
              {
                wireguard: strings.wireguard,
                min: MIN_WIREGUARD_MTU_VALUE,
                max: MAX_WIREGUARD_MTU_VALUE,
              },
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

function QuantumResistantSetting() {
  const { setWireguardQuantumResistant } = useAppContext();
  const quantumResistant = useSelector((state) => state.settings.wireguard.quantumResistant);

  const items: SelectorItem<boolean>[] = useMemo(
    () => [
      {
        label: messages.gettext('On'),
        value: true,
      },
      {
        label: messages.gettext('Off'),
        value: false,
      },
    ],
    [],
  );

  const selectQuantumResistant = useCallback(
    async (quantumResistant: boolean | null) => {
      await setWireguardQuantumResistant(quantumResistant ?? undefined);
    },
    [setWireguardQuantumResistant],
  );

  return (
    <AriaInputGroup>
      <StyledSelectorContainer>
        <Selector
          title={
            // TRANSLATORS: The title for the WireGuard quantum resistance selector. This setting
            // TRANSLATORS: makes the cryptography resistant to the future abilities of quantum
            // TRANSLATORS: computers.
            messages.pgettext('wireguard-settings-view', 'Quantum-resistant tunnel')
          }
          details={
            <>
              <ModalMessage>
                {messages.pgettext(
                  'wireguard-settings-view',
                  'This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.',
                )}
              </ModalMessage>
              <ModalMessage>
                {messages.pgettext(
                  'wireguard-settings-view',
                  'It does this by performing an extra key exchange using a quantum safe algorithm and mixing the result into WireGuard’s regular encryption. This extra step uses approximately 500 kiB of traffic every time a new tunnel is established.',
                )}
              </ModalMessage>
            </>
          }
          items={items}
          value={quantumResistant ?? null}
          onSelect={selectQuantumResistant}
          automaticValue={null}
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

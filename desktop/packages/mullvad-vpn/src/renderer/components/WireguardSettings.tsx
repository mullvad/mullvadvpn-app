import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../shared/constants';
import {
  Constraint,
  IpVersion,
  ObfuscationType,
  wrapConstraint,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { removeNonNumericCharacters } from '../../shared/string-helpers';
import { useAppContext } from '../context';
import { useRelaySettingsUpdater } from '../lib/constraint-updater';
import { useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useSelector } from '../redux/store';
import { AppNavigationHeader } from './';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import Selector, { SelectorItem, SelectorWithCustomItem } from './cell/Selector';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer, SettingsContent, SettingsGroup, SettingsStack } from './Layout';
import { ModalMessage } from './Modal';
import { NavigationContainer } from './NavigationContainer';
import { NavigationScrollbars } from './NavigationScrollbars';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const MIN_WIREGUARD_MTU_VALUE = 1280;
const MAX_WIREGUARD_MTU_VALUE = 1420;
const WIREUGARD_UDP_PORTS = [51820, 53];

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}

const StyledSelectorContainer = styled.div({
  flex: 0,
});

export default function WireguardSettings() {
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

function PortSelector() {
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const relaySettingsUpdater = useRelaySettingsUpdater();
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
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.port = wrapConstraint(port);
          return settings;
        });
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettingsUpdater],
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

  // TRANSLATORS: Text showing currently selected port.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(port)s - Can be either a number between 1 and 65535 or the text "Automatic".
  const subLabelTemplate = messages.pgettext('wireguard-settings-view', 'Port: %(port)s');

  const obfuscationType = obfuscationSettings.selectedObfuscation;
  const obfuscationTypeItems: SelectorItem<ObfuscationType>[] = useMemo(
    () => [
      {
        label: messages.pgettext('wireguard-settings-view', 'Shadowsocks'),
        subLabel: sprintf(subLabelTemplate, {
          port: formatPortForSubLabel(obfuscationSettings.shadowsocksSettings.port),
        }),
        value: ObfuscationType.shadowsocks,
        details: {
          path: RoutePath.shadowsocks,
          ariaLabel: messages.pgettext('accessibility', 'Shadowsocks settings'),
        },
      },
      {
        label: messages.pgettext('wireguard-settings-view', 'UDP-over-TCP'),
        subLabel: sprintf(subLabelTemplate, {
          port: formatPortForSubLabel(obfuscationSettings.udp2tcpSettings.port),
        }),
        value: ObfuscationType.udp2tcp,
        details: {
          path: RoutePath.udpOverTcp,
          ariaLabel: messages.pgettext('accessibility', 'UDP-over-TCP settings'),
        },
      },
      {
        label: messages.gettext('Off'),
        value: ObfuscationType.off,
      },
    ],
    [
      obfuscationSettings.shadowsocksSettings.port,
      obfuscationSettings.udp2tcpSettings.port,
      subLabelTemplate,
    ],
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
                'Obfuscation hides the WireGuard traffic inside another protocol. It can be used to help circumvent censorship and other types of filtering, where a plain WireGuard connection would be blocked.',
              )}
            </ModalMessage>
          }
          items={obfuscationTypeItems}
          value={obfuscationType}
          onSelect={selectObfuscationType}
          automaticValue={ObfuscationType.auto}
          automaticTestId="automatic-obfuscation"
        />
      </StyledSelectorContainer>
    </AriaInputGroup>
  );
}

function formatPortForSubLabel(port: Constraint<number>): string {
  return port === 'any' ? messages.gettext('Automatic') : `${port.only}`;
}

function IpVersionSetting() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
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
      try {
        await relaySettingsUpdater((settings) => {
          settings.wireguardConstraints.ipVersion = wrapConstraint(ipVersion);
          return settings;
        });
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update relay settings', error.message);
      }
    },
    [relaySettingsUpdater],
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

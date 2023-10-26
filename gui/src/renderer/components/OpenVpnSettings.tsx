import { useCallback, useMemo } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { strings } from '../../config.json';
import {
  BridgeState,
  RelayProtocol,
  TunnelProtocol,
  wrapConstraint,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { removeNonNumericCharacters } from '../../shared/string-helpers';
import { useAppContext } from '../context';
import { useRelaySettingsUpdater } from '../lib/constraint-updater';
import { useHistory } from '../lib/history';
import { formatHtml } from '../lib/html-formatter';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import Selector, { SelectorItem } from './cell/Selector';
import { BackAction } from './KeyboardNavigation';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import {
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const MIN_MSSFIX_VALUE = 1000;
const MAX_MSSFIX_VALUE = 1450;
const UDP_PORTS = [1194, 1195, 1196, 1197, 1300, 1301, 1302];
const TCP_PORTS = [80, 443];

export enum BridgeModeAvailability {
  available,
  blockedDueToTunnelProtocol,
  blockedDueToTransportProtocol,
}

function mapPortToSelectorItem(value: number): SelectorItem<number> {
  return { label: value.toString(), value };
}

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

export default function OpenVpnSettings() {
  const { pop } = useHistory();

  const relaySettings = useSelector((state) => state.settings.relaySettings);

  const protocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.openvpn.protocol : undefined;
    return protocol === 'any' ? undefined : protocol;
  }, [relaySettings]);

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
                    // TRANSLATORS: %(openvpn)s - Will be replaced with "OpenVPN"
                    messages.pgettext('openvpn-settings-nav', '%(openvpn)s settings'),
                    { openvpn: strings.openvpn },
                  )}
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <NavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {sprintf(
                    // TRANSLATORS: %(openvpn)s will be replaced with "OpenVPN"
                    messages.pgettext('openvpn-settings-view', '%(openvpn)s settings'),
                    {
                      openvpn: strings.openvpn,
                    },
                  )}
                </HeaderTitle>
              </SettingsHeader>

              <Cell.Group>
                <TransportProtocolSelector />
              </Cell.Group>

              {protocol ? (
                <Cell.Group>
                  <PortSelector />
                </Cell.Group>
              ) : undefined}

              <Cell.Group>
                <BridgeModeSelector />
              </Cell.Group>

              <Cell.Group>
                <MssFixSetting />
              </Cell.Group>
            </NavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    </BackAction>
  );
}

function TransportProtocolSelector() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);
  const bridgeState = useSelector((state) => state.settings.bridgeState);

  const protocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.openvpn.protocol : 'any';
    return protocol === 'any' ? null : protocol;
  }, [relaySettings]);

  const onSelect = useCallback(
    async (protocol: RelayProtocol | null) => {
      await relaySettingsUpdater((settings) => {
        settings.openvpnConstraints.protocol = wrapConstraint(protocol);
        return settings;
      });
    },
    [relaySettingsUpdater],
  );

  const items: SelectorItem<RelayProtocol>[] = useMemo(
    () => [
      {
        label: messages.gettext('TCP'),
        value: 'tcp',
      },
      {
        label: messages.gettext('UDP'),
        value: 'udp',
        disabled: bridgeState === 'on',
      },
    ],
    [bridgeState],
  );

  return (
    <StyledSelectorContainer>
      <AriaInputGroup>
        <Selector
          title={messages.pgettext('openvpn-settings-view', 'Transport protocol')}
          items={items}
          value={protocol}
          onSelect={onSelect}
          automaticValue={null}
        />
        {bridgeState === 'on' && (
          <Cell.CellFooter>
            <AriaDescription>
              <Cell.CellFooterText>
                {formatHtml(
                  // TRANSLATORS: This is used to instruct users how to make UDP mode
                  // TRANSLATORS: available.
                  messages.pgettext(
                    'openvpn-settings-view',
                    'To activate UDP, change <b>Bridge mode</b> to <b>Automatic</b> or <b>Off</b>.',
                  ),
                )}
              </Cell.CellFooterText>
            </AriaDescription>
          </Cell.CellFooter>
        )}
      </AriaInputGroup>
    </StyledSelectorContainer>
  );
}

function PortSelector() {
  const relaySettingsUpdater = useRelaySettingsUpdater();
  const relaySettings = useSelector((state) => state.settings.relaySettings);

  const protocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.openvpn.protocol : 'any';
    return protocol === 'any' ? null : protocol;
  }, [relaySettings]);

  const port = useMemo(() => {
    const port = 'normal' in relaySettings ? relaySettings.normal.openvpn.port : 'any';
    return port === 'any' ? null : port;
  }, [relaySettings]);

  const onSelect = useCallback(
    async (port: number | null) => {
      await relaySettingsUpdater((settings) => {
        settings.openvpnConstraints.port = wrapConstraint(port);
        return settings;
      });
    },
    [relaySettingsUpdater],
  );

  const portItems = {
    udp: UDP_PORTS.map(mapPortToSelectorItem),
    tcp: TCP_PORTS.map(mapPortToSelectorItem),
  };

  if (protocol === null) {
    return null;
  }

  return (
    <StyledSelectorContainer>
      <AriaInputGroup>
        <Selector
          title={sprintf(
            // TRANSLATORS: The title for the port selector section.
            // TRANSLATORS: Available placeholders:
            // TRANSLATORS: %(portType)s - a selected protocol (either TCP or UDP)
            messages.pgettext('openvpn-settings-view', '%(portType)s port'),
            {
              portType: protocol.toUpperCase(),
            },
          )}
          items={portItems[protocol]}
          value={port}
          onSelect={onSelect}
          automaticValue={null}
        />
      </AriaInputGroup>
    </StyledSelectorContainer>
  );
}

function BridgeModeSelector() {
  const { setBridgeState: setBridgeStateImpl } = useAppContext();
  const relaySettings = useSelector((state) => state.settings.relaySettings);

  const bridgeState = useSelector((state) => state.settings.bridgeState);

  const tunnelProtocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.tunnelProtocol : 'any';
    return protocol === 'any' ? null : protocol;
  }, [relaySettings]);

  const transportProtocol = useMemo(() => {
    const protocol = 'normal' in relaySettings ? relaySettings.normal.openvpn.protocol : 'any';
    return protocol === 'any' ? null : protocol;
  }, [relaySettings]);

  const options: SelectorItem<BridgeState>[] = useMemo(
    () => [
      {
        label: messages.gettext('On'),
        value: 'on',
        disabled: tunnelProtocol !== 'openvpn' || transportProtocol === 'udp',
      },
      {
        label: messages.gettext('Off'),
        value: 'off',
      },
    ],
    [tunnelProtocol, transportProtocol],
  );

  const [confirmationDialogVisible, showConfirmationDialog, hideConfirmationDialog] = useBoolean();

  const setBridgeState = useCallback(
    async (bridgeState: BridgeState) => {
      try {
        await setBridgeStateImpl(bridgeState);
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to update bridge state: ${error.message}`);
      }
    },
    [setBridgeStateImpl],
  );

  const onSelectBridgeState = useCallback(
    async (newValue: BridgeState) => {
      if (newValue === 'on') {
        showConfirmationDialog();
      } else {
        await setBridgeState(newValue);
      }
    },
    [showConfirmationDialog, setBridgeState],
  );

  const confirmBridgeState = useCallback(async () => {
    hideConfirmationDialog();
    await setBridgeState('on');
  }, [hideConfirmationDialog, setBridgeState]);

  return (
    <>
      <AriaInputGroup>
        <StyledSelectorContainer>
          <Selector
            title={
              // TRANSLATORS: The title for the shadowsocks bridge selector section.
              messages.pgettext('openvpn-settings-view', 'Bridge mode')
            }
            items={options}
            value={bridgeState}
            onSelect={onSelectBridgeState}
            automaticValue={'auto' as const}
          />
        </StyledSelectorContainer>
        <Cell.CellFooter>
          <AriaDescription>
            <Cell.CellFooterText>
              {bridgeModeFooterText(tunnelProtocol, transportProtocol)}
            </Cell.CellFooterText>
          </AriaDescription>
        </Cell.CellFooter>
      </AriaInputGroup>
      <ModalAlert
        isOpen={confirmationDialogVisible}
        type={ModalAlertType.caution}
        message={messages.gettext('This setting increases latency. Use only if needed.')}
        buttons={[
          <AppButton.RedButton key="confirm" onClick={confirmBridgeState}>
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

function bridgeModeFooterText(
  tunnelProtocol: TunnelProtocol | null,
  transportProtocol: RelayProtocol | null,
) {
  if (tunnelProtocol !== 'openvpn') {
    return formatHtml(
      sprintf(
        // TRANSLATORS: This is used to instruct users how to make the bridge mode setting
        // TRANSLATORS: available.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(tunnelProtocol)s - the name of the tunnel protocol setting
        // TRANSLATORS: %(openvpn)s - will be replaced with OpenVPN
        messages.pgettext(
          'openvpn-settings-view',
          'To activate Bridge mode, go back and change <b>%(tunnelProtocol)s</b> to <b>%(openvpn)s</b>.',
        ),
        {
          tunnelProtocol: messages.pgettext('vpn-settings-view', 'Tunnel protocol'),
          openvpn: strings.openvpn,
        },
      ),
    );
  } else if (transportProtocol === 'udp') {
    return formatHtml(
      sprintf(
        // TRANSLATORS: This is used to instruct users how to make the bridge mode setting
        // TRANSLATORS: available.
        // TRANSLATORS: Available placeholders:
        // TRANSLATORS: %(transportProtocol)s - the name of the transport protocol setting
        // TRANSLATORS: %(automat)s - the translation of "Automatic"
        // TRANSLATORS: %(openvpn)s - will be replaced with OpenVPN
        messages.pgettext(
          'openvpn-settings-view',
          'To activate Bridge mode, change <b>%(transportProtocol)s</b> to <b>%(automatic)s</b> or <b>%(tcp)s</b>.',
        ),
        {
          transportProtocol: messages.pgettext('openvpn-settings-view', 'Transport protocol'),
          automatic: messages.gettext('Automatic'),
          tcp: messages.gettext('TCP'),
        },
      ),
    );
  } else {
    return sprintf(
      // TRANSLATORS: This is used as a description for the bridge mode
      // TRANSLATORS: setting.
      // TRANSLATORS: Available placeholders:
      // TRANSLATORS: %(openvpn)s - will be replaced with OpenVPN
      messages.pgettext(
        'openvpn-settings-view',
        'Helps circumvent censorship, by routing your traffic through a bridge server before reaching an %(openvpn)s server. Obfuscation is added to make fingerprinting harder.',
      ),
      { openvpn: strings.openvpn },
    );
  }
}

function mssfixIsValid(mssfix: string): boolean {
  const parsedMssFix = mssfix ? parseInt(mssfix) : undefined;
  return (
    parsedMssFix === undefined ||
    (parsedMssFix >= MIN_MSSFIX_VALUE && parsedMssFix <= MAX_MSSFIX_VALUE)
  );
}

function MssFixSetting() {
  const { setOpenVpnMssfix: setOpenVpnMssfixImpl } = useAppContext();
  const mssfix = useSelector((state) => state.settings.openVpn.mssfix);

  const setOpenVpnMssfix = useCallback(
    async (mssfix?: number) => {
      try {
        await setOpenVpnMssfixImpl(mssfix);
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update mssfix value', error.message);
      }
    },
    [setOpenVpnMssfixImpl],
  );

  const onMssfixSubmit = useCallback(
    async (value: string) => {
      const parsedValue = value === '' ? undefined : parseInt(value, 10);
      if (mssfixIsValid(value)) {
        await setOpenVpnMssfix(parsedValue);
      }
    },
    [setOpenVpnMssfix],
  );

  return (
    <AriaInputGroup>
      <Cell.Container>
        <AriaLabel>
          <Cell.InputLabel>{messages.pgettext('openvpn-settings-view', 'Mssfix')}</Cell.InputLabel>
        </AriaLabel>
        <AriaInput>
          <Cell.AutoSizingTextInput
            initialValue={mssfix ? mssfix.toString() : ''}
            inputMode={'numeric'}
            maxLength={4}
            placeholder={messages.gettext('Default')}
            onSubmitValue={onMssfixSubmit}
            validateValue={mssfixIsValid}
            submitOnBlur={true}
            modifyValue={removeNonNumericCharacters}
          />
        </AriaInput>
      </Cell.Container>
      <Cell.CellFooter>
        <AriaDescription>
          <Cell.CellFooterText>
            {sprintf(
              // TRANSLATORS: The hint displayed below the Mssfix input field.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(openvpn)s - will be replaced with "OpenVPN"
              // TRANSLATORS: %(max)d - the maximum possible mssfix value
              // TRANSLATORS: %(min)d - the minimum possible mssfix value
              messages.pgettext(
                'openvpn-settings-view',
                'Set %(openvpn)s MSS value. Valid range: %(min)d - %(max)d.',
              ),
              {
                openvpn: strings.openvpn,
                min: MIN_MSSFIX_VALUE,
                max: MAX_MSSFIX_VALUE,
              },
            )}
          </Cell.CellFooterText>
        </AriaDescription>
      </Cell.CellFooter>
    </AriaInputGroup>
  );
}

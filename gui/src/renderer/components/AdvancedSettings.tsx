import ip from 'ip';
import * as React from 'react';
import { sprintf } from 'sprintf-js';
import { colors } from '../../config.json';
import {
  BridgeState,
  IDnsOptions,
  RelayProtocol,
  TunnelProtocol,
} from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import consumePromise from '../../shared/promise';
import { WgKeyState } from '../redux/settings/reducers';
import {
  StyledButtonCellGroup,
  StyledContainer,
  StyledInputFrame,
  StyledNavigationScrollbars,
  StyledNoWireguardKeyError,
  StyledNoWireguardKeyErrorContainer,
  StyledSelectorContainer,
  StyledTunnelProtocolSelector,
  StyledTunnelProtocolContainer,
  StyledCustomDnsSwitchContainer,
  StyledCustomDnsFotter,
  StyledAddCustomDnsLabel,
  StyledAddCustomDnsButton,
} from './AdvancedSettingsStyles';
import * as AppButton from './AppButton';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import CellList, { ICellListItem } from './cell/List';
import { Layout } from './Layout';
import { ModalAlert, ModalAlertType, ModalContainer, ModalMessage } from './Modal';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  TitleBarItem,
} from './NavigationBar';
import Selector, { ISelectorItem } from './cell/Selector';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import Accordion from './Accordion';

const MIN_MSSFIX_VALUE = 1000;
const MAX_MSSFIX_VALUE = 1450;
const MIN_WIREGUARD_MTU_VALUE = 1280;
const MAX_WIREGUARD_MTU_VALUE = 1420;
const UDP_PORTS = [1194, 1195, 1196, 1197, 1300, 1301, 1302];
const TCP_PORTS = [80, 443];
const WIREUGARD_UDP_PORTS = [53];

type OptionalPort = number | undefined;

type OptionalRelayProtocol = RelayProtocol | undefined;
type OptionalTunnelProtocol = TunnelProtocol | undefined;

function mapPortToSelectorItem(value: number): ISelectorItem<number> {
  return { label: value.toString(), value };
}

interface IProps {
  enableIpv6: boolean;
  blockWhenDisconnected: boolean;
  tunnelProtocol?: TunnelProtocol;
  openvpn: {
    protocol?: RelayProtocol;
    port?: number;
  };
  wireguardKeyState: WgKeyState;
  wireguard: { port?: number };
  mssfix?: number;
  wireguardMtu?: number;
  bridgeState: BridgeState;
  dns: IDnsOptions;
  setBridgeState: (value: BridgeState) => void;
  setEnableIpv6: (value: boolean) => void;
  setBlockWhenDisconnected: (value: boolean) => void;
  setTunnelProtocol: (value: OptionalTunnelProtocol) => void;
  setOpenVpnMssfix: (value: number | undefined) => void;
  setWireguardMtu: (value: number | undefined) => void;
  setOpenVpnRelayProtocolAndPort: (protocol?: RelayProtocol, port?: number) => void;
  setWireguardRelayPort: (port?: number) => void;
  setDnsOptions: (dns: IDnsOptions) => Promise<void>;
  onViewWireguardKeys: () => void;
  onViewLinuxSplitTunneling: () => void;
  onClose: () => void;
}

interface IState {
  showConfirmBlockWhenDisconnectedAlert: boolean;
  showAddCustomDns: boolean;
  invalidDnsIp: boolean;
  publicDnsIpToConfirm?: string;
}

export default class AdvancedSettings extends React.Component<IProps, IState> {
  public state = {
    showConfirmBlockWhenDisconnectedAlert: false,
    showAddCustomDns: false,
    invalidDnsIp: false,
    publicDnsIpToConfirm: undefined,
  };

  private customDnsSwitchRef = React.createRef<HTMLDivElement>();
  private customDnsAddButtonRef = React.createRef<HTMLButtonElement>();
  private customDnsInputContainerRef = React.createRef<HTMLDivElement>();

  private portItems: { [key in RelayProtocol]: Array<ISelectorItem<OptionalPort>> };
  private protocolItems: Array<ISelectorItem<OptionalRelayProtocol>>;
  private bridgeStateItems: Array<ISelectorItem<BridgeState>>;
  private wireguardPortItems: Array<ISelectorItem<OptionalPort>>;

  constructor(props: IProps) {
    super(props);

    const automaticPort: ISelectorItem<OptionalPort> = {
      label: messages.pgettext('advanced-settings-view', 'Automatic'),
      value: undefined,
    };

    this.portItems = {
      udp: [automaticPort].concat(UDP_PORTS.map(mapPortToSelectorItem)),
      tcp: [automaticPort].concat(TCP_PORTS.map(mapPortToSelectorItem)),
    };

    this.wireguardPortItems = [automaticPort].concat(
      WIREUGARD_UDP_PORTS.map(mapPortToSelectorItem),
    );

    this.protocolItems = [
      {
        label: messages.pgettext('advanced-settings-view', 'Automatic'),
        value: undefined,
      },
      {
        label: messages.pgettext('advanced-settings-view', 'TCP'),
        value: 'tcp',
      },
      {
        label: messages.pgettext('advanced-settings-view', 'UDP'),
        value: 'udp',
      },
    ];

    this.wireguardPortItems = [automaticPort].concat(
      WIREUGARD_UDP_PORTS.map(mapPortToSelectorItem),
    );

    this.bridgeStateItems = [
      {
        label: messages.pgettext('advanced-settings-view', 'Automatic'),
        value: 'auto',
      },
      {
        label: messages.pgettext('advanced-settings-view', 'On'),
        value: 'on',
      },
      {
        label: messages.pgettext('advanced-settings-view', 'Off'),
        value: 'off',
      },
    ];
  }

  public render() {
    const hasWireguardKey = this.props.wireguardKeyState.type === 'key-set';

    return (
      <ModalContainer>
        <Layout>
          <StyledContainer>
            <NavigationContainer>
              <NavigationBar>
                <NavigationItems>
                  <BackBarItem action={this.props.onClose}>
                    {
                      // TRANSLATORS: Back button in navigation bar
                      messages.pgettext('navigation-bar', 'Settings')
                    }
                  </BackBarItem>
                  <TitleBarItem>
                    {
                      // TRANSLATORS: Title label in navigation bar
                      messages.pgettext('advanced-settings-nav', 'Advanced')
                    }
                  </TitleBarItem>
                </NavigationItems>
              </NavigationBar>

              <StyledNavigationScrollbars>
                <SettingsHeader>
                  <HeaderTitle>
                    {messages.pgettext('advanced-settings-view', 'Advanced')}
                  </HeaderTitle>
                </SettingsHeader>

                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('advanced-settings-view', 'Enable IPv6')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch
                        isOn={this.props.enableIpv6}
                        onChange={this.props.setEnableIpv6}
                      />
                    </AriaInput>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'advanced-settings-view',
                          'Enable IPv6 communication through the tunnel.',
                        )}
                      </Cell.FooterText>
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>

                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('advanced-settings-view', 'Always require VPN')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch
                        isOn={this.props.blockWhenDisconnected}
                        onChange={this.setBlockWhenDisconnected}
                      />
                    </AriaInput>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {messages.pgettext(
                          'advanced-settings-view',
                          'If you disconnect or quit the app, this setting will block your internet.',
                        )}
                      </Cell.FooterText>
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>

                <AriaInputGroup>
                  <StyledTunnelProtocolContainer>
                    <StyledTunnelProtocolSelector
                      title={messages.pgettext('advanced-settings-view', 'Tunnel protocol')}
                      values={this.tunnelProtocolItems(hasWireguardKey)}
                      value={this.props.tunnelProtocol}
                      onSelect={this.onSelectTunnelProtocol}
                    />
                    {!hasWireguardKey && (
                      <StyledNoWireguardKeyErrorContainer>
                        <AriaDescription>
                          <StyledNoWireguardKeyError>
                            {messages.pgettext(
                              'advanced-settings-view',
                              'To enable WireGuard, generate a key under the "WireGuard key" setting below.',
                            )}
                          </StyledNoWireguardKeyError>
                        </AriaDescription>
                      </StyledNoWireguardKeyErrorContainer>
                    )}
                  </StyledTunnelProtocolContainer>
                </AriaInputGroup>

                {this.props.tunnelProtocol !== 'wireguard' ? (
                  <AriaInputGroup>
                    <StyledSelectorContainer>
                      <Selector
                        title={messages.pgettext(
                          'advanced-settings-view',
                          'OpenVPN transport protocol',
                        )}
                        values={this.protocolItems}
                        value={this.props.openvpn.protocol}
                        onSelect={this.onSelectOpenvpnProtocol}
                      />

                      {this.props.openvpn.protocol ? (
                        <Selector
                          title={sprintf(
                            // TRANSLATORS: The title for the port selector section.
                            // TRANSLATORS: Available placeholders:
                            // TRANSLATORS: %(portType)s - a selected protocol (either TCP or UDP)
                            messages.pgettext(
                              'advanced-settings-view',
                              'OpenVPN %(portType)s port',
                            ),
                            {
                              portType: this.props.openvpn.protocol.toUpperCase(),
                            },
                          )}
                          values={this.portItems[this.props.openvpn.protocol]}
                          value={this.props.openvpn.port}
                          onSelect={this.onSelectOpenVpnPort}
                        />
                      ) : undefined}
                    </StyledSelectorContainer>
                  </AriaInputGroup>
                ) : undefined}

                {this.props.tunnelProtocol === 'wireguard' ? (
                  <AriaInputGroup>
                    <StyledSelectorContainer>
                      <Selector
                        // TRANSLATORS: The title for the shadowsocks bridge selector section.
                        title={messages.pgettext('advanced-settings-view', 'WireGuard port')}
                        values={this.wireguardPortItems}
                        value={this.props.wireguard.port}
                        onSelect={this.onSelectWireguardPort}
                      />
                    </StyledSelectorContainer>
                  </AriaInputGroup>
                ) : undefined}

                <AriaInputGroup>
                  <Selector
                    title={
                      // TRANSLATORS: The title for the shadowsocks bridge selector section.
                      messages.pgettext('advanced-settings-view', 'Bridge mode')
                    }
                    values={this.bridgeStateItems}
                    value={this.props.bridgeState}
                    onSelect={this.onSelectBridgeState}
                  />
                </AriaInputGroup>

                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('advanced-settings-view', 'OpenVPN Mssfix')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <StyledInputFrame>
                      <AriaInput>
                        <Cell.AutoSizingTextInput
                          value={this.props.mssfix ? this.props.mssfix.toString() : ''}
                          inputMode={'numeric'}
                          maxLength={4}
                          placeholder={messages.pgettext('advanced-settings-view', 'Default')}
                          onSubmitValue={this.onMssfixSubmit}
                          validateValue={AdvancedSettings.mssfixIsValid}
                          submitOnBlur={true}
                          modifyValue={AdvancedSettings.removeNonNumericCharacters}
                        />
                      </AriaInput>
                    </StyledInputFrame>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {sprintf(
                          // TRANSLATORS: The hint displayed below the Mssfix input field.
                          // TRANSLATORS: Available placeholders:
                          // TRANSLATORS: %(max)d - the maximum possible mssfix value
                          // TRANSLATORS: %(min)d - the minimum possible mssfix value
                          messages.pgettext(
                            'advanced-settings-view',
                            'Set OpenVPN MSS value. Valid range: %(min)d - %(max)d.',
                          ),
                          {
                            min: MIN_MSSFIX_VALUE,
                            max: MAX_MSSFIX_VALUE,
                          },
                        )}
                      </Cell.FooterText>
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>

                <AriaInputGroup>
                  <Cell.Container>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('advanced-settings-view', 'WireGuard MTU')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <StyledInputFrame>
                      <AriaInput>
                        <Cell.AutoSizingTextInput
                          value={this.props.wireguardMtu ? this.props.wireguardMtu.toString() : ''}
                          inputMode={'numeric'}
                          maxLength={4}
                          placeholder={messages.pgettext('advanced-settings-view', 'Default')}
                          onSubmitValue={this.onWireguardMtuSubmit}
                          validateValue={AdvancedSettings.wireguarMtuIsValid}
                          submitOnBlur={true}
                          modifyValue={AdvancedSettings.removeNonNumericCharacters}
                        />
                      </AriaInput>
                    </StyledInputFrame>
                  </Cell.Container>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {sprintf(
                          // TRANSLATORS: The hint displayed below the WireGuard MTU input field.
                          // TRANSLATORS: Available placeholders:
                          // TRANSLATORS: %(max)d - the maximum possible wireguard mtu value
                          // TRANSLATORS: %(min)d - the minimum possible wireguard mtu value
                          messages.pgettext(
                            'advanced-settings-view',
                            'Set WireGuard MTU value. Valid range: %(min)d - %(max)d.',
                          ),
                          {
                            min: MIN_WIREGUARD_MTU_VALUE,
                            max: MAX_WIREGUARD_MTU_VALUE,
                          },
                        )}
                      </Cell.FooterText>
                    </AriaDescription>
                  </Cell.Footer>
                </AriaInputGroup>

                <StyledButtonCellGroup>
                  <Cell.CellButton onClick={this.props.onViewWireguardKeys}>
                    <Cell.Label>
                      {messages.pgettext('advanced-settings-view', 'WireGuard key')}
                    </Cell.Label>
                    <Cell.Icon height={12} width={7} source="icon-chevron" />
                  </Cell.CellButton>

                  {window.platform === 'linux' && (
                    <Cell.CellButton onClick={this.props.onViewLinuxSplitTunneling}>
                      <Cell.Label>
                        {messages.pgettext('advanced-settings-view', 'Split tunneling')}
                      </Cell.Label>
                      <Cell.Icon height={12} width={7} source="icon-chevron" />
                    </Cell.CellButton>
                  )}
                </StyledButtonCellGroup>

                <StyledCustomDnsSwitchContainer>
                  <AriaInputGroup>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('advanced-settings-view', 'Use custom DNS server')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch
                        ref={this.customDnsSwitchRef}
                        isOn={this.props.dns.custom}
                        onChange={this.setCustomDnsEnabled}
                      />
                    </AriaInput>
                  </AriaInputGroup>
                </StyledCustomDnsSwitchContainer>
                <Accordion expanded={this.props.dns.custom}>
                  <CellList items={this.customDnsItems()} onRemove={this.removeDnsAddress} />

                  {this.state.showAddCustomDns && (
                    <div ref={this.customDnsInputContainerRef}>
                      <Cell.RowInput
                        onSubmit={this.addDnsAddress}
                        onChange={this.addDnsInputChange}
                        invalid={this.state.invalidDnsIp}
                        paddingLeft={32}
                        onBlur={this.customDnsInputBlur}
                        autofocus
                      />
                    </div>
                  )}

                  <StyledAddCustomDnsButton
                    ref={this.customDnsAddButtonRef}
                    onClick={this.showAddCustomDnsRow}
                    disabled={this.state.showAddCustomDns}
                    tabIndex={-1}>
                    <StyledAddCustomDnsLabel tabIndex={-1}>
                      {messages.pgettext('advanced-settings-view', 'Add a server')}
                    </StyledAddCustomDnsLabel>
                    <Cell.Icon
                      source="icon-add"
                      width={22}
                      height={22}
                      tintColor={colors.white60}
                      tintHoverColor={colors.white80}
                      tabIndex={-1}
                    />
                  </StyledAddCustomDnsButton>
                </Accordion>

                <StyledCustomDnsFotter>
                  <Cell.FooterText>
                    {messages.pgettext(
                      'advanced-settings-view',
                      'Enable to add at least one DNS server.',
                    )}
                  </Cell.FooterText>
                </StyledCustomDnsFotter>
              </StyledNavigationScrollbars>
            </NavigationContainer>
          </StyledContainer>
        </Layout>

        {this.state.showConfirmBlockWhenDisconnectedAlert &&
          this.renderConfirmBlockWhenDisconnectedAlert()}
        {this.state.publicDnsIpToConfirm && this.renderCustomDnsConfirmationDialog()}
      </ModalContainer>
    );
  }

  private setCustomDnsEnabled = async (enabled: boolean) => {
    await this.props.setDnsOptions({
      custom: enabled,
      addresses: this.props.dns.addresses,
    });

    if (enabled && this.props.dns.addresses.length === 0) {
      this.showAddCustomDnsRow();
    }

    if (!enabled) {
      this.setState({ showAddCustomDns: false });
    }
  };

  private customDnsItems(): ICellListItem<string>[] {
    return this.props.dns.addresses.map((address) => ({
      label: address,
      value: address,
    }));
  }

  private showAddCustomDnsRow = () => {
    this.setState({ showAddCustomDns: true });
  };

  // The input field should be hidden when it loses focus unless something on the same row or the
  // add-button is the new focused element.
  private customDnsInputBlur = (event?: React.FocusEvent<HTMLTextAreaElement>) => {
    const relatedTarget = event?.relatedTarget as Node | undefined;
    if (
      relatedTarget &&
      (this.customDnsSwitchRef.current?.contains(relatedTarget) ||
        this.customDnsAddButtonRef.current?.contains(relatedTarget) ||
        this.customDnsInputContainerRef.current?.contains(relatedTarget))
    ) {
      event?.target.focus();
    } else {
      this.hideAddCustomDnsRow(false);
    }
  };

  private hideAddCustomDnsRow(justAdded: boolean) {
    if (!this.state.publicDnsIpToConfirm) {
      this.setState({ showAddCustomDns: false });
      if (!justAdded && this.props.dns.addresses.length === 0) {
        consumePromise(this.setCustomDnsEnabled(false));
      }
    }
  }

  private addDnsInputChange = (_value: string) => {
    this.setState({ invalidDnsIp: false });
  };

  private hideCustomDnsConfirmationDialog = () => {
    this.setState({ publicDnsIpToConfirm: undefined });
  };

  private confirmPublicDnsAddress = () => {
    consumePromise(this.addDnsAddress(this.state.publicDnsIpToConfirm!, true));
    this.hideCustomDnsConfirmationDialog();
  };

  private addDnsAddress = async (address: string, confirmed?: boolean) => {
    if (ip.isV4Format(address) || ip.isV6Format(address)) {
      if (ip.isPublic(address) && !confirmed) {
        this.setState({ publicDnsIpToConfirm: address });
      } else {
        try {
          await this.props.setDnsOptions({
            custom: this.props.dns.custom,
            addresses: [...this.props.dns.addresses, address],
          });
          this.hideAddCustomDnsRow(true);
        } catch (_e) {
          this.setState({ invalidDnsIp: true });
        }
      }
    } else {
      this.setState({ invalidDnsIp: true });
    }
  };

  private removeDnsAddress = (address: string) => {
    const addresses = this.props.dns.addresses.filter((item) => item !== address);
    consumePromise(
      this.props.setDnsOptions({
        custom: addresses.length > 0 && this.props.dns.custom,
        addresses,
      }),
    );
  };

  private tunnelProtocolItems = (
    hasWireguardKey: boolean,
  ): Array<ISelectorItem<OptionalTunnelProtocol>> => {
    return [
      {
        label: messages.pgettext('advanced-settings-view', 'Automatic'),
        value: undefined,
      },
      {
        label: messages.pgettext('advanced-settings-view', 'OpenVPN'),
        value: 'openvpn',
      },
      {
        label: hasWireguardKey
          ? messages.pgettext('advanced-settings-view', 'WireGuard')
          : sprintf('%(label)s (%(error)s)', {
              label: messages.pgettext('advanced-settings-view', 'WireGuard'),
              error: messages.pgettext('advanced-settings-view-wireguard', 'missing key'),
            }),
        value: 'wireguard',
        disabled: !hasWireguardKey,
      },
    ];
  };

  private renderCustomDnsConfirmationDialog = () => {
    return (
      <ModalAlert
        type={ModalAlertType.info}
        buttons={[
          <AppButton.RedButton key="confirm" onClick={this.confirmPublicDnsAddress}>
            {messages.pgettext('advanced-settings-view', 'Add anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="back" onClick={this.hideCustomDnsConfirmationDialog}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ]}
        close={this.hideCustomDnsConfirmationDialog}
        message={messages.pgettext(
          'advanced-settings-view',
          'The DNS server you are trying to add might not work because it is public. Currently we only support local DNS servers.',
        )}></ModalAlert>
    );
  };

  private renderConfirmBlockWhenDisconnectedAlert = () => {
    return (
      <ModalAlert
        type={ModalAlertType.info}
        buttons={[
          <AppButton.RedButton key="confirm" onClick={this.confirmEnableBlockWhenDisconnected}>
            {messages.pgettext('advanced-settings-view', 'Enable anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="back" onClick={this.hideConfirmBlockWhenDisconnectedAlert}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ]}
        close={this.hideConfirmBlockWhenDisconnectedAlert}>
        <ModalMessage>
          {messages.pgettext(
            'advanced-settings-view',
            'Attention: enabling this will always require a Mullvad VPN connection in order to reach the internet.',
          )}
        </ModalMessage>
        <ModalMessage>
          {messages.pgettext(
            'advanced-settings-view',
            'The app’s built-in kill switch is always on. This setting will additionally block the internet if clicking Disconnect or Quit.',
          )}
        </ModalMessage>
      </ModalAlert>
    );
  };

  private setBlockWhenDisconnected = (newValue: boolean) => {
    if (newValue) {
      this.props.setBlockWhenDisconnected(true);
      this.setState({ showConfirmBlockWhenDisconnectedAlert: true });
    } else {
      this.props.setBlockWhenDisconnected(false);
    }
  };

  private hideConfirmBlockWhenDisconnectedAlert = () => {
    this.props.setBlockWhenDisconnected(false);
    this.setState({ showConfirmBlockWhenDisconnectedAlert: false });
  };

  private confirmEnableBlockWhenDisconnected = () => {
    this.props.setBlockWhenDisconnected(true);
    this.setState({ showConfirmBlockWhenDisconnectedAlert: false });
  };

  private onSelectTunnelProtocol = (protocol?: TunnelProtocol) => {
    this.props.setTunnelProtocol(protocol);
  };

  private onSelectOpenvpnProtocol = (protocol?: RelayProtocol) => {
    this.props.setOpenVpnRelayProtocolAndPort(protocol);
  };

  private onSelectOpenVpnPort = (port?: number) => {
    this.props.setOpenVpnRelayProtocolAndPort(this.props.openvpn.protocol, port);
  };

  private onSelectWireguardPort = (port?: number) => {
    this.props.setWireguardRelayPort(port);
  };

  private onSelectBridgeState = (bridgeState: BridgeState) => {
    this.props.setBridgeState(bridgeState);
  };

  private onMssfixSubmit = (value: string) => {
    const parsedValue = value === '' ? undefined : parseInt(value, 10);
    if (AdvancedSettings.mssfixIsValid(value)) {
      this.props.setOpenVpnMssfix(parsedValue);
    }
  };

  private static removeNonNumericCharacters(value: string) {
    return value.replace(/[^0-9]/g, '');
  }

  private static mssfixIsValid(mssfix: string): boolean {
    const parsedMssFix = mssfix ? parseInt(mssfix) : undefined;
    return (
      parsedMssFix === undefined ||
      (parsedMssFix >= MIN_MSSFIX_VALUE && parsedMssFix <= MAX_MSSFIX_VALUE)
    );
  }

  private onWireguardMtuSubmit = (value: string) => {
    const parsedValue = value === '' ? undefined : parseInt(value, 10);
    if (AdvancedSettings.wireguarMtuIsValid(value)) {
      this.props.setWireguardMtu(parsedValue);
    }
  };

  private static wireguarMtuIsValid(mtu: string): boolean {
    const parsedMtu = mtu ? parseInt(mtu) : undefined;
    return (
      parsedMtu === undefined ||
      (parsedMtu >= MIN_WIREGUARD_MTU_VALUE && parsedMtu <= MAX_WIREGUARD_MTU_VALUE)
    );
  }
}

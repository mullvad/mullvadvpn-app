import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { sprintf } from 'sprintf-js';
import { BridgeState, RelayProtocol, TunnelProtocol } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { WgKeyState } from '../redux/settings/reducers';
import styles from './AdvancedSettingsStyles';
import * as Cell from './Cell';
import { Container, Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import Selector, { ISelectorItem } from './Selector';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

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
  setBridgeState: (value: BridgeState) => void;
  setEnableIpv6: (value: boolean) => void;
  setBlockWhenDisconnected: (value: boolean) => void;
  setTunnelProtocol: (value: OptionalTunnelProtocol) => void;
  setOpenVpnMssfix: (value: number | undefined) => void;
  setWireguardMtu: (value: number | undefined) => void;
  setOpenVpnRelayProtocolAndPort: (protocol?: RelayProtocol, port?: number) => void;
  setWireguardRelayPort: (port?: number) => void;
  onViewWireguardKeys: () => void;
  onClose: () => void;
}

export default class AdvancedSettings extends Component<IProps> {
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
      <Layout>
        <Container>
          <View style={styles.advanced_settings}>
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

              <View style={styles.advanced_settings__container}>
                <NavigationScrollbars style={styles.advanced_settings__scrollview}>
                  <SettingsHeader>
                    <HeaderTitle>
                      {messages.pgettext('advanced-settings-view', 'Advanced')}
                    </HeaderTitle>
                  </SettingsHeader>

                  <Cell.Container>
                    <Cell.Label>
                      {messages.pgettext('advanced-settings-view', 'Enable IPv6')}
                    </Cell.Label>
                    <Cell.Switch isOn={this.props.enableIpv6} onChange={this.props.setEnableIpv6} />
                  </Cell.Container>
                  <Cell.Footer>
                    <Cell.FooterText>
                      {messages.pgettext(
                        'advanced-settings-view',
                        'Enable IPv6 communication through the tunnel.',
                      )}
                    </Cell.FooterText>
                  </Cell.Footer>

                  <Cell.Container>
                    <Cell.Label textStyle={styles.advanced_settings__block_when_disconnected_label}>
                      {messages.pgettext('advanced-settings-view', 'Block when disconnected')}
                    </Cell.Label>
                    <Cell.Switch
                      isOn={this.props.blockWhenDisconnected}
                      onChange={this.props.setBlockWhenDisconnected}
                    />
                  </Cell.Container>
                  <Cell.Footer>
                    <Cell.FooterText>
                      {messages.pgettext(
                        'advanced-settings-view',
                        "Unless connected to Mullvad, this setting will completely block your internet, even when you've disconnected or quit the app.",
                      )}
                    </Cell.FooterText>

                    {this.props.blockWhenDisconnected && (
                      <Cell.FooterBoldText
                        style={styles.advanced_settings__cell_footer_internet_warning_label}>
                        {messages.pgettext(
                          'advanced-settings-view',
                          "Warning: Your internet won't work without a VPN connection, even when you've quit the app.",
                        )}
                      </Cell.FooterBoldText>
                    )}
                  </Cell.Footer>

                  <View
                    style={[
                      styles.advanced_settings__content,
                      styles.advanced_settings__tunnel_protocol,
                    ]}>
                    <Selector
                      title={messages.pgettext('advanced-settings-view', 'Tunnel protocol')}
                      values={this.tunnelProtocolItems(hasWireguardKey)}
                      value={this.props.tunnelProtocol}
                      onSelect={this.onSelectTunnelProtocol}
                      style={styles.advanced_settings__tunnel_protocol_selector}
                    />
                    {!hasWireguardKey && (
                      <Text style={styles.advanced_settings__wg_no_key}>
                        {messages.pgettext(
                          'advanced-settings-view',
                          'To enable WireGuard, generate a key under the "WireGuard key" setting below.',
                        )}
                      </Text>
                    )}
                  </View>

                  {this.props.tunnelProtocol !== 'wireguard' ? (
                    <View style={styles.advanced_settings__content}>
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
                    </View>
                  ) : undefined}

                  {this.props.tunnelProtocol === 'wireguard' ? (
                    <View style={styles.advanced_settings__content}>
                      <Selector
                        // TRANSLATORS: The title for the shadowsocks bridge selector section.
                        title={messages.pgettext('advanced-settings-view', 'WireGuard port')}
                        values={this.wireguardPortItems}
                        value={this.props.wireguard.port}
                        onSelect={this.onSelectWireguardPort}
                      />
                    </View>
                  ) : undefined}

                  <Selector
                    title={
                      // TRANSLATORS: The title for the shadowsocks bridge selector section.
                      messages.pgettext('advanced-settings-view', 'Bridge mode')
                    }
                    values={this.bridgeStateItems}
                    value={this.props.bridgeState}
                    onSelect={this.onSelectBridgeState}
                  />

                  <Cell.Container>
                    <Cell.Label>
                      {messages.pgettext('advanced-settings-view', 'OpenVPN Mssfix')}
                    </Cell.Label>
                    <Cell.InputFrame style={styles.advanced_settings__input_frame}>
                      <Cell.AutoSizingTextInputContainer>
                        <Cell.Input
                          value={this.props.mssfix ? this.props.mssfix.toString() : ''}
                          keyboardType={'numeric'}
                          maxLength={4}
                          placeholder={messages.pgettext('advanced-settings-view', 'Default')}
                          onSubmit={this.onMssfixSubmit}
                          validateValue={AdvancedSettings.mssfixIsValid}
                          submitOnBlur={true}
                          modifyValue={AdvancedSettings.removeNonNumericCharacters}
                        />
                      </Cell.AutoSizingTextInputContainer>
                    </Cell.InputFrame>
                  </Cell.Container>
                  <Cell.Footer>
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
                  </Cell.Footer>

                  <Cell.Container>
                    <Cell.Label>
                      {messages.pgettext('advanced-settings-view', 'WireGuard MTU')}
                    </Cell.Label>
                    <Cell.InputFrame style={styles.advanced_settings__input_frame}>
                      <Cell.AutoSizingTextInputContainer>
                        <Cell.Input
                          value={this.props.wireguardMtu ? this.props.wireguardMtu.toString() : ''}
                          keyboardType={'numeric'}
                          maxLength={4}
                          placeholder={messages.pgettext('advanced-settings-view', 'Default')}
                          onSubmit={this.onWireguardMtuSubmit}
                          validateValue={AdvancedSettings.wireguarMtuIsValid}
                          submitOnBlur={true}
                          modifyValue={AdvancedSettings.removeNonNumericCharacters}
                        />
                      </Cell.AutoSizingTextInputContainer>
                    </Cell.InputFrame>
                  </Cell.Container>
                  <Cell.Footer>
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
                  </Cell.Footer>

                  <View style={styles.advanced_settings__wgkeys_cell}>
                    <Cell.CellButton onPress={this.props.onViewWireguardKeys}>
                      <Cell.Label>
                        {messages.pgettext('advanced-settings-view', 'WireGuard key')}
                      </Cell.Label>
                      <Cell.Icon height={12} width={7} source="icon-chevron" />
                    </Cell.CellButton>
                  </View>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

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

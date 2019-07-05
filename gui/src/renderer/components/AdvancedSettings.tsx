import * as React from 'react';
import { Component, View } from 'reactxp';
import { sprintf } from 'sprintf-js';
import { colors } from '../../config.json';
import { BridgeState, RelayProtocol } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import styles from './AdvancedSettingsStyles';
import * as Cell from './Cell';
import { Container, Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const MIN_MSSFIX_VALUE = 1000;
const MAX_MSSFIX_VALUE = 1450;
const UDP_PORTS = [1194, 1195, 1196, 1197, 1300, 1301, 1302];
const TCP_PORTS = [80, 443];

type OptionalPort = number | undefined;

type OptionalRelayProtocol = RelayProtocol | undefined;

function mapPortToSelectorItem(value: number): ISelectorItem<number> {
  return { label: value.toString(), value };
}

interface IProps {
  enableIpv6: boolean;
  blockWhenDisconnected: boolean;
  protocol?: RelayProtocol;
  mssfix?: number;
  port?: number;
  bridgeState: BridgeState;
  setBridgeState: (value: BridgeState) => void;
  setEnableIpv6: (value: boolean) => void;
  setBlockWhenDisconnected: (value: boolean) => void;
  setOpenVpnMssfix: (value: number | undefined) => void;
  setRelayProtocolAndPort: (protocol?: RelayProtocol, port?: number) => void;
  onClose: () => void;
}

interface IState {
  persistedMssfix?: number;
  editedMssfix?: number;
  focusOnMssfix: boolean;
}

export default class AdvancedSettings extends Component<IProps, IState> {
  private portItems: { [key in RelayProtocol]: Array<ISelectorItem<OptionalPort>> };
  private protocolItems: Array<ISelectorItem<OptionalRelayProtocol>>;
  private bridgeStateItems: Array<ISelectorItem<BridgeState>>;

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

    this.state = {
      persistedMssfix: props.mssfix,
      editedMssfix: props.mssfix,
      focusOnMssfix: false,
    };
  }

  public componentDidUpdate(_oldProps: IProps, _oldState: IState) {
    if (this.props.mssfix !== this.state.persistedMssfix) {
      this.setState((state, props) => ({
        ...state,
        persistedMssfix: props.mssfix,
        editedMssfix: state.focusOnMssfix ? state.editedMssfix : props.mssfix,
      }));
    }
  }

  public render() {
    const mssfixStyle = this.mssfixIsValid()
      ? styles.advanced_settings__mssfix_valid_value
      : styles.advanced_settings__mssfix_invalid_value;
    const mssfixValue = this.state.editedMssfix;

    return (
      <Layout>
        <Container>
          <View style={styles.advanced_settings}>
            <NavigationContainer>
              <NavigationBar>
                <BackBarItem action={this.props.onClose}>
                  {// TRANSLATORS: Back button in navigation bar
                  messages.pgettext('advanced-settings-nav', 'Settings')}
                </BackBarItem>
                <TitleBarItem>
                  {// TRANSLATORS: Title label in navigation bar
                  messages.pgettext('advanced-settings-nav', 'Advanced')}
                </TitleBarItem>
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
                    {messages.pgettext(
                      'advanced-settings-view',
                      'Enable IPv6 communication through the tunnel.',
                    )}
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
                    {messages.pgettext(
                      'advanced-settings-view',
                      "Unless connected to Mullvad, this setting will completely block your internet, even when you have quit the app.",
                    )}
                    {this.props.blockWhenDisconnected ? (
                      messages.pgettext(
                        'advanced-settings-view',
                        "\n\nWarning: Your internet won't work without a VPN connection, even when you've quit the app. Unless connected to Mullvad, this setting will completely block your internet, even when you have quit the app.",
                      )
                    ) : (
                      undefined
                    )}
                  </Cell.Footer>

                  <View style={styles.advanced_settings__content}>
                    <Selector
                      title={messages.pgettext('advanced-settings-view', 'Network protocols')}
                      values={this.protocolItems}
                      value={this.props.protocol}
                      onSelect={this.onSelectProtocol}
                    />

                    {this.props.protocol ? (
                      <Selector
                        title={sprintf(
                          // TRANSLATORS: The title for the port selector section.
                          // TRANSLATORS: Available placeholders:
                          // TRANSLATORS: %(portType)s - a selected protocol (either TCP or UDP)
                          messages.pgettext('advanced-settings-view', '%(portType)s port'),
                          {
                            portType: this.props.protocol.toUpperCase(),
                          },
                        )}
                        values={this.portItems[this.props.protocol]}
                        value={this.props.port}
                        onSelect={this.onSelectPort}
                      />
                    ) : (
                      undefined
                    )}
                  </View>

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
                    <Cell.Label>{messages.pgettext('advanced-settings-view', 'Mssfix')}</Cell.Label>
                    <Cell.InputFrame style={styles.advanced_settings__mssfix_frame}>
                      <Cell.AutoSizingTextInputContainer>
                        <Cell.Input
                          keyboardType={'numeric'}
                          maxLength={4}
                          placeholder={messages.pgettext('advanced-settings-view', 'Default')}
                          value={mssfixValue ? mssfixValue.toString() : ''}
                          style={[styles.advanced_settings__mssfix_input, mssfixStyle]}
                          onChangeText={this.onMssfixChange}
                          onFocus={this.onMssfixFocus}
                          onBlur={this.onMssfixBlur}
                        />
                      </Cell.AutoSizingTextInputContainer>
                    </Cell.InputFrame>
                  </Cell.Container>
                  <Cell.Footer>
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
                  </Cell.Footer>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  private onSelectProtocol = (protocol?: RelayProtocol) => {
    this.props.setRelayProtocolAndPort(protocol);
  };

  private onSelectPort = (port?: number) => {
    this.props.setRelayProtocolAndPort(this.props.protocol, port);
  };

  private onSelectBridgeState = (bridgeState: BridgeState) => {
    this.props.setBridgeState(bridgeState);
  };

  private onMssfixChange = (mssfixString: string) => {
    const mssfix = mssfixString.replace(/[^0-9]/g, '');

    if (mssfix === '') {
      this.setState({ editedMssfix: undefined });
    } else {
      this.setState({ editedMssfix: parseInt(mssfix, 10) });
    }
  };

  private onMssfixFocus = () => {
    this.setState({ focusOnMssfix: true });
  };

  private onMssfixBlur = () => {
    this.setState({ focusOnMssfix: false });

    if (this.mssfixIsValid()) {
      this.props.setOpenVpnMssfix(this.state.editedMssfix);
      this.setState((state, _props) => ({ persistedMssfix: state.editedMssfix }));
    }
  };

  private mssfixIsValid(): boolean {
    const mssfix = this.state.editedMssfix;

    return mssfix === undefined || (mssfix >= MIN_MSSFIX_VALUE && mssfix <= MAX_MSSFIX_VALUE);
  }
}

interface ISelectorItem<T> {
  label: string;
  value: T;
}

interface ISelectorProps<T> {
  title: string;
  values: Array<ISelectorItem<T>>;
  value: T;
  onSelect: (value: T) => void;
}

class Selector<T> extends Component<ISelectorProps<T>> {
  public render() {
    return (
      <Cell.Section style={styles.advanced_settings__selector_section}>
        <Cell.SectionTitle>{this.props.title}</Cell.SectionTitle>
        {this.props.values.map((item, i) => (
          <SelectorCell
            key={i}
            value={item.value}
            selected={item.value === this.props.value}
            onSelect={this.props.onSelect}>
            {item.label}
          </SelectorCell>
        ))}
      </Cell.Section>
    );
  }
}

interface ISelectorCell<T> {
  value: T;
  selected: boolean;
  onSelect: (value: T) => void;
  children?: React.ReactText;
}

class SelectorCell<T> extends Component<ISelectorCell<T>> {
  public render() {
    return (
      <Cell.CellButton
        style={this.props.selected ? styles.advanced_settings__cell_selected_hover : undefined}
        cellHoverStyle={
          this.props.selected ? styles.advanced_settings__cell_selected_hover : undefined
        }
        onPress={this.onPress}>
        <Cell.Icon
          style={this.props.selected ? undefined : styles.advanced_settings__cell_icon_invisible}
          source="icon-tick"
          width={24}
          height={24}
          tintColor={colors.white}
        />
        <Cell.Label>{this.props.children}</Cell.Label>
      </Cell.CellButton>
    );
  }

  private onPress = () => {
    if (!this.props.selected) {
      this.props.onSelect(this.props.value);
    }
  };
}

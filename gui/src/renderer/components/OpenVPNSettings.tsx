import * as React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { BridgeState, RelayProtocol } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import * as AppButton from './AppButton';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType } from './Modal';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import Selector, { ISelectorItem } from './cell/Selector';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import { formatMarkdown } from '../markdown-formatter';

const MIN_MSSFIX_VALUE = 1000;
const MAX_MSSFIX_VALUE = 1450;
const UDP_PORTS = [1194, 1195, 1196, 1197, 1300, 1301, 1302];
const TCP_PORTS = [80, 443];

type OptionalPort = number | undefined;

type OptionalRelayProtocol = RelayProtocol | undefined;

export enum BridgeModeAvailability {
  available,
  blockedDueToTunnelProtocol,
  blockedDueToTransportProtocol,
}

function mapPortToSelectorItem(value: number): ISelectorItem<number> {
  return { label: value.toString(), value };
}

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

interface IProps {
  bridgeModeAvailablity: BridgeModeAvailability;
  openvpn: {
    protocol?: RelayProtocol;
    port?: number;
  };
  mssfix?: number;
  bridgeState: BridgeState;
  setOpenVpnMssfix: (value: number | undefined) => void;
  setOpenVpnRelayProtocolAndPort: (protocol?: RelayProtocol, port?: number) => void;
  setBridgeState: (value: BridgeState) => void;
  onClose: () => void;
}

interface IState {
  showBridgeStateConfirmationDialog: boolean;
}

export default class OpenVpnSettings extends React.Component<IProps, IState> {
  public state = { showBridgeStateConfirmationDialog: false };

  private portItems: { [key in RelayProtocol]: Array<ISelectorItem<OptionalPort>> };

  constructor(props: IProps) {
    super(props);

    const automaticPort: ISelectorItem<OptionalPort> = {
      label: messages.gettext('Automatic'),
      value: undefined,
    };

    this.portItems = {
      udp: [automaticPort].concat(UDP_PORTS.map(mapPortToSelectorItem)),
      tcp: [automaticPort].concat(TCP_PORTS.map(mapPortToSelectorItem)),
    };
  }

  public render() {
    return (
      <Layout>
        <SettingsContainer>
          <NavigationContainer>
            <NavigationBar>
              <NavigationItems>
                <BackBarItem action={this.props.onClose} />
                <TitleBarItem>
                  {
                    // TRANSLATORS: Title label in navigation bar
                    messages.pgettext('openvpn-settings-nav', 'OpenVPN settings')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('openvpn-settings-view', 'OpenVPN settings')}
                </HeaderTitle>
              </SettingsHeader>

              <StyledSelectorContainer>
                <AriaInputGroup>
                  <Selector
                    title={messages.pgettext('openvpn-settings-view', 'Transport protocol')}
                    values={this.protocolItems(this.props.bridgeState !== 'on')}
                    value={this.props.openvpn.protocol}
                    onSelect={this.onSelectOpenvpnProtocol}
                    hasFooter={this.props.bridgeState === 'on'}
                  />
                  {this.props.bridgeState === 'on' && (
                    <Cell.Footer>
                      <AriaDescription>
                        <Cell.FooterText>
                          {formatMarkdown(
                            // TRANSLATORS: This is used to instruct users how to make UDP mode
                            // TRANSLATORS: available.
                            messages.pgettext(
                              'openvpn-settings-view',
                              'To activate UDP, change **Bridge mode** to **Automatic** or **Off**.',
                            ),
                          )}
                        </Cell.FooterText>
                      </AriaDescription>
                    </Cell.Footer>
                  )}
                </AriaInputGroup>
              </StyledSelectorContainer>

              <StyledSelectorContainer>
                <AriaInputGroup>
                  {this.props.openvpn.protocol ? (
                    <Selector
                      title={sprintf(
                        // TRANSLATORS: The title for the port selector section.
                        // TRANSLATORS: Available placeholders:
                        // TRANSLATORS: %(portType)s - a selected protocol (either TCP or UDP)
                        messages.pgettext('openvpn-settings-view', '%(portType)s port'),
                        {
                          portType: this.props.openvpn.protocol.toUpperCase(),
                        },
                      )}
                      values={this.portItems[this.props.openvpn.protocol]}
                      value={this.props.openvpn.port}
                      onSelect={this.onSelectOpenVpnPort}
                    />
                  ) : undefined}
                </AriaInputGroup>
              </StyledSelectorContainer>

              <AriaInputGroup>
                <StyledSelectorContainer>
                  <Selector
                    title={
                      // TRANSLATORS: The title for the shadowsocks bridge selector section.
                      messages.pgettext('openvpn-settings-view', 'Bridge mode')
                    }
                    values={this.bridgeStateItems(
                      this.props.bridgeModeAvailablity === BridgeModeAvailability.available,
                    )}
                    value={this.props.bridgeState}
                    onSelect={this.onSelectBridgeState}
                    hasFooter
                  />
                </StyledSelectorContainer>
                <Cell.Footer>
                  <AriaDescription>
                    <Cell.FooterText>{this.bridgeModeFooterText()}</Cell.FooterText>
                  </AriaDescription>
                </Cell.Footer>
              </AriaInputGroup>

              <AriaInputGroup>
                <Cell.Container>
                  <AriaLabel>
                    <Cell.InputLabel>
                      {messages.pgettext('openvpn-settings-view', 'Mssfix')}
                    </Cell.InputLabel>
                  </AriaLabel>
                  <AriaInput>
                    <Cell.AutoSizingTextInput
                      value={this.props.mssfix ? this.props.mssfix.toString() : ''}
                      inputMode={'numeric'}
                      maxLength={4}
                      placeholder={messages.gettext('Default')}
                      onSubmitValue={this.onMssfixSubmit}
                      validateValue={OpenVpnSettings.mssfixIsValid}
                      submitOnBlur={true}
                      modifyValue={OpenVpnSettings.removeNonNumericCharacters}
                    />
                  </AriaInput>
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
                          'openvpn-settings-view',
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
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>

        {this.renderBridgeStateConfirmation()}
      </Layout>
    );
  }

  private bridgeStateItems(onAvailable: boolean): Array<ISelectorItem<BridgeState>> {
    return [
      {
        label: messages.gettext('Automatic'),
        value: 'auto',
      },
      {
        label: messages.gettext('On'),
        value: 'on',
        disabled: !onAvailable,
      },
      {
        label: messages.gettext('Off'),
        value: 'off',
      },
    ];
  }

  private protocolItems(udpAvailable: boolean): Array<ISelectorItem<OptionalRelayProtocol>> {
    return [
      {
        label: messages.gettext('Automatic'),
        value: undefined,
      },
      {
        label: messages.gettext('TCP'),
        value: 'tcp',
      },
      {
        label: messages.gettext('UDP'),
        value: 'udp',
        disabled: !udpAvailable,
      },
    ];
  }

  private onSelectOpenvpnProtocol = (protocol?: RelayProtocol) => {
    this.props.setOpenVpnRelayProtocolAndPort(protocol);
  };

  private onSelectOpenVpnPort = (port?: number) => {
    this.props.setOpenVpnRelayProtocolAndPort(this.props.openvpn.protocol, port);
  };

  private onMssfixSubmit = (value: string) => {
    const parsedValue = value === '' ? undefined : parseInt(value, 10);
    if (OpenVpnSettings.mssfixIsValid(value)) {
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

  private bridgeModeFooterText() {
    switch (this.props.bridgeModeAvailablity) {
      case BridgeModeAvailability.blockedDueToTunnelProtocol:
        return formatMarkdown(
          // TRANSLATORS: This is used to instruct users how to make the bridge mode setting
          // TRANSLATORS: available.
          messages.pgettext(
            'openvpn-settings-view',
            'To activate Bridge mode, go back and change **Tunnel protocol** to **OpenVPN**.',
          ),
        );
      case BridgeModeAvailability.blockedDueToTransportProtocol:
        return formatMarkdown(
          // TRANSLATORS: This is used to instruct users how to make the bridge mode setting
          // TRANSLATORS: available.
          messages.pgettext(
            'openvpn-settings-view',
            'To activate Bridge mode, change **Transport protocol** to **Automatic** or **TCP**.',
          ),
        );
      case BridgeModeAvailability.available:
        // This line is here to prevent prettier from moving up the next line.
        // TRANSLATORS: This is used as a description for the bridge mode
        // TRANSLATORS: setting.
        return messages.pgettext(
          'openvpn-settings-view',
          'Helps circumvent censorship, by routing your traffic through a bridge server before reaching an OpenVPN server. Obfuscation is added to make fingerprinting harder.',
        );
    }
  }

  private renderBridgeStateConfirmation = () => {
    return (
      <ModalAlert
        isOpen={this.state.showBridgeStateConfirmationDialog}
        type={ModalAlertType.info}
        message={messages.gettext('This setting increases latency. Use only if needed.')}
        buttons={[
          <AppButton.RedButton key="confirm" onClick={this.confirmBridgeState}>
            {messages.gettext('Enable anyway')}
          </AppButton.RedButton>,
          <AppButton.BlueButton key="back" onClick={this.hideBridgeStateConfirmationDialog}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ]}
        close={this.hideBridgeStateConfirmationDialog}></ModalAlert>
    );
  };

  private onSelectBridgeState = (newValue: BridgeState) => {
    if (newValue === 'on') {
      this.setState({ showBridgeStateConfirmationDialog: true });
    } else {
      this.props.setBridgeState(newValue);
    }
  };

  private hideBridgeStateConfirmationDialog = () => {
    this.setState({ showBridgeStateConfirmationDialog: false });
  };

  private confirmBridgeState = () => {
    this.setState({ showBridgeStateConfirmationDialog: false });
    this.props.setBridgeState('on');
  };
}

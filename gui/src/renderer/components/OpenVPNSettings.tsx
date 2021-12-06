import * as React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { BridgeState, RelayProtocol } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import { Layout, SettingsContainer } from './Layout';
import { ModalContainer } from './Modal';
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

function mapPortToSelectorItem(value: number): ISelectorItem<number> {
  return { label: value.toString(), value };
}

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

export const StyledInputFrame = styled(Cell.InputFrame)({
  flex: 0,
});

export const StyledSelectorForFooter = (styled(Selector)({
  marginBottom: 0,
}) as unknown) as new <T>() => Selector<T>;

interface IProps {
  tunnelProtocolIsOpenVpn: boolean;
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

export default class OpenVpnSettings extends React.Component<IProps> {
  private portItems: { [key in RelayProtocol]: Array<ISelectorItem<OptionalPort>> };
  private protocolItems: Array<ISelectorItem<OptionalRelayProtocol>>;

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

    this.protocolItems = [
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
      },
    ];
  }

  public render() {
    return (
      <ModalContainer>
        <Layout>
          <SettingsContainer>
            <NavigationContainer>
              <NavigationBar>
                <NavigationItems>
                  <BackBarItem action={this.props.onClose}>
                    {
                      // TRANSLATORS: Back button in navigation bar
                      messages.pgettext('navigation-bar', 'Advanced')
                    }
                  </BackBarItem>
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

                <AriaInputGroup>
                  <StyledSelectorContainer>
                    <Selector
                      title={messages.pgettext('openvpn-settings-view', 'Transport protocol')}
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
                  </StyledSelectorContainer>
                </AriaInputGroup>

                <AriaInputGroup>
                  <StyledSelectorContainer>
                    <StyledSelectorForFooter
                      title={
                        // TRANSLATORS: The title for the shadowsocks bridge selector section.
                        messages.pgettext('openvpn-settings-view', 'Bridge mode')
                      }
                      values={this.bridgeStateItems(this.props.tunnelProtocolIsOpenVpn)}
                      value={this.props.bridgeState}
                      onSelect={this.onSelectBridgeState}
                    />
                  </StyledSelectorContainer>
                  <Cell.Footer>
                    <AriaDescription>
                      <Cell.FooterText>
                        {this.props.tunnelProtocolIsOpenVpn
                          ? // This line is here to prevent prettier from moving up the next line.
                            // TRANSLATORS: This is used as a description for the bridge mode
                            // TRANSLATORS: setting.
                            messages.pgettext(
                              'openvpn-settings-view',
                              'Helps circumvent censorship, by routing your traffic through a bridge server before reaching an OpenVPN server. Obfuscation is added to make fingerprinting harder.',
                            )
                          : // This line is here to prevent prettier from moving up the next line.
                            // TRANSLATORS: This is used to instruct users how to make the bridge
                            // TRANSLATORS: mode setting available.
                            formatMarkdown(
                              messages.pgettext(
                                'wireguard-settings-view',
                                'To activate Bridge mode, go back and change **Tunnel protocol** to **OpenVPN**.',
                              ),
                            )}
                      </Cell.FooterText>
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
                    <StyledInputFrame>
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
        </Layout>
      </ModalContainer>
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

  private onSelectOpenvpnProtocol = (protocol?: RelayProtocol) => {
    this.props.setOpenVpnRelayProtocolAndPort(protocol);
  };

  private onSelectOpenVpnPort = (port?: number) => {
    this.props.setOpenVpnRelayProtocolAndPort(this.props.openvpn.protocol, port);
  };

  private onSelectBridgeState = (bridgeState: BridgeState) => {
    this.props.setBridgeState(bridgeState);
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
}

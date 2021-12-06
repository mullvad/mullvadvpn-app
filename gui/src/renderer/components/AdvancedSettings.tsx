import * as React from 'react';
import { sprintf } from 'sprintf-js';
import { TunnelProtocol } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { WgKeyState } from '../redux/settings/reducers';
import {
  StyledNavigationScrollbars,
  StyledNoWireguardKeyError,
  StyledNoWireguardKeyErrorContainer,
  StyledSelectorForFooter,
  StyledTunnelProtocolContainer,
} from './AdvancedSettingsStyles';
import * as AppButton from './AppButton';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import CustomDnsSettings from './CustomDnsSettings';
import { Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType, ModalContainer, ModalMessage } from './Modal';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  TitleBarItem,
} from './NavigationBar';
import { ISelectorItem } from './cell/Selector';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import Switch from './Switch';

type OptionalTunnelProtocol = TunnelProtocol | undefined;

interface IProps {
  enableIpv6: boolean;
  blockWhenDisconnected: boolean;
  tunnelProtocol?: TunnelProtocol;
  wireguardKeyState: WgKeyState;
  setEnableIpv6: (value: boolean) => void;
  setBlockWhenDisconnected: (value: boolean) => void;
  setTunnelProtocol: (value: OptionalTunnelProtocol) => void;
  onViewWireguardSettings: () => void;
  onViewOpenVpnSettings: () => void;
  onViewSplitTunneling: () => void;
  onClose: () => void;
}

interface IState {
  showConfirmBlockWhenDisconnectedAlert: boolean;
}

export default class AdvancedSettings extends React.Component<IProps, IState> {
  public state = {
    showConfirmBlockWhenDisconnectedAlert: false,
  };

  private blockWhenDisconnectedRef = React.createRef<Switch>();

  public render() {
    const hasWireguardKey = this.props.wireguardKeyState.type === 'key-set';

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
                        ref={this.blockWhenDisconnectedRef}
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

                {(window.env.platform === 'linux' || window.env.platform === 'win32') && (
                  <Cell.CellButtonGroup>
                    <Cell.CellButton onClick={this.props.onViewSplitTunneling}>
                      <Cell.Label>
                        {messages.pgettext('advanced-settings-view', 'Split tunneling')}
                      </Cell.Label>
                      <Cell.Icon height={12} width={7} source="icon-chevron" />
                    </Cell.CellButton>
                  </Cell.CellButtonGroup>
                )}

                <AriaInputGroup>
                  <StyledTunnelProtocolContainer>
                    <StyledSelectorForFooter
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

                <Cell.CellButtonGroup>
                  <Cell.CellButton
                    onClick={this.props.onViewWireguardSettings}
                    disabled={this.props.tunnelProtocol === 'openvpn'}>
                    <Cell.Label>
                      {messages.pgettext('advanced-settings-view', 'WireGuard settings')}
                    </Cell.Label>
                    <Cell.Icon height={12} width={7} source="icon-chevron" />
                  </Cell.CellButton>

                  <Cell.CellButton
                    onClick={this.props.onViewOpenVpnSettings}
                    disabled={this.props.tunnelProtocol === 'wireguard'}>
                    <Cell.Label>
                      {messages.pgettext('advanced-settings-view', 'OpenVPN settings')}
                    </Cell.Label>
                    <Cell.Icon height={12} width={7} source="icon-chevron" />
                  </Cell.CellButton>
                </Cell.CellButtonGroup>

                <CustomDnsSettings />
              </StyledNavigationScrollbars>
            </NavigationContainer>
          </SettingsContainer>
        </Layout>

        {this.state.showConfirmBlockWhenDisconnectedAlert &&
          this.renderConfirmBlockWhenDisconnectedAlert()}
      </ModalContainer>
    );
  }

  private tunnelProtocolItems = (
    hasWireguardKey: boolean,
  ): Array<ISelectorItem<OptionalTunnelProtocol>> => {
    return [
      {
        label: messages.gettext('Automatic'),
        value: undefined,
      },
      {
        label: hasWireguardKey
          ? messages.pgettext('advanced-settings-view', 'WireGuard')
          : sprintf('%(label)s (%(error)s)', {
              label: messages.pgettext('advanced-settings-view', 'WireGuard'),
              error: messages.pgettext('advanced-settings-view', 'missing key'),
            }),
        value: 'wireguard',
        disabled: !hasWireguardKey,
      },
      {
        label: messages.pgettext('advanced-settings-view', 'OpenVPN'),
        value: 'openvpn',
      },
    ];
  };

  private renderConfirmBlockWhenDisconnectedAlert = () => {
    return (
      <ModalAlert
        type={ModalAlertType.caution}
        buttons={[
          <AppButton.RedButton key="confirm" onClick={this.confirmEnableBlockWhenDisconnected}>
            {messages.gettext('Enable anyway')}
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
      this.setState({ showConfirmBlockWhenDisconnectedAlert: true });
    } else {
      this.props.setBlockWhenDisconnected(false);
    }
  };

  private hideConfirmBlockWhenDisconnectedAlert = () => {
    this.setState({ showConfirmBlockWhenDisconnectedAlert: false });
    this.blockWhenDisconnectedRef.current?.setOn(this.props.blockWhenDisconnected);
  };

  private confirmEnableBlockWhenDisconnected = () => {
    this.setState({ showConfirmBlockWhenDisconnectedAlert: false });
    this.props.setBlockWhenDisconnected(true);
  };

  private onSelectTunnelProtocol = (protocol?: TunnelProtocol) => {
    this.props.setTunnelProtocol(protocol);
  };
}

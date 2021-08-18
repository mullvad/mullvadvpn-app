import * as React from 'react';
import { sprintf } from 'sprintf-js';
import { colors } from '../../config.json';
import { IDnsOptions, TunnelProtocol } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { IpAddress } from '../lib/ip';
import { WgKeyState } from '../redux/settings/reducers';
import {
  StyledButtonCellGroup,
  StyledContainer,
  StyledNavigationScrollbars,
  StyledNoWireguardKeyError,
  StyledNoWireguardKeyErrorContainer,
  StyledSelectorForFooter,
  StyledTunnelProtocolContainer,
  StyledCustomDnsSwitchContainer,
  StyledCustomDnsFooter,
  StyledAddCustomDnsLabel,
  StyledAddCustomDnsButton,
  StyledBetaLabel,
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
import { ISelectorItem } from './cell/Selector';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import Accordion from './Accordion';
import { formatMarkdown } from '../markdown-formatter';

type OptionalTunnelProtocol = TunnelProtocol | undefined;

interface IProps {
  enableIpv6: boolean;
  blockWhenDisconnected: boolean;
  tunnelProtocol?: TunnelProtocol;
  wireguardKeyState: WgKeyState;
  dns: IDnsOptions;
  setEnableIpv6: (value: boolean) => void;
  setBlockWhenDisconnected: (value: boolean) => void;
  setTunnelProtocol: (value: OptionalTunnelProtocol) => void;
  setDnsOptions: (dns: IDnsOptions) => Promise<void>;
  onViewWireguardSettings: () => void;
  onViewOpenVpnSettings: () => void;
  onViewSplitTunneling: () => void;
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

                {(window.env.platform === 'linux' || window.env.platform === 'win32') && (
                  <StyledButtonCellGroup>
                    <Cell.CellButton onClick={this.props.onViewSplitTunneling}>
                      <Cell.Label>
                        {window.env.platform === 'win32' && <StyledBetaLabel />}
                        {messages.pgettext('advanced-settings-view', 'Split tunneling')}
                      </Cell.Label>
                      <Cell.Icon height={12} width={7} source="icon-chevron" />
                    </Cell.CellButton>
                  </StyledButtonCellGroup>
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

                <StyledButtonCellGroup>
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
                </StyledButtonCellGroup>

                <StyledCustomDnsSwitchContainer disabled={!this.customDnsAvailable()}>
                  <AriaInputGroup>
                    <AriaLabel>
                      <Cell.InputLabel>
                        {messages.pgettext('advanced-settings-view', 'Use custom DNS server')}
                      </Cell.InputLabel>
                    </AriaLabel>
                    <AriaInput>
                      <Cell.Switch
                        ref={this.customDnsSwitchRef}
                        isOn={this.props.dns.state === 'custom' || this.state.showAddCustomDns}
                        onChange={this.setCustomDnsEnabled}
                      />
                    </AriaInput>
                  </AriaInputGroup>
                </StyledCustomDnsSwitchContainer>
                <Accordion
                  expanded={
                    this.customDnsAvailable() &&
                    (this.props.dns.state === 'custom' || this.state.showAddCustomDns)
                  }>
                  <CellList items={this.customDnsItems()} onRemove={this.removeDnsAddress} />

                  {this.state.showAddCustomDns && (
                    <div ref={this.customDnsInputContainerRef}>
                      <Cell.RowInput
                        placeholder={messages.pgettext('advanced-settings-view', 'Enter IP')}
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

                <StyledCustomDnsFooter>
                  <Cell.FooterText>
                    {this.customDnsAvailable() ? (
                      messages.pgettext(
                        'advanced-settings-view',
                        'Enable to add at least one DNS server.',
                      )
                    ) : (
                      <CustomDnsDisabledMessage />
                    )}
                  </Cell.FooterText>
                </StyledCustomDnsFooter>
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

  private customDnsAvailable(): boolean {
    return (
      this.props.dns.state === 'custom' ||
      (!this.props.dns.defaultOptions.blockAds && !this.props.dns.defaultOptions.blockTrackers)
    );
  }

  private setCustomDnsEnabled = async (enabled: boolean) => {
    if (this.props.dns.customOptions.addresses.length > 0) {
      await this.props.setDnsOptions({
        ...this.props.dns,
        state: enabled ? 'custom' : 'default',
      });
    }

    if (enabled && this.props.dns.customOptions.addresses.length === 0) {
      this.showAddCustomDnsRow();
    }

    if (!enabled) {
      this.setState({ showAddCustomDns: false });
    }
  };

  private customDnsItems(): ICellListItem<string>[] {
    return this.props.dns.customOptions.addresses.map((address) => ({
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
      this.hideAddCustomDnsRow();
    }
  };

  private hideAddCustomDnsRow() {
    if (!this.state.publicDnsIpToConfirm) {
      this.setState({ showAddCustomDns: false });
    }
  }

  private addDnsInputChange = (_value: string) => {
    this.setState({ invalidDnsIp: false });
  };

  private hideCustomDnsConfirmationDialog = () => {
    this.setState({ publicDnsIpToConfirm: undefined });
  };

  private confirmPublicDnsAddress = () => {
    void this.addDnsAddress(this.state.publicDnsIpToConfirm!, true);
    this.hideCustomDnsConfirmationDialog();
  };

  private addDnsAddress = async (address: string, confirmed?: boolean) => {
    try {
      const ipAddress = IpAddress.fromString(address);

      if (ipAddress.isLocal() || confirmed) {
        await this.props.setDnsOptions({
          ...this.props.dns,
          state:
            this.props.dns.state === 'custom' || this.state.showAddCustomDns ? 'custom' : 'default',
          customOptions: {
            addresses: [...this.props.dns.customOptions.addresses, address],
          },
        });
        this.hideAddCustomDnsRow();
      } else {
        this.setState({ publicDnsIpToConfirm: address });
      }
    } catch (e) {
      this.setState({ invalidDnsIp: true });
    }
  };

  private removeDnsAddress = (address: string) => {
    const addresses = this.props.dns.customOptions.addresses.filter((item) => item !== address);
    void this.props.setDnsOptions({
      ...this.props.dns,
      state: addresses.length > 0 && this.props.dns.state === 'custom' ? 'custom' : 'default',
      customOptions: {
        addresses,
      },
    });
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
          'The DNS server you want to add is public and will only work with WireGuard. To ensure that it always works, set the "Tunnel protocol" (in Advanced settings) to WireGuard.',
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
}

function CustomDnsDisabledMessage() {
  const blockAdsFeatureName = messages.pgettext('preferences-view', 'Block ads');
  const blockTrackersFeatureName = messages.pgettext('preferences-view', 'Block trackers');
  const preferencesPageName = messages.pgettext('preferences-nav', 'Preferences');

  // TRANSLATORS: This is displayed when either or both of the block ads/trackers settings are
  // TRANSLATORS: turned on which makes the custom DNS setting disabled. The text enclosed in "**"
  // TRANSLATORS: will appear bold.
  // TRANSLATORS: Available placeholders:
  // TRANSLATORS: %(blockAdsFeatureName)s - The name displayed next to the "Block ads" toggle.
  // TRANSLATORS: %(blockTrackersFeatureName)s - The name displayed next to the "Block trackers" toggle.
  // TRANSLATORS: %(preferencesPageName)s - The page title showed on top in the preferences page.
  const customDnsDisabledMessage = messages.pgettext(
    'preferences-view',
    'Disable **%(blockAdsFeatureName)s** and **%(blockTrackersFeatureName)s** (under %(preferencesPageName)s) to activate this setting.',
  );

  return formatMarkdown(
    sprintf(customDnsDisabledMessage, {
      blockAdsFeatureName,
      blockTrackersFeatureName,
      preferencesPageName,
    }),
  );
}

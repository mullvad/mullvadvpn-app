import * as React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { IpVersion } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import { Layout, SettingsContainer } from './Layout';
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

const MIN_WIREGUARD_MTU_VALUE = 1280;
const MAX_WIREGUARD_MTU_VALUE = 1420;
const WIREUGARD_UDP_PORTS = [51820, 53];

type OptionalPort = number | undefined;
type OptionalIpVersion = IpVersion | undefined;

function mapPortToSelectorItem(value: number): ISelectorItem<number> {
  return { label: value.toString(), value };
}

export const StyledNavigationScrollbars = styled(NavigationScrollbars)({
  flex: 1,
});

export const StyledSelectorContainer = styled.div({
  flex: 0,
});

export const StyledSelectorForFooter = (styled(Selector)({
  marginBottom: 0,
}) as unknown) as new <T>() => Selector<T>;

export const StyledInputFrame = styled(Cell.InputFrame)({
  flex: 0,
});

interface IProps {
  wireguard: { port?: number; ipVersion?: IpVersion };
  wireguardMtu?: number;
  setWireguardMtu: (value: number | undefined) => void;
  setWireguardRelayPortAndIpVersion: (port?: number, ipVersion?: IpVersion) => void;
  onViewWireguardKeys: () => void;
  onClose: () => void;
}

export default class WireguardSettings extends React.Component<IProps> {
  private wireguardPortItems: Array<ISelectorItem<OptionalPort>>;
  private wireguardIpVersionItems: Array<ISelectorItem<OptionalIpVersion>>;

  constructor(props: IProps) {
    super(props);

    const automaticPort: ISelectorItem<OptionalPort> = {
      label: messages.gettext('Automatic'),
      value: undefined,
    };

    this.wireguardPortItems = [automaticPort].concat(
      WIREUGARD_UDP_PORTS.map(mapPortToSelectorItem),
    );

    this.wireguardIpVersionItems = [
      {
        label: messages.gettext('Automatic'),
        value: undefined,
      },
      {
        label: messages.gettext('IPv4'),
        value: 'ipv4',
      },
      {
        label: messages.gettext('IPv6'),
        value: 'ipv6',
      },
    ];
  }

  public render() {
    return (
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
                    messages.pgettext('wireguard-settings-nav', 'WireGuard settings')
                  }
                </TitleBarItem>
              </NavigationItems>
            </NavigationBar>

            <StyledNavigationScrollbars>
              <SettingsHeader>
                <HeaderTitle>
                  {messages.pgettext('wireguard-settings-view', 'WireGuard settings')}
                </HeaderTitle>
              </SettingsHeader>

              <AriaInputGroup>
                <StyledSelectorContainer>
                  <StyledSelectorForFooter
                    // TRANSLATORS: The title for the WireGuard port selector.
                    title={messages.pgettext('wireguard-settings-view', 'Port')}
                    values={this.wireguardPortItems}
                    value={this.props.wireguard.port}
                    onSelect={this.onSelectWireguardPort}
                  />
                </StyledSelectorContainer>
                <Cell.Footer>
                  <AriaDescription>
                    <Cell.FooterText>
                      {
                        // TRANSLATORS: The hint displayed below the WireGuard port selector.
                        messages.pgettext(
                          'wireguard-settings-view',
                          'The automatic setting will randomly choose from a wide range of ports.',
                        )
                      }
                    </Cell.FooterText>
                  </AriaDescription>
                </Cell.Footer>
              </AriaInputGroup>

              <AriaInputGroup>
                <StyledSelectorContainer>
                  <StyledSelectorForFooter
                    // TRANSLATORS: The title for the WireGuard IP version selector.
                    title={messages.pgettext('wireguard-settings-view', 'IP version')}
                    values={this.wireguardIpVersionItems}
                    value={this.props.wireguard.ipVersion}
                    onSelect={this.onSelectWireguardIpVersion}
                  />
                </StyledSelectorContainer>
                <Cell.Footer>
                  <AriaDescription>
                    <Cell.FooterText>
                      {
                        // TRANSLATORS: The hint displayed below the WireGuard IP version selector.
                        messages.pgettext(
                          'wireguard-settings-view',
                          'This allows access to WireGuard for devices that only support IPv6.',
                        )
                      }
                    </Cell.FooterText>
                  </AriaDescription>
                </Cell.Footer>
              </AriaInputGroup>

              <Cell.CellButtonGroup>
                <Cell.CellButton onClick={this.props.onViewWireguardKeys}>
                  <Cell.Label>
                    {messages.pgettext('wireguard-settings-view', 'WireGuard key')}
                  </Cell.Label>
                  <Cell.Icon height={12} width={7} source="icon-chevron" />
                </Cell.CellButton>
              </Cell.CellButtonGroup>

              <AriaInputGroup>
                <Cell.Container>
                  <AriaLabel>
                    <Cell.InputLabel>
                      {messages.pgettext('wireguard-settings-view', 'MTU')}
                    </Cell.InputLabel>
                  </AriaLabel>
                  <StyledInputFrame>
                    <AriaInput>
                      <Cell.AutoSizingTextInput
                        value={this.props.wireguardMtu ? this.props.wireguardMtu.toString() : ''}
                        inputMode={'numeric'}
                        maxLength={4}
                        placeholder={messages.gettext('Default')}
                        onSubmitValue={this.onWireguardMtuSubmit}
                        validateValue={WireguardSettings.wireguarMtuIsValid}
                        submitOnBlur={true}
                        modifyValue={WireguardSettings.removeNonNumericCharacters}
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
                          'wireguard-settings-view',
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
            </StyledNavigationScrollbars>
          </NavigationContainer>
        </SettingsContainer>
      </Layout>
    );
  }

  private onSelectWireguardPort = (port?: number) => {
    this.props.setWireguardRelayPortAndIpVersion(port, this.props.wireguard.ipVersion);
  };

  private onSelectWireguardIpVersion = (ipVersion?: IpVersion) => {
    this.props.setWireguardRelayPortAndIpVersion(this.props.wireguard.port, ipVersion);
  };

  private static removeNonNumericCharacters(value: string) {
    return value.replace(/[^0-9]/g, '');
  }

  private onWireguardMtuSubmit = (value: string) => {
    const parsedValue = value === '' ? undefined : parseInt(value, 10);
    if (WireguardSettings.wireguarMtuIsValid(value)) {
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

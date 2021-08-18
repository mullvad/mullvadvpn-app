import * as React from 'react';
import { sprintf } from 'sprintf-js';
import { messages } from '../../shared/gettext';
import {
  StyledButtonCellGroup,
  StyledContainer,
  StyledInputFrame,
  StyledNavigationScrollbars,
  StyledSelectorContainer,
  StyledSelectorForFooter,
} from './AdvancedSettingsStyles';
import { AriaDescription, AriaInput, AriaInputGroup, AriaLabel } from './AriaGroup';
import * as Cell from './cell';
import { Layout } from './Layout';
import {
  BackBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationItems,
  TitleBarItem,
} from './NavigationBar';
import { ISelectorItem } from './cell/Selector';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

const MIN_WIREGUARD_MTU_VALUE = 1280;
const MAX_WIREGUARD_MTU_VALUE = 1420;
const WIREUGARD_UDP_PORTS = [51820, 53];

type OptionalPort = number | undefined;

function mapPortToSelectorItem(value: number): ISelectorItem<number> {
  return { label: value.toString(), value };
}

interface IProps {
  wireguard: { port?: number };
  wireguardMtu?: number;
  setWireguardMtu: (value: number | undefined) => void;
  setWireguardRelayPort: (port?: number) => void;
  onViewWireguardKeys: () => void;
  onClose: () => void;
}

export default class WireguardSettings extends React.Component<IProps> {
  private wireguardPortItems: Array<ISelectorItem<OptionalPort>>;

  constructor(props: IProps) {
    super(props);

    const automaticPort: ISelectorItem<OptionalPort> = {
      label: messages.gettext('Automatic'),
      value: undefined,
    };

    this.wireguardPortItems = [automaticPort].concat(
      WIREUGARD_UDP_PORTS.map(mapPortToSelectorItem),
    );
  }

  public render() {
    return (
      <Layout>
        <StyledContainer>
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
                    // TRANSLATORS: The title for the shadowsocks bridge selector section.
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

              <StyledButtonCellGroup>
                <Cell.CellButton onClick={this.props.onViewWireguardKeys}>
                  <Cell.Label>
                    {messages.pgettext('wireguard-settings-view', 'WireGuard key')}
                  </Cell.Label>
                  <Cell.Icon height={12} width={7} source="icon-chevron" />
                </Cell.CellButton>
              </StyledButtonCellGroup>

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
        </StyledContainer>
      </Layout>
    );
  }

  private onSelectWireguardPort = (port?: number) => {
    this.props.setWireguardRelayPort(port);
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

// @flow

import * as React from 'react';
import ReactDOM from 'react-dom';
import { View, Component } from 'reactxp';
import { Accordion, SettingsHeader, HeaderTitle, HeaderSubTitle } from '@mullvad/components';
import { Layout, Container } from './Layout';
import {
  NavigationContainer,
  NavigationScrollbars,
  NavigationBar,
  CloseBarItem,
  TitleBarItem,
} from './NavigationBar';
import * as Cell from './Cell';
import styles from './SelectLocationStyles';

import type {
  RelaySettingsRedux,
  RelayLocationRedux,
  RelayLocationCityRedux,
  RelayLocationRelayRedux,
} from '../redux/settings/reducers';
import type { RelayLocation } from '../lib/daemon-rpc-proxy';
import { colors } from '../../config';

type Props = {
  relaySettings: RelaySettingsRedux,
  relayLocations: Array<RelayLocationRedux>,
  onClose: () => void,
  onSelect: (location: RelayLocation) => void,
};

export default class SelectLocation extends Component<Props> {
  _cellRefs = {};
  _scrollViewRef = React.createRef();
  _selectedLocation = (undefined: ?RelayLocation);

  constructor(props: Props) {
    super(props);

    if (this.props.relaySettings.normal) {
      const location = this.props.relaySettings.normal.location;

      this._selectedLocation = location;
    }
  }

  componentDidUpdate() {
    if (this.props.relaySettings.normal) {
      const location = this.props.relaySettings.normal.location;

      if (
        !this._selectedLocation ||
        this._getLocationKey(this._selectedLocation) !== this._getLocationKey(location)
      ) {
        this._setSelectedLocation(location);
      }
    } else {
      if (this._selectedLocation) {
        this._setSelectedLocation(undefined);
      }
    }
  }

  componentDidMount() {
    // restore scroll to selected cell
    const cell = (this._getSelectedCellRef() || {}).current;
    const scrollView = this._scrollViewRef.current;
    if (scrollView && cell) {
      // eslint-disable-next-line react/no-find-dom-node
      const cellDOMNode = ReactDOM.findDOMNode(cell);
      if (cellDOMNode instanceof HTMLElement) {
        scrollView.scrollToElement(cellDOMNode, 'middle');
      }
    }
  }

  render() {
    return (
      <Layout>
        <Container>
          <View style={styles.select_location}>
            <NavigationContainer>
              <NavigationBar>
                <CloseBarItem action={this.props.onClose} />
                <TitleBarItem>{'Select location'}</TitleBarItem>
              </NavigationBar>
              <View style={styles.container}>
                <NavigationScrollbars ref={this._scrollViewRef}>
                  <View style={styles.content}>
                    <SettingsHeader style={styles.subtitle_header}>
                      <HeaderTitle>Select location</HeaderTitle>
                      <HeaderSubTitle>
                        While connected, your real location is masked with a private and secure
                        location in the selected region
                      </HeaderSubTitle>
                    </SettingsHeader>

                    {this.props.relayLocations.map((relayCountry) => {
                      const location = { country: relayCountry.code };
                      const ref = this._getCellRef(location);
                      const isSelected = this._isSelected(location);

                      return (
                        <CountryRow
                          key={this._getLocationKey(location)}
                          relayCountry={relayCountry}
                          ref={ref}
                          defaultSelected={isSelected}
                          defaultCollapsed={this._isCollapsed(location)}
                          onPress={() => this._handleSelection(location)}>
                          {relayCountry.cities.map((relayCity) => {
                            const location = { city: [relayCountry.code, relayCity.code] };
                            const ref = this._getCellRef(location);
                            const isSelected = this._isSelected(location);
                            const key = this._getLocationKey(location);

                            return (
                              <CityRow
                                key={key}
                                ref={ref}
                                countryCode={relayCountry.code}
                                relayCity={relayCity}
                                defaultSelected={isSelected}
                                defaultCollapsed={this._isCollapsed(location)}
                                onPress={() => this._handleSelection(location)}>
                                {relayCity.relays.map((relay) => {
                                  const location = {
                                    hostname: [relayCountry.code, relayCity.code, relay.hostname],
                                  };
                                  const ref = this._getCellRef(location);
                                  const isSelected = this._isSelected(location);
                                  const key = this._getLocationKey(location);

                                  return (
                                    <RelayRow
                                      key={key}
                                      ref={ref}
                                      defaultSelected={isSelected}
                                      countryCode={relayCountry.code}
                                      cityCode={relayCity.code}
                                      relay={relay}
                                      onPress={() => this._handleSelection(location)}
                                    />
                                  );
                                })}
                              </CityRow>
                            );
                          })}
                        </CountryRow>
                      );
                    })}
                  </View>
                </NavigationScrollbars>
              </View>
            </NavigationContainer>
          </View>
        </Container>
      </Layout>
    );
  }

  _isCollapsed(relayLocation: RelayLocation) {
    const relaySettings = this.props.relaySettings;
    if (relaySettings.normal) {
      const selectedLocation = relaySettings.normal.location;

      if (selectedLocation.city && relayLocation.country) {
        return selectedLocation.city[0] !== relayLocation.country;
      } else if (selectedLocation.hostname && relayLocation.country) {
        return selectedLocation.hostname[0] !== relayLocation.country;
      } else if (selectedLocation.hostname && relayLocation.city) {
        return (
          selectedLocation.hostname[0] !== relayLocation.city[0] &&
          selectedLocation.hostname[1] !== relayLocation.city[1]
        );
      }
    }

    return true;
  }

  _isSelected(selectedLocation: RelayLocation) {
    const relaySettings = this.props.relaySettings;
    if (relaySettings.normal) {
      const otherLocation = relaySettings.normal.location;

      if (
        selectedLocation.country &&
        otherLocation.country &&
        selectedLocation.country === otherLocation.country
      ) {
        return true;
      }

      if (Array.isArray(selectedLocation.city) && Array.isArray(otherLocation.city)) {
        const selectedCity = selectedLocation.city;
        const otherCity = otherLocation.city;

        return (
          selectedCity.length === otherCity.length &&
          selectedCity.every((v, i) => v === otherCity[i])
        );
      }

      if (Array.isArray(selectedLocation.hostname) && Array.isArray(otherLocation.hostname)) {
        const selectedRelay = selectedLocation.hostname;
        const otherRelay = otherLocation.hostname;

        return (
          selectedRelay.length === otherRelay.length &&
          selectedRelay.every((v, i) => v === otherRelay[i])
        );
      }
    }
    return false;
  }

  _setSelectedLocation(location: ?RelayLocation) {
    const selectedCell = (this._getSelectedCellRef() || {}).current;
    const nextSelectedCell = location ? this._getCellRef(location).current : undefined;

    if (selectedCell) {
      selectedCell.setSelected(false);
    }

    if (nextSelectedCell) {
      nextSelectedCell.setSelected(true);
    }

    this._selectedLocation = location;
  }

  _handleSelection = (location: RelayLocation) => {
    this._setSelectedLocation(location);

    this.props.onSelect(location);
  };

  _getLocationKey(location: RelayLocation) {
    const components = location.city || location.country || location.hostname || [];
    return [].concat(components).join('-');
  }

  _getCellRef(location: RelayLocation) {
    const key = this._getLocationKey(location);

    if (this._cellRefs[key]) {
      return this._cellRefs[key];
    } else {
      const ref = React.createRef();
      this._cellRefs[key] = ref;
      return ref;
    }
  }

  _getSelectedCellRef() {
    if (this._selectedLocation) {
      const key = this._getLocationKey(this._selectedLocation);

      return this._cellRefs[key];
    } else {
      return undefined;
    }
  }
}

type CollapseButtonProps = {
  collapsed: boolean,
  onPress: () => void,
};

class CollapseButton extends Component<CollapseButtonProps> {
  state = {
    collapsed: false,
  };

  constructor(props: CollapseButtonProps) {
    super(props);

    this.state.collapsed = props.collapsed;
  }

  setAppearance(collapsed: boolean) {
    this.setState({ collapsed });
  }

  render() {
    return (
      <Cell.Icon
        style={styles.collapse_button}
        tintColor={colors.white80}
        tintHoverColor={colors.white}
        onPress={this.props.onPress}
        source={this.state.collapsed ? 'icon-chevron-down' : 'icon-chevron-up'}
        height={24}
        width={24}
      />
    );
  }
}

type CountryRowProps = {
  relayCountry: RelayLocationRedux,
  defaultSelected: boolean,
  defaultCollapsed: boolean,
  onPress?: () => void,
  children?: React.Element<typeof CityRow>,
};

class CountryRow extends Component<CountryRowProps> {
  _accordionRef = React.createRef();
  _collapseButtonRef = React.createRef();
  _collapsed = false;

  state = {
    selected: false,
  };

  setSelected(selected: boolean) {
    this.setState({ selected });
  }

  constructor(props: CountryRowProps) {
    super(props);

    this._collapsed = props.defaultCollapsed;
    this.state.selected = props.defaultSelected;
  }

  render() {
    const { relayCountry } = this.props;
    const hasChildren =
      relayCountry.cities.length > 1 ||
      (relayCountry.cities.length == 1 && relayCountry.cities[0].relays.length > 1);

    return (
      <View style={styles.country}>
        <Cell.CellButton
          cellHoverStyle={this.state.selected ? styles.cell_selected : null}
          style={this.state.selected ? styles.cell_selected : styles.cell}
          onPress={this.props.onPress}
          disabled={!relayCountry.hasActiveRelays}
          testName="country">
          <RelayStatusIndicator
            isActive={relayCountry.hasActiveRelays}
            isSelected={this.state.selected}
          />
          <Cell.Label>{relayCountry.name}</Cell.Label>
          {hasChildren ? (
            <CollapseButton
              ref={this._collapseButtonRef}
              onPress={this._toggleCollapse}
              collapsed={this._collapsed}
            />
          ) : null}
        </Cell.CellButton>

        {hasChildren && (
          <Accordion ref={this._accordionRef} defaultCollapsed={this._collapsed}>
            {this.props.children}
          </Accordion>
        )}
      </View>
    );
  }

  _toggleCollapse = (event: Event) => {
    const accordion = this._accordionRef.current;
    const collapseButton = this._collapseButtonRef.current;

    if (accordion && collapseButton) {
      this._collapsed = !accordion.isCollapsed;

      collapseButton.setAppearance(this._collapsed);
      accordion.toggle();
    }

    event.stopPropagation();
  };
}

type CityRowProps = {
  countryCode: string,
  relayCity: RelayLocationCityRedux,
  defaultSelected: boolean,
  defaultCollapsed: boolean,
  onPress?: () => void,
  children?: React.Element<typeof RelayRow>,
};

class CityRow extends Component<CityRowProps> {
  _accordionRef = React.createRef();
  _collapseButtonRef = React.createRef();
  _collapsed = false;

  state = {
    selected: false,
  };

  setSelected(selected: boolean) {
    this.setState({ selected });
  }

  constructor(props: CityRowProps) {
    super(props);

    this._collapsed = props.defaultCollapsed;
    this.state.selected = props.defaultSelected;
  }

  render() {
    const { relayCity } = this.props;
    const hasChildren = this.props.children.length > 1;

    return (
      <View>
        <Cell.CellButton
          onPress={this.props.onPress}
          disabled={!relayCity.hasActiveRelays}
          cellHoverStyle={this.state.selected ? styles.sub_cell__selected : null}
          style={this.state.selected ? styles.sub_cell__selected : styles.sub_cell}
          testName="city">
          <RelayStatusIndicator
            isActive={relayCity.hasActiveRelays}
            isSelected={this.state.selected}
          />
          <Cell.Label>{relayCity.name}</Cell.Label>

          {hasChildren && (
            <CollapseButton
              ref={this._collapseButtonRef}
              onPress={this._toggleCollapse}
              collapsed={this._collapsed}
            />
          )}
        </Cell.CellButton>

        {hasChildren && (
          <Accordion ref={this._accordionRef} defaultCollapsed={this._collapsed}>
            {this.props.children}
          </Accordion>
        )}
      </View>
    );
  }

  _toggleCollapse = (event: Event) => {
    const accordion = this._accordionRef.current;
    const collapseButton = this._collapseButtonRef.current;

    if (accordion && collapseButton) {
      this._collapsed = !accordion.isCollapsed;

      collapseButton.setAppearance(this._collapsed);
      accordion.toggle();
    }

    event.stopPropagation();
  };
}

type RelayRowProps = {
  defaultSelected: boolean,
  countryCode: string,
  cityCode: string,
  relay: RelayLocationRelayRedux,
  onPress?: () => void,
};

class RelayRow extends Component<RelayRowProps> {
  state = {
    selected: false,
  };

  setSelected(selected: boolean) {
    this.setState({ selected });
  }

  constructor(props: RelayRowProps) {
    super(props);

    this.state.selected = props.defaultSelected;
  }

  render() {
    const { relay } = this.props;

    return (
      <Cell.CellButton
        onPress={this.props.onPress}
        cellHoverStyle={this.state.selected ? styles.sub_sub_cell__selected : null}
        style={this.state.selected ? styles.sub_sub_cell__selected : styles.sub_sub_cell}
        testName="relay">
        <RelayStatusIndicator isActive={true} isSelected={this.state.selected} />

        <Cell.Label>{relay.hostname}</Cell.Label>
      </Cell.CellButton>
    );
  }
}

type RelayStatusIndicatorProps = {
  isActive: boolean,
  isSelected: boolean,
};

class RelayStatusIndicator extends Component<RelayStatusIndicatorProps> {
  render() {
    const statusClass = this.props.isActive
      ? styles.relay_status__active
      : styles.relay_status__inactive;

    return this.props.isSelected ? (
      <Cell.Icon
        style={styles.tick_icon}
        tintColor={colors.white}
        source="icon-tick"
        height={24}
        width={24}
      />
    ) : (
      <View style={[styles.relay_status, statusClass]} />
    );
  }
}

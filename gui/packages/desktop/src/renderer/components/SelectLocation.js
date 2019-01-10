// @flow

import * as React from 'react';
import ReactDOM from 'react-dom';
import { View, Component } from 'reactxp';
import { SettingsHeader, HeaderTitle, HeaderSubTitle } from '@mullvad/components';
import { Layout, Container } from './Layout';
import {
  NavigationContainer,
  NavigationScrollbars,
  NavigationBar,
  CloseBarItem,
  TitleBarItem,
} from './NavigationBar';
import styles from './SelectLocationStyles';

import CountryRow from './CountryRow';
import CityRow from './CityRow';
import RelayRow from './RelayRow';

import type { RelaySettingsRedux, RelayLocationRedux } from '../redux/settings/reducers';
import type { RelayLocation } from '../lib/daemon-rpc-proxy';

type Props = {
  relaySettings: RelaySettingsRedux,
  relayLocations: Array<RelayLocationRedux>,
  onClose: () => void,
  onSelect: (location: RelayLocation) => void,
};

type State = {
  selectedLocation?: RelayLocation,
  expandedItems: Array<RelayLocation>,
};

export default class SelectLocation extends Component<Props, State> {
  _selectedCellRef = React.createRef();
  _scrollViewRef = React.createRef();

  state = {
    expandedItems: [],
  };

  constructor(props: Props) {
    super(props);

    if (this.props.relaySettings.normal) {
      const expandedItems = [];
      const location = this.props.relaySettings.normal.location;

      if (location.city) {
        expandedItems.push({ country: location.city[0] });
      }

      if (location.hostname) {
        expandedItems.push({ country: location.hostname[0] });
        expandedItems.push({ city: [location.hostname[0], location.hostname[1]] });
      }

      this.state = {
        selectedLocation: location,
        expandedItems,
      };
    }
  }

  componentDidUpdate() {
    if (this.props.relaySettings.normal) {
      const location = this.props.relaySettings.normal.location;

      if (
        !this.state.selectedLocation ||
        this._getLocationKey(this.state.selectedLocation) !== this._getLocationKey(location)
      ) {
        this._setSelectedLocation(location);
      }
    } else {
      if (this.state.selectedLocation) {
        this._setSelectedLocation(undefined);
      }
    }
  }

  componentDidMount() {
    // restore scroll to selected cell
    const cell = this._selectedCellRef.current;
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
                      const isSelected = this._isSelected(location);
                      const ref = isSelected ? this._selectedCellRef : undefined;

                      return (
                        <CountryRow
                          key={this._getLocationKey(location)}
                          name={relayCountry.name}
                          hasActiveRelays={relayCountry.hasActiveRelays}
                          ref={ref}
                          selected={isSelected}
                          collapsed={this._isCollapsed(location)}
                          onPress={() => this._handleSelection(location)}
                          onCollapse={(collapse) => this._handleCollapse(location, collapse)}>
                          {relayCountry.cities.map((relayCity) => {
                            const location = { city: [relayCountry.code, relayCity.code] };
                            const isSelected = this._isSelected(location);
                            const ref = isSelected ? this._selectedCellRef : undefined;
                            const key = this._getLocationKey(location);

                            return (
                              <CityRow
                                key={key}
                                ref={ref}
                                name={relayCity.name}
                                hasActiveRelays={relayCity.hasActiveRelays}
                                selected={isSelected}
                                collapsed={this._isCollapsed(location)}
                                onPress={() => this._handleSelection(location)}
                                onCollapse={(collapse) => this._handleCollapse(location, collapse)}>
                                {relayCity.relays.map((relay) => {
                                  const location = {
                                    hostname: [relayCountry.code, relayCity.code, relay.hostname],
                                  };
                                  const isSelected = this._isSelected(location);
                                  const ref = isSelected ? this._selectedCellRef : undefined;
                                  const key = this._getLocationKey(location);

                                  return (
                                    <RelayRow
                                      key={key}
                                      ref={ref}
                                      hostname={relay.hostname}
                                      selected={isSelected}
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
    for (const location of this.state.expandedItems) {
      if (this._testLocationEqual(location, relayLocation)) {
        return false;
      }
    }
    return true;
  }

  _testLocationEqual(lhs: RelayLocation, rhs: RelayLocation) {
    if (lhs.country && rhs.country) {
      return lhs.country === rhs.country;
    } else if (lhs.city && rhs.city) {
      return lhs.city[0] === rhs.city[0] && lhs.city[1] === rhs.city[1];
    } else if (lhs.hostname && rhs.hostname) {
      return (
        lhs.hostname[0] === rhs.hostname[0] &&
        lhs.hostname[1] === rhs.hostname[1] &&
        lhs.hostname[2] === rhs.hostname[2]
      );
    } else {
      return false;
    }
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

  _setSelectedLocation(location: ?RelayLocation, callback?: () => void) {
    this.setState({ selectedLocation: location }, callback);
  }

  _handleSelection = (location: RelayLocation) => {
    this._setSelectedLocation(location, () => {
      this.props.onSelect(location);
    });
  };

  _handleCollapse = (location: RelayLocation, collapse: boolean) => {
    this.setState((state) => {
      const expandedItems = state.expandedItems.filter(
        (item) => !this._testLocationEqual(item, location),
      );

      if (!collapse) {
        expandedItems.push(location);
      }

      return {
        ...state,
        expandedItems,
      };
    });
  };

  _getLocationKey(location: RelayLocation) {
    const components = location.city || location.country || location.hostname || [];
    return [].concat(components).join('-');
  }
}

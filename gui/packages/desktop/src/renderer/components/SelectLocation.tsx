import * as React from 'react';
import ReactDOM from 'react-dom';
import { View, Component } from 'reactxp';
import { SettingsHeader, HeaderTitle, HeaderSubTitle } from '@mullvad/components';
import { Layout, Container } from './Layout';
import CustomScrollbars from './CustomScrollbars';
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

import { RelaySettingsRedux, RelayLocationRedux } from '../redux/settings/reducers';
import { RelayLocation } from '../../shared/daemon-rpc-types';

type Props = {
  relaySettings: RelaySettingsRedux;
  relayLocations: RelayLocationRedux[];
  onClose: () => void;
  onSelect: (location: RelayLocation) => void;
};

type State = {
  selectedLocation?: RelayLocation;
  expandedItems: RelayLocation[];
};

export default class SelectLocation extends Component<Props, State> {
  _selectedCellRef = React.createRef<React.ReactNode>();
  _scrollViewRef = React.createRef<CustomScrollbars>();

  state: State = {
    expandedItems: [],
  };

  constructor(props: Props) {
    super(props);

    if ('normal' in this.props.relaySettings) {
      const location = this.props.relaySettings.normal.location;

      if (typeof location === 'object') {
        const expandedItems: RelayLocation[] = [];

        if ('city' in location) {
          expandedItems.push({ country: location.city[0] });
        } else if ('hostname' in location) {
          expandedItems.push({ country: location.hostname[0] });
          expandedItems.push({ city: [location.hostname[0], location.hostname[1]] });
        }

        this.state = {
          selectedLocation: location,
          expandedItems,
        };
      }
    }
  }

  componentDidUpdate(oldProps: Props) {
    const currentLocation = this.state.selectedLocation;
    let newLocation =
      'normal' in this.props.relaySettings ? this.props.relaySettings.normal.location : undefined;

    let oldLocation =
      'normal' in oldProps.relaySettings ? oldProps.relaySettings.normal.location : undefined;

    if (newLocation === 'any') {
      newLocation = undefined;
    }

    if (oldLocation === 'any') {
      oldLocation = undefined;
    }

    if (
      !compareLocationLoose(oldLocation, newLocation) &&
      !compareLocationLoose(currentLocation, newLocation)
    ) {
      this.setState({ selectedLocation: newLocation });
    }
  }

  componentDidMount() {
    // restore scroll to the selected cell
    const cell = this._selectedCellRef.current;
    const scrollView = this._scrollViewRef.current;
    if (scrollView && cell) {
      // TODO: Fix the browser specific code
      const cellDOMNode = ReactDOM.findDOMNode(cell as Element);
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
                      const location: RelayLocation = { country: relayCountry.code };

                      return (
                        <CountryRow
                          key={getLocationKey(location)}
                          name={relayCountry.name}
                          hasActiveRelays={relayCountry.hasActiveRelays}
                          expanded={this._isExpanded(location)}
                          onSelect={() => this._handleSelection(location)}
                          onExpand={(expand) => this._handleExpand(location, expand)}
                          {...this._getCommonCellProps(location)}>
                          {relayCountry.cities.map((relayCity) => {
                            const location: RelayLocation = {
                              city: [relayCountry.code, relayCity.code],
                            };

                            return (
                              <CityRow
                                key={getLocationKey(location)}
                                name={relayCity.name}
                                hasActiveRelays={relayCity.hasActiveRelays}
                                expanded={this._isExpanded(location)}
                                onSelect={() => this._handleSelection(location)}
                                onExpand={(expand) => this._handleExpand(location, expand)}
                                {...this._getCommonCellProps(location)}>
                                {relayCity.relays.map((relay) => {
                                  const location: RelayLocation = {
                                    hostname: [relayCountry.code, relayCity.code, relay.hostname],
                                  };

                                  return (
                                    <RelayRow
                                      key={getLocationKey(location)}
                                      hostname={relay.hostname}
                                      onSelect={() => this._handleSelection(location)}
                                      {...this._getCommonCellProps(location)}
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

  _isExpanded(relayLocation: RelayLocation) {
    return this.state.expandedItems.some((location) => compareLocation(location, relayLocation));
  }

  _isSelected(relayLocation: RelayLocation) {
    return compareLocationLoose(this.state.selectedLocation, relayLocation);
  }

  _handleSelection = (location: RelayLocation) => {
    if (!compareLocationLoose(this.state.selectedLocation, location)) {
      this.setState({ selectedLocation: location }, () => {
        this.props.onSelect(location);
      });
    }
  };

  _handleExpand = (location: RelayLocation, expand: boolean) => {
    this.setState((state) => {
      const expandedItems = state.expandedItems.filter((item) => !compareLocation(item, location));

      if (expand) {
        expandedItems.push(location);
      }

      return {
        ...state,
        expandedItems,
      };
    });
  };

  _getCommonCellProps<T>(location: RelayLocation): { selected: boolean; ref?: React.RefObject<T> } {
    const selected = this._isSelected(location);
    const ref = selected ? (this._selectedCellRef as React.RefObject<T>) : undefined;

    return { ref, selected };
  }
}

function getLocationKey(location: RelayLocation): string {
  const components: string[] = [];

  if ('city' in location) {
    components.push(...location.city);
  } else if ('country' in location) {
    components.push(location.country);
  } else if ('hostname' in location) {
    components.push(...location.hostname);
  }

  return ([] as string[]).concat(components).join('-');
}

function compareLocation(lhs: RelayLocation, rhs: RelayLocation) {
  if ('country' in lhs && 'country' in rhs && lhs.country && rhs.country) {
    return lhs.country === rhs.country;
  } else if ('city' in lhs && 'city' in rhs && lhs.city && rhs.city) {
    return lhs.city[0] === rhs.city[0] && lhs.city[1] === rhs.city[1];
  } else if ('hostname' in lhs && 'hostname' in rhs && lhs.hostname && rhs.hostname) {
    return (
      lhs.hostname[0] === rhs.hostname[0] &&
      lhs.hostname[1] === rhs.hostname[1] &&
      lhs.hostname[2] === rhs.hostname[2]
    );
  } else {
    return false;
  }
}

function compareLocationLoose(lhs?: RelayLocation, rhs?: RelayLocation) {
  if (lhs && rhs) {
    return compareLocation(lhs, rhs);
  } else {
    return lhs === rhs;
  }
}

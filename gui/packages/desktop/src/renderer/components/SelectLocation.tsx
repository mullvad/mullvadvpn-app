import * as React from 'react';
import ReactDOM from 'react-dom';
import { Component, View } from 'reactxp';
import { pgettext } from '../../shared/gettext';
import CustomScrollbars from './CustomScrollbars';
import { Container, Layout } from './Layout';
import {
  CloseBarItem,
  NavigationBar,
  NavigationContainer,
  NavigationScrollbars,
  TitleBarItem,
} from './NavigationBar';
import styles from './SelectLocationStyles';
import SettingsHeader, { HeaderSubTitle, HeaderTitle } from './SettingsHeader';

import CityRow from './CityRow';
import CountryRow from './CountryRow';
import RelayRow from './RelayRow';

import {
  compareRelayLocation,
  compareRelayLocationLoose,
  RelayLocation,
} from '../../shared/daemon-rpc-types';
import { IRelayLocationRedux, RelaySettingsRedux } from '../redux/settings/reducers';

interface IProps {
  relaySettings: RelaySettingsRedux;
  relayLocations: IRelayLocationRedux[];
  onClose: () => void;
  onSelect: (location: RelayLocation) => void;
}

interface IState {
  selectedLocation?: RelayLocation;
  expandedItems: RelayLocation[];
}

export default class SelectLocation extends Component<IProps, IState> {
  public state: IState = {
    expandedItems: [],
  };
  private selectedCellRef = React.createRef<React.ReactNode>();
  private scrollViewRef = React.createRef<CustomScrollbars>();

  constructor(props: IProps) {
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

  public componentDidUpdate(oldProps: IProps) {
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
      !compareRelayLocationLoose(oldLocation, newLocation) &&
      !compareRelayLocationLoose(currentLocation, newLocation)
    ) {
      this.setState({ selectedLocation: newLocation });
    }
  }

  public componentDidMount() {
    // restore scroll to the selected cell
    const cell = this.selectedCellRef.current;
    const scrollView = this.scrollViewRef.current;
    if (scrollView && cell) {
      // TODO: Fix the browser specific code
      const cellDOMNode = ReactDOM.findDOMNode(cell as Element);
      if (cellDOMNode instanceof HTMLElement) {
        scrollView.scrollToElement(cellDOMNode, 'middle');
      }
    }
  }

  public render() {
    return (
      <Layout>
        <Container>
          <View style={styles.select_location}>
            <NavigationContainer>
              <NavigationBar>
                <CloseBarItem action={this.props.onClose} />
                <TitleBarItem>
                  {// TRANSLATORS: Title label in navigation bar
                  pgettext('select-location-nav', 'Select location')}
                </TitleBarItem>
              </NavigationBar>
              <View style={styles.container}>
                <NavigationScrollbars ref={this.scrollViewRef}>
                  <View style={styles.content}>
                    <SettingsHeader style={styles.subtitle_header}>
                      <HeaderTitle>
                        {pgettext('select-location-view', 'Select location')}
                      </HeaderTitle>
                      <HeaderSubTitle>
                        {pgettext(
                          'select-location-view',
                          'While connected, your real location is masked with a private and secure location in the selected region',
                        )}
                      </HeaderSubTitle>
                    </SettingsHeader>

                    {this.props.relayLocations.map((relayCountry) => {
                      const countryLocation: RelayLocation = { country: relayCountry.code };

                      return (
                        <CountryRow
                          key={getLocationKey(countryLocation)}
                          name={relayCountry.name}
                          hasActiveRelays={relayCountry.hasActiveRelays}
                          expanded={this.isExpanded(countryLocation)}
                          onSelect={this.handleSelection}
                          onExpand={this.handleExpand}
                          {...this.getCommonCellProps(countryLocation)}>
                          {relayCountry.cities.map((relayCity) => {
                            const cityLocation: RelayLocation = {
                              city: [relayCountry.code, relayCity.code],
                            };

                            return (
                              <CityRow
                                key={getLocationKey(cityLocation)}
                                name={relayCity.name}
                                hasActiveRelays={relayCity.hasActiveRelays}
                                expanded={this.isExpanded(cityLocation)}
                                onSelect={this.handleSelection}
                                onExpand={this.handleExpand}
                                {...this.getCommonCellProps(cityLocation)}>
                                {relayCity.relays.map((relay) => {
                                  const relayLocation: RelayLocation = {
                                    hostname: [relayCountry.code, relayCity.code, relay.hostname],
                                  };

                                  return (
                                    <RelayRow
                                      key={getLocationKey(relayLocation)}
                                      hostname={relay.hostname}
                                      onSelect={this.handleSelection}
                                      {...this.getCommonCellProps(relayLocation)}
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

  private isExpanded(relayLocation: RelayLocation) {
    return this.state.expandedItems.some((location) =>
      compareRelayLocation(location, relayLocation),
    );
  }

  private isSelected(relayLocation: RelayLocation) {
    return compareRelayLocationLoose(this.state.selectedLocation, relayLocation);
  }

  private handleSelection = (location: RelayLocation) => {
    if (!compareRelayLocationLoose(this.state.selectedLocation, location)) {
      this.setState({ selectedLocation: location }, () => {
        this.props.onSelect(location);
      });
    }
  };

  private handleExpand = (location: RelayLocation, expand: boolean) => {
    this.setState((state) => {
      const expandedItems = state.expandedItems.filter(
        (item) => !compareRelayLocation(item, location),
      );

      if (expand) {
        expandedItems.push(location);
      }

      return {
        ...state,
        expandedItems,
      };
    });
  };

  private getCommonCellProps<T>(
    location: RelayLocation,
  ): { location: RelayLocation; selected: boolean; ref?: React.RefObject<T> } {
    const selected = this.isSelected(location);
    const ref = selected ? (this.selectedCellRef as React.RefObject<T>) : undefined;

    return { ref, selected, location };
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

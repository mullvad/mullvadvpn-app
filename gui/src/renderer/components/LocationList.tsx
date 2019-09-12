import * as React from 'react';
import { Component, View } from 'reactxp';
import {
  compareRelayLocation,
  compareRelayLocationLoose,
  RelayLocation,
  relayLocationComponents,
} from '../../shared/daemon-rpc-types';
import { countries, relayLocations } from '../../shared/gettext';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import CityRow from './CityRow';
import CountryRow from './CountryRow';
import RelayRow from './RelayRow';

interface IProps {
  relayLocations: IRelayLocationRedux[];
  selectedLocation?: RelayLocation;
  onSelect: (location: RelayLocation) => void;
}

interface IState {
  selectedLocation?: RelayLocation;
  expandedItems: RelayLocation[];
}

interface ICommonCellProps<T> {
  location: RelayLocation;
  selected: boolean;
  ref?: React.RefObject<T>;
}

export default class LocationList extends Component<IProps, IState> {
  public selectedCell = React.createRef<React.ReactNode>();

  constructor(props: IProps) {
    super(props);

    this.state = {
      expandedItems: props.selectedLocation ? expandRelayLocation(props.selectedLocation) : [],
      selectedLocation: props.selectedLocation,
    };
  }

  public componentDidUpdate(prevProps: IProps, _prevState: IState) {
    if (this.props.selectedLocation !== prevProps.selectedLocation) {
      this.setState({ selectedLocation: this.props.selectedLocation });
    }
  }

  public render() {
    return (
      <View>
        {this.props.relayLocations.map((relayCountry) => {
          const countryLocation: RelayLocation = { country: relayCountry.code };

          return (
            <CountryRow
              key={getLocationKey(countryLocation)}
              name={countries.gettext(relayCountry.name)}
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
                    name={relayLocations.gettext(relayCity.name)}
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
                          active={relay.active}
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

  private getCommonCellProps<T>(location: RelayLocation): ICommonCellProps<T> {
    const selected = this.isSelected(location);
    const ref = selected ? (this.selectedCell as React.RefObject<T>) : undefined;

    return { ref, selected, location };
  }
}

function expandRelayLocation(location: RelayLocation): RelayLocation[] {
  const expandedItems: RelayLocation[] = [];

  if ('city' in location) {
    expandedItems.push({ country: location.city[0] });
  } else if ('hostname' in location) {
    expandedItems.push({ country: location.hostname[0] });
    expandedItems.push({ city: [location.hostname[0], location.hostname[1]] });
  }

  return expandedItems;
}

function getLocationKey(location: RelayLocation): string {
  return relayLocationComponents(location).join('-');
}

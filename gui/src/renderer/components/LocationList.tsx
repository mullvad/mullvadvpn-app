import * as React from 'react';
import styled from 'styled-components';
import { Component, View } from 'reactxp';
import { colors } from '../../config.json';
import {
  compareRelayLocation,
  compareRelayLocationLoose,
  RelayLocation,
  relayLocationComponents,
} from '../../shared/daemon-rpc-types';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import * as Cell from './Cell';
import CityRow from './CityRow';
import CountryRow from './CountryRow';
import RelayRow from './RelayRow';

export enum LocationSelectionType {
  relay = 'relay',
  special = 'special',
}

export type LocationSelection<SpecialValueType> =
  | { type: LocationSelectionType.special; value: SpecialValueType }
  | { type: LocationSelectionType.relay; value: RelayLocation };

interface ILocationListState<SpecialValueType> {
  selectedValue?: LocationSelection<SpecialValueType>;
  expandedLocations: RelayLocation[];
}

interface ILocationListProps<SpecialValueType> {
  defaultExpandedLocations?: RelayLocation[];
  selectedValue?: LocationSelection<SpecialValueType>;
  selectedElementRef?: React.Ref<React.ReactInstance>;
  onSelect?: (value: LocationSelection<SpecialValueType>) => void;
}

export default class LocationList<SpecialValueType> extends Component<
  ILocationListProps<SpecialValueType>,
  ILocationListState<SpecialValueType>
> {
  public state: ILocationListState<SpecialValueType> = {
    expandedLocations: [],
  };

  public selectedRelayLocationRef: React.ReactInstance | null = null;
  public selectedSpecialLocationRef: React.ReactInstance | null = null;

  constructor(props: ILocationListProps<SpecialValueType>) {
    super(props);

    if (props.selectedValue) {
      const expandedLocations =
        props.defaultExpandedLocations ||
        (props.selectedValue.type === LocationSelectionType.relay
          ? expandRelayLocation(props.selectedValue.value)
          : []);

      this.state = {
        selectedValue: props.selectedValue,
        expandedLocations,
      };
    }
  }

  public getExpandedLocations(): RelayLocation[] {
    return this.state.expandedLocations;
  }

  public componentDidUpdate(prevProps: ILocationListProps<SpecialValueType>) {
    if (!compareLocationSelectionLoose(prevProps.selectedValue, this.props.selectedValue)) {
      this.setState({ selectedValue: this.props.selectedValue });
    }
  }

  public render() {
    const selection = this.state.selectedValue;
    const specialSelection =
      selection && selection.type === LocationSelectionType.special ? selection.value : undefined;
    const relaySelection =
      selection && selection.type === LocationSelectionType.relay ? selection.value : undefined;

    return (
      <View>
        {React.Children.map(this.props.children, (child) => {
          if (React.isValidElement(child)) {
            if (child.type === SpecialLocations) {
              return React.cloneElement(child, {
                ...child.props,
                selectedElementRef: this.onSpecialLocationRef,
                selectedValue: specialSelection,
                onSelect: this.onSelectSpecialLocation,
              });
            } else if (child.type === RelayLocations) {
              return React.cloneElement(child, {
                ...child.props,
                selectedLocation: relaySelection,
                selectedElementRef: this.onRelayLocationRef,
                expandedItems: this.state.expandedLocations,
                onSelect: this.onSelectRelayLocation,
                onExpand: this.onExpandRelayLocation,
              });
            }
          }
          return child;
        })}
      </View>
    );
  }

  private onSpecialLocationRef = (ref: React.ReactInstance | null) => {
    this.selectedSpecialLocationRef = ref;

    this.updateExternalRef();
  };

  private onRelayLocationRef = (ref: React.ReactInstance | null) => {
    this.selectedRelayLocationRef = ref;

    this.updateExternalRef();
  };

  private updateExternalRef() {
    if (this.props.selectedElementRef) {
      const value = this.selectedRelayLocationRef || this.selectedSpecialLocationRef;

      if (typeof this.props.selectedElementRef === 'function') {
        this.props.selectedElementRef(value);
      } else {
        // @ts-ignore
        this.props.selectedElementRef.current = value;
      }
    }
  }

  private onSelectRelayLocation = (value: RelayLocation) => {
    const selectedValue: LocationSelection<SpecialValueType> = {
      type: LocationSelectionType.relay,
      value,
    };

    this.setState({ selectedValue }, () => {
      this.notifySelection(selectedValue);
    });
  };

  private onSelectSpecialLocation = (value: SpecialValueType) => {
    const selectedValue: LocationSelection<SpecialValueType> = {
      type: LocationSelectionType.special,
      value,
    };

    this.setState({ selectedValue }, () => {
      this.notifySelection(selectedValue);
    });
  };

  private notifySelection(value: LocationSelection<SpecialValueType>) {
    if (this.props.onSelect) {
      this.props.onSelect(value);
    }
  }

  private onExpandRelayLocation = (location: RelayLocation, expand: boolean) => {
    this.setState((state) => {
      const expandedLocations = state.expandedLocations.filter(
        (item) => !compareRelayLocation(item, location),
      );

      if (expand) {
        expandedLocations.push(location);
      }

      return {
        ...state,
        expandedLocations,
      };
    });
  };
}

export enum SpecialLocationIcon {
  geoLocation = 'icon-nearest',
}

interface ISpecialLocationsProps<T> {
  children: React.ReactNode;
  selectedValue?: T;
  selectedElementRef?: React.Ref<SpecialLocation<T>>;
  onSelect?: (value: T) => void;
}

export function SpecialLocations<T>(props: ISpecialLocationsProps<T>) {
  return (
    <View>
      {React.Children.map(props.children, (child) => {
        if (React.isValidElement(child) && child.type === SpecialLocation) {
          const isSelected = props.selectedValue === child.props.value;

          return React.cloneElement(child, {
            ...child.props,
            ref: isSelected ? props.selectedElementRef : undefined,
            onSelect: props.onSelect,
            isSelected,
          });
        } else {
          return undefined;
        }
      })}
    </View>
  );
}

const StyledSpecialLocationCellButton = styled(Cell.CellButton)({
  paddingLeft: '18px',
});

const StyledSpecialLocationCellLabel = styled(Cell.Label)({
  fontFamily: 'Open Sans',
  fontWeight: 'normal',
  fontSize: '16px',
});

const StyledSpecialLocationIcon = styled(Cell.Icon)({
  marginRight: '8px',
});

interface ISpecialLocationProps<T> {
  icon: SpecialLocationIcon;
  value: T;
  isSelected?: boolean;
  onSelect?: (value: T) => void;
}

export class SpecialLocation<T> extends Component<ISpecialLocationProps<T>> {
  public render() {
    return (
      <StyledSpecialLocationCellButton selected={this.props.isSelected} onClick={this.onSelect}>
        <StyledSpecialLocationIcon
          source={this.props.isSelected ? 'icon-tick' : this.props.icon}
          tintColor={colors.white}
          height={24}
          width={24}
        />
        <StyledSpecialLocationCellLabel>{this.props.children}</StyledSpecialLocationCellLabel>
      </StyledSpecialLocationCellButton>
    );
  }

  private onSelect = () => {
    if (!this.props.isSelected && this.props.onSelect) {
      this.props.onSelect(this.props.value);
    }
  };
}

interface IRelayLocationsProps {
  source: IRelayLocationRedux[];
  selectedLocation?: RelayLocation;
  selectedElementRef?: React.Ref<React.ReactInstance>;
  expandedItems?: RelayLocation[];
  onSelect?: (location: RelayLocation) => void;
  onExpand?: (location: RelayLocation, expand: boolean) => void;
  onWillExpand?: (locationRect: DOMRect, expandedContentHeight: number) => void;
  onTransitionEnd?: () => void;
}

interface ICommonCellProps<T> {
  location: RelayLocation;
  selected: boolean;
  ref?: React.Ref<T>;
}

export class RelayLocations extends Component<IRelayLocationsProps> {
  public render() {
    return (
      <View>
        {this.props.source.map((relayCountry) => {
          const countryLocation: RelayLocation = { country: relayCountry.code };

          return (
            <CountryRow
              key={getLocationKey(countryLocation)}
              name={relayCountry.name}
              hasActiveRelays={relayCountry.hasActiveRelays}
              expanded={this.isExpanded(countryLocation)}
              onSelect={this.handleSelection}
              onExpand={this.handleExpand}
              onWillExpand={this.props.onWillExpand}
              onTransitionEnd={this.props.onTransitionEnd}
              {...this.getCommonCellProps<CountryRow>(countryLocation)}>
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
                    onWillExpand={this.props.onWillExpand}
                    onTransitionEnd={this.props.onTransitionEnd}
                    {...this.getCommonCellProps<CityRow>(cityLocation)}>
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
                          {...this.getCommonCellProps<RelayRow>(relayLocation)}
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
    return (this.props.expandedItems || []).some((location) =>
      compareRelayLocation(location, relayLocation),
    );
  }

  private isSelected(relayLocation: RelayLocation) {
    return compareRelayLocationLoose(this.props.selectedLocation, relayLocation);
  }

  private handleSelection = (location: RelayLocation) => {
    if (!compareRelayLocationLoose(this.props.selectedLocation, location)) {
      if (this.props.onSelect) {
        this.props.onSelect(location);
      }
    }
  };

  private handleExpand = (location: RelayLocation, expand: boolean) => {
    if (this.props.onExpand) {
      this.props.onExpand(location, expand);
    }
  };

  private getCommonCellProps<T>(location: RelayLocation): ICommonCellProps<T> {
    const selected = this.isSelected(location);
    const ref =
      selected && this.props.selectedElementRef ? this.props.selectedElementRef : undefined;

    return { ref: ref as React.Ref<T>, selected, location };
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

function compareLocationSelectionLoose<SpecialValueType>(
  lhs?: LocationSelection<SpecialValueType>,
  rhs?: LocationSelection<SpecialValueType>,
) {
  if (!lhs || !rhs) {
    return lhs === rhs;
  } else if (lhs.type === LocationSelectionType.relay && rhs.type === LocationSelectionType.relay) {
    return compareRelayLocation(lhs.value, rhs.value);
  } else {
    return lhs.value === rhs.value;
  }
}

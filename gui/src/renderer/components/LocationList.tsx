import * as React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import {
  compareRelayLocation,
  compareRelayLocationLoose,
  RelayLocation,
  relayLocationComponents,
} from '../../shared/daemon-rpc-types';
import { IRelayLocationRedux } from '../redux/settings/reducers';
import * as Cell from './Cell';
import LocationRow from './LocationRow';

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

export default class LocationList<SpecialValueType> extends React.Component<
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
      <>
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
      </>
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
    <>
      {React.Children.map(props.children, (child) => {
        if (React.isValidElement(child) && child.type === SpecialLocation) {
          const isSelected = props.selectedValue === child.props.value;

          return React.cloneElement(child, {
            ...child.props,
            forwardedRef: isSelected ? props.selectedElementRef : undefined,
            onSelect: props.onSelect,
            isSelected,
          });
        } else {
          return undefined;
        }
      })}
    </>
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
  forwardedRef?: React.Ref<HTMLButtonElement>;
}

export class SpecialLocation<T> extends React.Component<ISpecialLocationProps<T>> {
  public render() {
    return (
      <StyledSpecialLocationCellButton
        ref={this.props.forwardedRef}
        selected={this.props.isSelected}
        onClick={this.onSelect}>
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

interface ICommonCellProps {
  location: RelayLocation;
  selected: boolean;
  ref?: React.Ref<HTMLDivElement>;
}

export class RelayLocations extends React.PureComponent<IRelayLocationsProps> {
  public render() {
    return (
      <>
        {this.props.source.map((relayCountry) => {
          const countryLocation: RelayLocation = { country: relayCountry.code };

          return (
            <LocationRow
              key={getLocationKey(countryLocation)}
              name={relayCountry.name}
              active={relayCountry.hasActiveRelays}
              expanded={this.isExpanded(countryLocation)}
              onSelect={this.handleSelection}
              onExpand={this.handleExpand}
              onWillExpand={this.props.onWillExpand}
              onTransitionEnd={this.props.onTransitionEnd}
              {...this.getCommonCellProps(countryLocation)}>
              {relayCountry.cities.map((relayCity) => {
                const cityLocation: RelayLocation = {
                  city: [relayCountry.code, relayCity.code],
                };

                return (
                  <LocationRow
                    key={getLocationKey(cityLocation)}
                    name={relayCity.name}
                    active={relayCity.hasActiveRelays}
                    expanded={this.isExpanded(cityLocation)}
                    onSelect={this.handleSelection}
                    onExpand={this.handleExpand}
                    onWillExpand={this.props.onWillExpand}
                    onTransitionEnd={this.props.onTransitionEnd}
                    {...this.getCommonCellProps(cityLocation)}>
                    {relayCity.relays.map((relay) => {
                      const relayLocation: RelayLocation = {
                        hostname: [relayCountry.code, relayCity.code, relay.hostname],
                      };

                      return (
                        <LocationRow
                          key={getLocationKey(relayLocation)}
                          name={relay.hostname}
                          active={relay.active}
                          onSelect={this.handleSelection}
                          {...this.getCommonCellProps(relayLocation)}
                        />
                      );
                    })}
                  </LocationRow>
                );
              })}
            </LocationRow>
          );
        })}
      </>
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

  private getCommonCellProps(location: RelayLocation): ICommonCellProps {
    const selected = this.isSelected(location);
    const ref =
      selected && this.props.selectedElementRef ? this.props.selectedElementRef : undefined;

    return { ref: ref as React.Ref<HTMLDivElement>, selected, location };
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

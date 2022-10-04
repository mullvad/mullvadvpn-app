import * as React from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../config.json';
import {
  compareRelayLocation,
  compareRelayLocationLoose,
  RelayLocation,
  relayLocationComponents,
} from '../../shared/daemon-rpc-types';
import { messages, relayLocations } from '../../shared/gettext';
import {
  IRelayLocationCityRedux,
  IRelayLocationRedux,
  IRelayLocationRelayRedux,
} from '../redux/settings/reducers';
import * as Cell from './cell';
import InfoButton from './InfoButton';
import LocationRow, {
  StyledLocationRowButton,
  StyledLocationRowContainer,
  StyledLocationRowIcon,
  StyledLocationRowLabel,
} from './LocationRow';

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
  children?: React.ReactNode;
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
        const ref = this.props
          .selectedElementRef as React.MutableRefObject<React.ReactInstance | null>;
        ref.current = value;
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

const StyledLocationRowContainerWithMargin = styled(StyledLocationRowContainer)({
  marginBottom: 1,
});

const StyledSpecialLocationIcon = styled(Cell.Icon)({
  flex: 0,
  marginLeft: '2px',
  marginRight: '8px',
});

const StyledSpecialLocationInfoButton = styled(InfoButton)({
  margin: 0,
  padding: '0 25px',
});

interface ISpecialLocationProps<T> {
  icon: SpecialLocationIcon;
  value: T;
  isSelected?: boolean;
  onSelect?: (value: T) => void;
  info?: string;
  forwardedRef?: React.Ref<HTMLButtonElement>;
  children?: React.ReactNode;
}

export class SpecialLocation<T> extends React.Component<ISpecialLocationProps<T>> {
  public render() {
    return (
      <StyledLocationRowContainerWithMargin>
        <StyledLocationRowButton onClick={this.onSelect} selected={this.props.isSelected ?? false}>
          <StyledSpecialLocationIcon
            source={this.props.isSelected ? 'icon-tick' : this.props.icon}
            tintColor={colors.white}
            height={22}
            width={22}
          />
          <StyledLocationRowLabel>{this.props.children}</StyledLocationRowLabel>
        </StyledLocationRowButton>
        <StyledLocationRowIcon
          as={StyledSpecialLocationInfoButton}
          message={this.props.info}
          selected={this.props.isSelected ?? false}
          aria-label={messages.pgettext('accessibility', 'info')}
        />
      </StyledLocationRowContainerWithMargin>
    );
  }

  private onSelect = () => {
    if (!this.props.isSelected && this.props.onSelect) {
      this.props.onSelect(this.props.value);
    }
  };
}

export enum DisabledReason {
  entry,
  exit,
  inactive,
}

interface IRelayLocationsProps {
  source: IRelayLocationRedux[];
  locale: string;
  selectedLocation?: RelayLocation;
  selectedElementRef?: React.Ref<React.ReactInstance>;
  expandedItems?: RelayLocation[];
  disabledLocation?: { location: RelayLocation; reason: DisabledReason };
  onSelect?: (location: RelayLocation) => void;
  onExpand?: (location: RelayLocation, expand: boolean) => void;
  onWillExpand?: (locationRect: DOMRect, expandedContentHeight: number) => void;
  onTransitionEnd?: () => void;
}

interface Relay extends IRelayLocationRelayRedux {
  label: string;
  disabled: boolean;
}

interface City extends Omit<IRelayLocationCityRedux, 'relays'> {
  label: string;
  active: boolean;
  disabled: boolean;
  relays: Array<Relay>;
}

interface Country extends Omit<IRelayLocationRedux, 'cities'> {
  label: string;
  active: boolean;
  disabled: boolean;
  cities: Array<City>;
}

type CountryList = Array<Country>;

interface IRelayLocationsState {
  countries: CountryList;
}

interface ICommonCellProps {
  location: RelayLocation;
  selected: boolean;
  ref?: React.Ref<HTMLDivElement>;
}

export class RelayLocations extends React.PureComponent<
  IRelayLocationsProps,
  IRelayLocationsState
> {
  public state = {
    countries: this.prepareRelaysForPresentation(this.props.source),
  };

  public componentDidUpdate(prevProps: IRelayLocationsProps) {
    if (this.props.source !== prevProps.source) {
      this.setState({ countries: this.prepareRelaysForPresentation(this.props.source) });
    }
  }

  public render() {
    return (
      <Cell.Group noMarginBottom>
        {this.state.countries.map((relayCountry) => {
          const countryLocation: RelayLocation = { country: relayCountry.code };

          return (
            <LocationRow
              key={getLocationKey(countryLocation)}
              name={relayCountry.label}
              active={relayCountry.active}
              disabled={relayCountry.disabled}
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
                    name={relayCity.label}
                    active={relayCity.active}
                    disabled={relayCity.disabled}
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
                          name={relay.label}
                          active={relay.active}
                          disabled={relay.disabled}
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
      </Cell.Group>
    );
  }

  private prepareRelaysForPresentation(relayList: IRelayLocationRedux[]): CountryList {
    return relayList
      .map((country) => {
        const countryDisabled = this.isCountryDisabled(country, country.code);
        const countryLocation = { country: country.code };

        return {
          ...country,
          label: this.formatRowName(country.name, countryLocation, countryDisabled),
          active: countryDisabled !== DisabledReason.inactive,
          disabled: countryDisabled !== undefined,
          cities: country.cities
            .map((city) => {
              const cityDisabled =
                countryDisabled ?? this.isCityDisabled(city, [country.code, city.code]);
              const cityLocation: RelayLocation = { city: [country.code, city.code] };

              return {
                ...city,
                label: this.formatRowName(city.name, cityLocation, cityDisabled),
                active: cityDisabled !== DisabledReason.inactive,
                disabled: cityDisabled !== undefined,
                relays: city.relays
                  .map((relay) => {
                    const relayDisabled =
                      countryDisabled ??
                      cityDisabled ??
                      this.isRelayDisabled(relay, [country.code, city.code, relay.hostname]);
                    const relayLocation: RelayLocation = {
                      hostname: [country.code, city.code, relay.hostname],
                    };

                    return {
                      ...relay,
                      label: this.formatRowName(relay.hostname, relayLocation, relayDisabled),
                      disabled: relayDisabled !== undefined,
                    };
                  })
                  .sort((a, b) =>
                    a.hostname.localeCompare(b.hostname, this.props.locale, { numeric: true }),
                  ),
              };
            })
            .sort((a, b) => a.label.localeCompare(b.label, this.props.locale)),
        };
      })
      .sort((a, b) => a.label.localeCompare(b.label, this.props.locale));
  }

  private formatRowName(
    name: string,
    location: RelayLocation,
    disabledReason?: DisabledReason,
  ): string {
    const translatedName = 'hostname' in location ? name : relayLocations.gettext(name);
    const disabledLocation = this.props.disabledLocation;
    const matchDisabledLocation = compareRelayLocationLoose(location, disabledLocation?.location);

    let info: string | undefined;
    if (
      disabledReason === DisabledReason.entry ||
      (matchDisabledLocation && disabledLocation?.reason === DisabledReason.entry)
    ) {
      info = messages.pgettext('select-location-view', 'Entry');
    } else if (
      disabledReason === DisabledReason.exit ||
      (matchDisabledLocation && disabledLocation?.reason === DisabledReason.exit)
    ) {
      info = messages.pgettext('select-location-view', 'Exit');
    }

    return info !== undefined
      ? sprintf(
          // TRANSLATORS: This is used for appending information about a location.
          // TRANSLATORS: E.g. "Gothenburg (Entry)" if Gothenburg has been selected as the entrypoint.
          // TRANSLATORS: Available placeholders:
          // TRANSLATORS: %(location)s - Translated location name
          // TRANSLATORS: %(info)s - Information about the location
          messages.pgettext('select-location-view', '%(location)s (%(info)s)'),
          {
            location: translatedName,
            info,
          },
        )
      : translatedName;
  }

  private isRelayDisabled(
    relay: IRelayLocationRelayRedux,
    location: [string, string, string],
  ): DisabledReason | undefined {
    if (!relay.active) {
      return DisabledReason.inactive;
    } else if (
      this.props.disabledLocation &&
      compareRelayLocation({ hostname: location }, this.props.disabledLocation.location)
    ) {
      return this.props.disabledLocation.reason;
    } else {
      return undefined;
    }
  }

  private isCityDisabled(
    city: IRelayLocationCityRedux,
    location: [string, string],
  ): DisabledReason | undefined {
    const relaysDisabled = city.relays.map((relay) =>
      this.isRelayDisabled(relay, [...location, relay.hostname]),
    );
    if (relaysDisabled.every((status) => status === DisabledReason.inactive)) {
      return DisabledReason.inactive;
    }

    const disabledDueToSelection = relaysDisabled.find(
      (status) => status === DisabledReason.entry || status === DisabledReason.exit,
    );

    if (
      relaysDisabled.every((status) => status !== undefined) &&
      disabledDueToSelection !== undefined
    ) {
      return disabledDueToSelection;
    }

    if (
      this.props.disabledLocation &&
      compareRelayLocation({ city: location }, this.props.disabledLocation.location) &&
      city.relays.filter((relay) => relay.active).length <= 1
    ) {
      return this.props.disabledLocation.reason;
    }

    return undefined;
  }

  private isCountryDisabled(
    country: IRelayLocationRedux,
    location: string,
  ): DisabledReason | undefined {
    const citiesDisabled = country.cities.map((city) =>
      this.isCityDisabled(city, [location, city.code]),
    );
    if (citiesDisabled.every((status) => status === DisabledReason.inactive)) {
      return DisabledReason.inactive;
    }

    const disabledDueToSelection = citiesDisabled.find(
      (status) => status === DisabledReason.entry || status === DisabledReason.exit,
    );
    if (
      citiesDisabled.every((status) => status !== undefined) &&
      disabledDueToSelection !== undefined
    ) {
      return disabledDueToSelection;
    }

    if (
      this.props.disabledLocation &&
      compareRelayLocation({ country: location }, this.props.disabledLocation.location) &&
      country.cities.flatMap((city) => city.relays).filter((relay) => relay.active).length <= 1
    ) {
      return this.props.disabledLocation.reason;
    }

    return undefined;
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

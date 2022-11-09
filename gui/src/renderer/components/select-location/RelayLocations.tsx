import React from 'react';
import { sprintf } from 'sprintf-js';

import {
  compareRelayLocation,
  compareRelayLocationLoose,
  RelayLocation,
  relayLocationComponents,
} from '../../../shared/daemon-rpc-types';
import { messages, relayLocations } from '../../../shared/gettext';
import {
  IRelayLocationCityRedux,
  IRelayLocationRedux,
  IRelayLocationRelayRedux,
} from '../../redux/settings/reducers';
import * as Cell from '../cell';
import LocationRow from './LocationRow';
import { City, Country, Relay } from './types';

export enum DisabledReason {
  entry,
  exit,
  inactive,
}

interface IRelayLocationsProps {
  source: IRelayLocationRedux[];
  filter: string;
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

interface IRelayLocationsState {
  countries: Array<Country>;
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
    countries: this.applyFilter(this.prepareRelaysForPresentation(this.props.source)),
  };

  public componentDidUpdate(prevProps: IRelayLocationsProps) {
    if (
      this.props.source !== prevProps.source ||
      this.props.filter !== prevProps.filter ||
      this.props.expandedItems !== prevProps.expandedItems
    ) {
      this.setState({
        countries: this.applyFilter(this.prepareRelaysForPresentation(this.props.source)),
      });
    }
  }

  public render() {
    return (
      <Cell.Group noMarginBottom>
        {this.state.countries.map((relayCountry) => {
          return (
            <LocationRow
              key={getLocationKey(relayCountry.location)}
              name={relayCountry.label}
              active={relayCountry.active}
              disabled={relayCountry.disabled}
              expanded={relayCountry.expanded}
              expandable={!this.props.filter}
              onSelect={this.handleSelection}
              onExpand={this.handleExpand}
              onWillExpand={this.props.onWillExpand}
              onTransitionEnd={this.props.onTransitionEnd}
              {...this.getCommonCellProps(relayCountry.location)}>
              {relayCountry.cities.map((relayCity) => {
                return (
                  <LocationRow
                    key={getLocationKey(relayCity.location)}
                    name={relayCity.label}
                    active={relayCity.active}
                    disabled={relayCity.disabled}
                    expanded={relayCity.expanded}
                    expandable={!this.props.filter}
                    onSelect={this.handleSelection}
                    onExpand={this.handleExpand}
                    onWillExpand={this.props.onWillExpand}
                    onTransitionEnd={this.props.onTransitionEnd}
                    {...this.getCommonCellProps(relayCity.location)}>
                    {relayCity.relays.map((relay) => {
                      return (
                        <LocationRow
                          key={getLocationKey(relay.location)}
                          name={relay.label}
                          active={relay.active}
                          disabled={relay.disabled}
                          expandable={false}
                          onSelect={this.handleSelection}
                          {...this.getCommonCellProps(relay.location)}
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

  private prepareRelaysForPresentation(relayList: IRelayLocationRedux[]): Array<Country> {
    return relayList
      .map((country) => {
        const countryDisabled = this.isCountryDisabled(country, country.code);
        const countryLocation = { country: country.code };

        return {
          ...country,
          label: this.formatRowName(country.name, countryLocation, countryDisabled),
          location: countryLocation,
          active: countryDisabled !== DisabledReason.inactive,
          disabled: countryDisabled !== undefined,
          expanded: this.isExpanded(countryLocation),
          cities: country.cities
            .map((city) => {
              const cityDisabled =
                countryDisabled ?? this.isCityDisabled(city, [country.code, city.code]);
              const cityLocation: RelayLocation = { city: [country.code, city.code] };

              return {
                ...city,
                label: this.formatRowName(city.name, cityLocation, cityDisabled),
                location: cityLocation,
                active: cityDisabled !== DisabledReason.inactive,
                disabled: cityDisabled !== undefined,
                expanded: this.isExpanded(cityLocation),
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
                      location: relayLocation,
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

  private applyFilter(countries: Array<Country>): Array<Country> {
    if (!this.props.filter) {
      return countries;
    }

    const filter = this.props.filter.toLowerCase();
    return countries.reduce((countries, country) => {
      const cities = RelayLocations.filterCities(country.cities, filter);
      const match =
        cities.length > 0 ||
        country.code.toLowerCase().includes(filter) ||
        country.name.toLowerCase().includes(filter);
      return match
        ? [...countries, { ...country, expanded: cities.length > 0, cities }]
        : countries;
    }, [] as Array<Country>);
  }

  private static filterCities(cities: Array<City>, filter: string): Array<City> {
    return cities.reduce((cities, city) => {
      const relays = RelayLocations.filterRelays(city.relays, filter);
      const match =
        relays.length > 0 ||
        city.code.toLowerCase().includes(filter) ||
        city.name.toLowerCase().includes(filter);
      return match ? [...cities, { ...city, expanded: relays.length > 0, relays }] : cities;
    }, [] as Array<City>);
  }

  private static filterRelays(relays: Array<Relay>, filter: string): Array<Relay> {
    return relays.filter((relay) => relay.hostname.toLowerCase().includes(filter));
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

function getLocationKey(location: RelayLocation): string {
  return relayLocationComponents(location).join('-');
}

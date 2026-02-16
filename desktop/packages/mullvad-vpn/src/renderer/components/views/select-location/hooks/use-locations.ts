import React from 'react';

import type { RelayLocation as DaemonRelayLocation } from '../../../../../shared/daemon-rpc-types';
import type {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
  IRelayLocationRelayRedux,
} from '../../../../redux/settings/reducers';
import { useSelector } from '../../../../redux/store';
import {
  formatRowName,
  isCityDisabled,
  isCountryDisabled,
  isRelayDisabled,
  isSelected,
} from '../select-location-helpers';
import {
  type CityLocation,
  type CountryLocation,
  DisabledReason,
  type RelayLocation,
} from '../select-location-types';
import { useDisabledLocation, useSelectedLocation } from './';

export function useLocations(relayList: Array<IRelayLocationCountryRedux>): CountryLocation[] {
  const locale = useSelector((state) => state.userInterface.locale);
  const selectedLocation = useSelectedLocation();
  const disabledLocation = useDisabledLocation();

  return React.useMemo(() => {
    return relayList
      .map((country) => {
        return toCountryLocation(country, selectedLocation, disabledLocation, locale);
      })
      .sort((a, b) => a.label.localeCompare(b.label, locale));
  }, [relayList, disabledLocation, selectedLocation, locale]);
}

function toCountryLocation(
  country: IRelayLocationCountryRedux,
  selectedLocation: ReturnType<typeof useSelectedLocation>,
  disabledLocation: ReturnType<typeof useDisabledLocation>,
  locale: string,
): CountryLocation {
  {
    const countryLocation = { country: country.code };
    const countryDisabledReason = isCountryDisabled(country, countryLocation, disabledLocation);

    let hasSelectedChild = false;
    let hasExpandedChild = false;

    const cities = country.cities
      .map((city) => {
        const cityLocation = toCityLocation(
          country,
          city,
          selectedLocation,
          disabledLocation,
          countryDisabledReason,
          locale,
        );
        if (cityLocation.selected) {
          hasSelectedChild = true;
        }
        if (cityLocation.expanded) {
          hasExpandedChild = true;
        }
        return cityLocation;
      })
      .sort((a, b) => a.label.localeCompare(b.label, locale));

    return {
      type: 'country',
      label: formatRowName(country.name, countryLocation, countryDisabledReason),
      details: {
        country: country.code,
      },
      active: countryDisabledReason !== DisabledReason.inactive,
      disabled: countryDisabledReason !== undefined,
      disabledReason: countryDisabledReason,
      selected: isSelected(countryLocation, selectedLocation),
      expanded: hasExpandedChild || hasSelectedChild,
      cities,
    };
  }
}

function toCityLocation(
  country: IRelayLocationCountryRedux,
  city: IRelayLocationCityRedux,
  selectedLocation: ReturnType<typeof useSelectedLocation>,
  disabledLocation: ReturnType<typeof useDisabledLocation>,
  parentDisabledReason: DisabledReason | undefined,
  locale: string,
): CityLocation {
  let hasSelectedChild = false;

  const relays = city.relays
    .map((relay) => {
      const relayLocation = toRelayLocations(
        country,
        city,
        relay,
        selectedLocation,
        disabledLocation,
        parentDisabledReason,
      );
      if (relayLocation.selected) {
        hasSelectedChild = true;
      }
      return relayLocation;
    })
    .sort((a, b) => a.label.localeCompare(b.label, locale, { numeric: true }));

  const cityLocation: DaemonRelayLocation = { country: country.code, city: city.code };
  const cityDisabledReason =
    parentDisabledReason ?? isCityDisabled(city, cityLocation, disabledLocation);

  return {
    type: 'city',
    label: formatRowName(city.name, cityLocation, cityDisabledReason),
    details: {
      city: city.code,
      country: country.code,
    },
    active: cityDisabledReason !== DisabledReason.inactive,
    disabled: cityDisabledReason !== undefined,
    disabledReason: cityDisabledReason,
    selected: isSelected(cityLocation, selectedLocation),
    expanded: hasSelectedChild,
    relays,
  };
}

function toRelayLocations(
  country: IRelayLocationCountryRedux,
  city: IRelayLocationCityRedux,
  relay: IRelayLocationRelayRedux,
  selectedLocation: ReturnType<typeof useSelectedLocation>,
  disabledLocation: ReturnType<typeof useDisabledLocation>,
  parentDisabledReason: DisabledReason | undefined,
): RelayLocation {
  const relayLocation: DaemonRelayLocation = {
    country: country.code,
    city: city.code,
    hostname: relay.hostname,
  };

  const relayDisabledReason =
    parentDisabledReason ?? isRelayDisabled(relay, relayLocation, disabledLocation);

  return {
    type: 'relay',
    label: formatRowName(relay.hostname, relayLocation, relayDisabledReason),
    details: {
      country: country.code,
      city: city.code,
      hostname: relay.hostname,
    },
    active: relayDisabledReason !== DisabledReason.inactive,
    disabled: relayDisabledReason !== undefined,
    disabledReason: relayDisabledReason,
    selected: isSelected(relayLocation, selectedLocation),
    expanded: false,
  };
}

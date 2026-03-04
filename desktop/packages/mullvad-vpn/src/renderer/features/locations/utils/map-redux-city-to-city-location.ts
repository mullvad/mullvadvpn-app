import type { RelayLocation as DaemonRelayLocation } from '../../../../shared/daemon-rpc-types';
import type {
  IRelayLocationCityRedux,
  IRelayLocationCountryRedux,
} from '../../../redux/settings/reducers';
import type { useDisabledLocation, useSelectedLocation } from '../hooks';
import { type CityLocation, DisabledReason } from '../types';
import { createLocationLabel } from './create-location-label';
import { isCityDisabled } from './is-city-disabled';
import { isLocationSelected } from './is-location-selected';
import { mapReduxRelayToRelayLocation } from './map-redux-relay-to-relay-location';

export function mapReduxCityToCityLocation(
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
      const relayLocation = mapReduxRelayToRelayLocation(
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
  const label = createLocationLabel(city.name, cityLocation, cityDisabledReason);

  return {
    type: 'city',
    label,
    searchText: label.toLowerCase(),
    details: {
      city: city.code,
      country: country.code,
    },
    active: cityDisabledReason !== DisabledReason.inactive,
    disabled: cityDisabledReason !== undefined,
    disabledReason: cityDisabledReason,
    selected: isLocationSelected(cityLocation, selectedLocation),
    expanded: hasSelectedChild,
    relays,
  };
}

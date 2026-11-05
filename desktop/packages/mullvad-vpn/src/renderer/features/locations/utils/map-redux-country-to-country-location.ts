import type { RelayLocation as DaemonRelayLocation } from '../../../../shared/daemon-rpc-types';
import type { RelaySelectorRelayMatch } from '../../../../shared/relay-selector-rpc-types';
import type { IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import type { useDisabledLocation } from '../hooks/use-disabled-location';
import { type CountryLocation, DisabledReason } from '../types';
import { createLocationLabel } from './create-location-label';
import { isCountryDisabled } from './is-country-disabled';
import { isLocationSelected } from './is-location-selected';
import { mapReduxCityToCityLocation } from './map-redux-city-to-city-location';

export function mapReduxCountryToCountryLocation(
  country: IRelayLocationCountryRedux,
  selectedLocation: DaemonRelayLocation | undefined,
  disabledLocation: ReturnType<typeof useDisabledLocation>,
  locale: string,
  matches: RelaySelectorRelayMatch[],
): CountryLocation {
  {
    const countryLocation = { country: country.code };
    const countryDisabledReason = isCountryDisabled(country, countryLocation, disabledLocation);

    const cities = country.cities
      .map((city) => {
        const cityLocation = mapReduxCityToCityLocation(
          country,
          city,
          selectedLocation,
          disabledLocation,
          countryDisabledReason,
          locale,
          matches,
        );

        return cityLocation;
      })
      .sort((a, b) => a.label.localeCompare(b.label, locale));

    const label = createLocationLabel(country.name, countryLocation, countryDisabledReason);
    const hasSelectedChild = cities.some(({ selected }) => selected);
    const hasExpandedChild = cities.some(({ expanded }) => expanded);
    const needsOtherEntry =
      cities.length > 0 && cities.every(({ needsOtherEntry }) => needsOtherEntry);

    return {
      type: 'country',
      label,
      details: {
        country: country.code,
      },
      active: countryDisabledReason !== DisabledReason.inactive,
      disabled: countryDisabledReason !== undefined,
      disabledReason: countryDisabledReason,
      selected: isLocationSelected(countryLocation, selectedLocation),
      expanded: hasExpandedChild || hasSelectedChild,
      cities,
      needsOtherEntry,
    };
  }
}

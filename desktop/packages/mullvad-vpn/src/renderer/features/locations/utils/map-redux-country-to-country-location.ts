import type { IRelayLocationCountryRedux } from '../../../redux/settings/reducers';
import type { useDisabledLocation } from '../hooks/use-disabled-location';
import type { useSelectedLocation } from '../hooks/use-selected-location';
import { type CountryLocation, DisabledReason } from '../types';
import { createLocationLabel } from './create-location-label';
import { isCountryDisabled } from './is-country-disabled';
import { isLocationSelected } from './is-location-selected';
import { mapReduxCityToCityLocation } from './map-redux-city-to-city-location';

export function mapReduxCountryToCountryLocation(
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
        const cityLocation = mapReduxCityToCityLocation(
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

    const label = createLocationLabel(country.name, countryLocation, countryDisabledReason);

    return {
      type: 'country',
      label,
      searchText: label.toLowerCase(),
      details: {
        country: country.code,
      },
      active: countryDisabledReason !== DisabledReason.inactive,
      disabled: countryDisabledReason !== undefined,
      disabledReason: countryDisabledReason,
      selected: isLocationSelected(countryLocation, selectedLocation),
      expanded: hasExpandedChild || hasSelectedChild,
      cities,
    };
  }
}

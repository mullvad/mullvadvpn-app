import { findCityForRelay } from '../../../../../../features/locations/utils';
import { type IRelayLocationCountryRedux } from '../../../../../../redux/settings/reducers';

export function getCitiesForHostnames(
  hostnames: string[],
  locations: IRelayLocationCountryRedux[],
) {
  const cities = hostnames
    .map((hostname) => findCityForRelay(hostname, locations))
    .filter((city) => city !== undefined);

  return cities; // TODO: remove duplicates
}
